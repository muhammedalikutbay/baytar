use super::{now, transaction, RepoError, Result};
use crate::model::Charge;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCharge {
    pub owner_id: i64,
    pub patient_id: Option<i64>,
    pub visit_id: Option<i64>,
    pub service_id: Option<i64>,
    pub inventory_item_id: Option<i64>,
    pub description: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
    pub discount_minor: i64,
    pub vat_rate: Option<i64>,
    pub charged_on: String,
    pub notes: Option<String>,
}
const COLUMNS: &str = "id, owner_id, patient_id, visit_id, service_id, inventory_item_id, description, quantity, unit_price_minor, discount_minor, vat_rate, charged_on, notes, total_minor, created_at, updated_at, deleted_at";
fn row(r: &Row<'_>) -> rusqlite::Result<Charge> {
    Ok(Charge {
        id: r.get(0)?,
        owner_id: r.get(1)?,
        patient_id: r.get(2)?,
        visit_id: r.get(3)?,
        service_id: r.get(4)?,
        inventory_item_id: r.get(5)?,
        description: r.get(6)?,
        quantity: r.get(7)?,
        unit_price_minor: r.get(8)?,
        discount_minor: r.get(9)?,
        vat_rate: r.get(10)?,
        charged_on: r.get(11)?,
        notes: r.get(12)?,
        total_minor: r.get(13)?,
        created_at: r.get(14)?,
        updated_at: r.get(15)?,
        deleted_at: r.get(16)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Charge> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM charges WHERE id=?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(conn: &Connection, include_deleted: bool) -> Result<Vec<Charge>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM charges WHERE (?1 OR deleted_at IS NULL) ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([include_deleted], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
pub fn create(conn: &Connection, input: NewCharge) -> Result<Charge> {
    let tx = transaction(conn)?;
    validate(&tx, &input)?;
    let total = total_minor(input.quantity, input.unit_price_minor, input.discount_minor)?;
    tx.execute("INSERT INTO charges (owner_id, patient_id, visit_id, service_id, inventory_item_id, description, quantity, unit_price_minor, discount_minor, vat_rate, charged_on, notes, total_minor) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", params![input.owner_id, input.patient_id, input.visit_id, input.service_id, input.inventory_item_id, input.description, input.quantity, input.unit_price_minor, input.discount_minor, input.vat_rate, input.charged_on, input.notes, total])?;
    let result = get(&tx, tx.last_insert_rowid())?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, input: NewCharge) -> Result<Charge> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    validate(&tx, &input)?;
    let total = total_minor(input.quantity, input.unit_price_minor, input.discount_minor)?;
    tx.execute("UPDATE charges SET owner_id=?1, patient_id=?2, visit_id=?3, service_id=?4, inventory_item_id=?5, description=?6, quantity=?7, unit_price_minor=?8, discount_minor=?9, vat_rate=?10, charged_on=?11, notes=?12, total_minor=?13, updated_at=?14 WHERE id=?15", params![input.owner_id, input.patient_id, input.visit_id, input.service_id, input.inventory_item_id, input.description, input.quantity, input.unit_price_minor, input.discount_minor, input.vat_rate, input.charged_on, input.notes, total, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Charge> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Charge> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Charge> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE charges SET deleted_at=?1, updated_at=?2 WHERE id=?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
/// Positive half-minor-unit values round up; totals stay within the exact JS integer range.
pub fn total_minor(quantity: f64, unit_price_minor: i64, discount_minor: i64) -> Result<i64> {
    super::positive("quantity", Some(quantity))?;
    super::accounts::money("unitPriceMinor", Some(unit_price_minor))?;
    super::accounts::money("discountMinor", Some(discount_minor))?;
    let gross = (quantity * unit_price_minor as f64).round();
    if !gross.is_finite() || gross > super::accounts::MAX_MONEY as f64 {
        return Err(super::invalid("quantity", "total is too large"));
    }
    let gross = gross as i64;
    if discount_minor > gross {
        return Err(super::invalid("discountMinor", "must not exceed gross"));
    }
    Ok(gross - discount_minor)
}
fn validate(conn: &Connection, input: &NewCharge) -> Result<()> {
    super::accounts::active_owner(conn, input.owner_id)?;
    super::required("description", &input.description)?;
    super::date("chargedOn", &input.charged_on)?;
    super::accounts::vat(input.vat_rate)?;
    total_minor(input.quantity, input.unit_price_minor, input.discount_minor)?;
    if let Some(id) = input.patient_id {
        if super::active_patient(conn, id)?.owner_id != input.owner_id {
            return Err(super::invalid("patientId", "must belong to owner"));
        }
    }
    if let Some(id) = input.visit_id {
        let visit = super::visits::get(conn, id)?;
        let patient = super::active_patient(conn, visit.patient_id)?;
        if visit.deleted_at.is_some()
            || patient.owner_id != input.owner_id
            || input.patient_id.is_some_and(|id| id != visit.patient_id)
        {
            return Err(super::invalid(
                "visitId",
                "must belong to patient and owner",
            ));
        }
    }
    if let Some(id) = input.service_id {
        if super::services::get(conn, id)?.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    if let Some(id) = input.inventory_item_id {
        let active: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE id=?1 AND deleted_at IS NULL)",
            [id],
            |r| r.get(0),
        )?;
        if !active {
            return Err(RepoError::NotFound);
        }
    }
    Ok(())
}
pub fn by_visit(conn: &Connection, visit_id: i64, include_deleted: bool) -> Result<Vec<Charge>> {
    let mut stmt=conn.prepare(&format!("SELECT {COLUMNS} FROM charges WHERE visit_id=?1 AND (?2 OR deleted_at IS NULL) ORDER BY charged_on,id"))?;
    let rows = stmt
        .query_map(params![visit_id, include_deleted], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
