use super::{now, transaction, RepoError, Result};
use crate::model::{Payment, PaymentMethod};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPayment {
    pub owner_id: i64,
    pub paid_on: String,
    pub amount_minor: i64,
    pub method: PaymentMethod,
    pub notes: Option<String>,
}
const COLUMNS: &str =
    "id, owner_id, paid_on, amount_minor, method, notes, created_at, updated_at, deleted_at";
fn row(r: &Row<'_>) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: r.get(0)?,
        owner_id: r.get(1)?,
        paid_on: r.get(2)?,
        amount_minor: r.get(3)?,
        method: r.get::<_, String>(4)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
        })?,
        notes: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
        deleted_at: r.get(8)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Payment> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM payments WHERE id=?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(conn: &Connection, include_deleted: bool) -> Result<Vec<Payment>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM payments WHERE (?1 OR deleted_at IS NULL) ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([include_deleted], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
pub fn create(conn: &Connection, input: NewPayment) -> Result<Payment> {
    let tx = transaction(conn)?;
    validate(&tx, &input)?;

    tx.execute("INSERT INTO payments (owner_id, paid_on, amount_minor, method, notes) VALUES (?1, ?2, ?3, ?4, ?5)", params![input.owner_id, input.paid_on, input.amount_minor, input.method.as_str(), input.notes])?;
    let result = get(&tx, tx.last_insert_rowid())?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, input: NewPayment) -> Result<Payment> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    validate(&tx, &input)?;

    tx.execute("UPDATE payments SET owner_id=?1, paid_on=?2, amount_minor=?3, method=?4, notes=?5, updated_at=?6 WHERE id=?7", params![input.owner_id, input.paid_on, input.amount_minor, input.method.as_str(), input.notes, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Payment> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Payment> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Payment> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE payments SET deleted_at=?1, updated_at=?2 WHERE id=?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
fn validate(conn: &Connection, input: &NewPayment) -> Result<()> {
    super::accounts::active_owner(conn, input.owner_id)?;
    super::date("paidOn", &input.paid_on)?;
    if input.amount_minor <= 0 {
        return Err(super::invalid("amountMinor", "must be greater than zero"));
    }
    super::accounts::money("amountMinor", Some(input.amount_minor))
}
