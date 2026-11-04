use super::{now, transaction, RepoError, Result};
use crate::model::{Service, ServiceCategory};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewService {
    pub name: String,
    pub category: ServiceCategory,
    pub price_minor: Option<i64>,
    pub vat_rate: Option<i64>,
    pub is_active: bool,
    pub notes: Option<String>,
}
const COLUMNS: &str = "id, name, category, price_minor, vat_rate, is_active, notes, is_builtin, created_at, updated_at, deleted_at";
fn row(r: &Row<'_>) -> rusqlite::Result<Service> {
    Ok(Service {
        id: r.get(0)?,
        name: r.get(1)?,
        category: r.get::<_, String>(2)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?,
        price_minor: r.get(3)?,
        vat_rate: r.get(4)?,
        is_active: r.get(5)?,
        notes: r.get(6)?,
        is_builtin: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
        deleted_at: r.get(10)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Service> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM services WHERE id=?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(conn: &Connection, include_deleted: bool) -> Result<Vec<Service>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM services WHERE (?1 OR deleted_at IS NULL) ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([include_deleted], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
pub fn create(conn: &Connection, input: NewService) -> Result<Service> {
    let tx = transaction(conn)?;
    validate(&tx, &input)?;

    tx.execute("INSERT INTO services (name, category, price_minor, vat_rate, is_active, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![input.name, input.category.as_str(), input.price_minor, input.vat_rate, input.is_active, input.notes])?;
    let result = get(&tx, tx.last_insert_rowid())?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, input: NewService) -> Result<Service> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    validate(&tx, &input)?;

    tx.execute("UPDATE services SET name=?1, category=?2, price_minor=?3, vat_rate=?4, is_active=?5, notes=?6, updated_at=?7 WHERE id=?8", params![input.name, input.category.as_str(), input.price_minor, input.vat_rate, input.is_active, input.notes, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Service> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Service> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Service> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE services SET deleted_at=?1, updated_at=?2 WHERE id=?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
fn validate(_conn: &Connection, input: &NewService) -> Result<()> {
    super::required("name", &input.name)?;
    super::accounts::money("priceMinor", input.price_minor)?;
    super::accounts::vat(input.vat_rate)
}
