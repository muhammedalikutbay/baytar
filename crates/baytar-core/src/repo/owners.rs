use super::{digits, ListQuery};
use super::{now, required, transaction, RepoError, Result};
use crate::{model::Owner, text::fold};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOwner {
    pub full_name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub phone_alt: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}
/// Omitted fields are unchanged; null clears nullable fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub phone: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub phone_alt: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub email: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub address: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub notes: Option<Option<String>>,
}
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Owner> {
    Ok(Owner {
        id: r.get(0)?,
        full_name: r.get(1)?,
        phone: r.get(2)?,
        phone_raw: r.get(3)?,
        phone_alt: r.get(4)?,
        email: r.get(5)?,
        address: r.get(6)?,
        notes: r.get(7)?,
        search_text: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
        deleted_at: r.get(11)?,
    })
}
const COLUMNS: &str = "id, full_name, phone, phone_raw, phone_alt, email, address, notes, search_text, created_at, updated_at, deleted_at";
pub fn get(conn: &Connection, id: i64) -> Result<Owner> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM owners WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(conn: &Connection, query: ListQuery) -> Result<Vec<Owner>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM owners WHERE (?1 OR deleted_at IS NULL) ORDER BY search_text, id LIMIT ?2 OFFSET ?3"))?;
    let rows = stmt
        .query_map(
            params![query.include_deleted, query.limit, query.offset],
            row,
        )?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerDetail {
    pub owner: Owner,
    pub patients: Vec<crate::model::Patient>,
}
pub fn owner_detail(conn: &Connection, id: i64) -> Result<OwnerDetail> {
    Ok(OwnerDetail {
        owner: get(conn, id)?,
        patients: super::patients::list_by_owner(conn, id)?,
    })
}
fn validate(input: &NewOwner) -> Result<()> {
    required("fullName", &input.full_name)?;
    Ok(())
}
fn search_text(input: &NewOwner) -> String {
    fold(&format!(
        "{} {} {}",
        input.full_name,
        digits(input.phone.as_deref()).unwrap_or_default(),
        digits(input.phone_alt.as_deref()).unwrap_or_default()
    ))
}
pub fn create(conn: &Connection, input: NewOwner) -> Result<Owner> {
    validate(&input)?;
    let tx = transaction(conn)?;
    let stamp = now();
    let key = search_text(&input);
    tx.execute("INSERT INTO owners (full_name, phone, phone_raw, phone_alt, email, address, notes, search_text, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![input.full_name, digits(input.phone.as_deref()), input.phone, digits(input.phone_alt.as_deref()), input.email, input.address, input.notes, key, stamp])?;
    let id = tx.last_insert_rowid();
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, patch: OwnerPatch) -> Result<Owner> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    // Imported phone_raw can differ from the normalized number (e.g. a restored
    // leading zero). An unrelated edit must preserve both existing values.
    let (phone, phone_raw) = match patch.phone {
        Some(raw) => (digits(raw.as_deref()), raw),
        None => (old.phone, old.phone_raw),
    };
    let input = NewOwner {
        full_name: patch.full_name.unwrap_or(old.full_name),
        phone,
        phone_alt: patch.phone_alt.unwrap_or(old.phone_alt),
        email: patch.email.unwrap_or(old.email),
        address: patch.address.unwrap_or(old.address),
        notes: patch.notes.unwrap_or(old.notes),
    };
    validate(&input)?;
    let stamp = now();
    let key = search_text(&input);
    tx.execute("UPDATE owners SET full_name = ?1, phone = ?2, phone_raw = ?3, phone_alt = ?4, email = ?5, address = ?6, notes = ?7, search_text = ?8, updated_at = ?9 WHERE id = ?10", params![input.full_name, input.phone, phone_raw, digits(input.phone_alt.as_deref()), input.email, input.address, input.notes, key, stamp, id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Vec<i64>> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    // A prior independent deletion must never share this cascade token.
    let mut stamp = stamp;
    while tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM patients WHERE owner_id = ?1 AND deleted_at = ?2)",
        params![id, stamp],
        |r| r.get::<_, bool>(0),
    )? {
        stamp = (chrono::DateTime::parse_from_rfc3339(&stamp)
            .map_err(|_| super::invalid("deletedAt", "invalid timestamp"))?
            + chrono::Duration::nanoseconds(1))
        .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    }
    let ids = super::patients::list_by_owner(&tx, id)?
        .into_iter()
        .map(|p| p.id)
        .collect::<Vec<_>>();
    for patient_id in &ids {
        super::patients::set_deleted(&tx, *patient_id, Some(&stamp), &stamp)?;
    }
    set_deleted(&tx, id, Some(&stamp), &stamp)?;
    tx.commit()?;
    Ok(ids)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Owner> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    let deleted = old.deleted_at.as_deref().ok_or(RepoError::Conflict)?;
    let stamp = now();
    let ids = {
        let mut stmt =
            tx.prepare("SELECT id FROM patients WHERE owner_id = ?1 AND deleted_at = ?2")?;
        let rows = stmt
            .query_map(params![id, deleted], |r| r.get::<_, i64>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows
    };
    for patient_id in ids {
        super::patients::set_deleted(&tx, patient_id, None, &stamp)?;
    }
    set_deleted(&tx, id, None, &stamp)?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub(crate) fn set_deleted(
    conn: &Connection,
    id: i64,
    deleted: Option<&str>,
    stamp: &str,
) -> Result<()> {
    let raw: String = conn.query_row("SELECT full_name || ' ' || coalesce(phone, '') || ' ' || coalesce(phone_alt, '') FROM owners WHERE id = ?1", [id], |r| r.get(0))?;
    conn.execute(
        "UPDATE owners SET deleted_at = ?1, updated_at = ?2, search_text = ?3 WHERE id = ?4",
        params![deleted, stamp, fold(&raw), id],
    )?;
    Ok(())
}
