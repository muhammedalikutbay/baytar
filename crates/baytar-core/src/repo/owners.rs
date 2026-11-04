use super::{clean, digits, phone_value, ListQuery};
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
    #[serde(default)]
    pub national_id: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub district: Option<String>,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub national_id: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub city: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub district: Option<Option<String>>,
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
        national_id: r.get(12)?,
        city: r.get(13)?,
        district: r.get(14)?,
    })
}
const COLUMNS: &str = "id, full_name, phone, phone_raw, phone_alt, email, address, notes, search_text, created_at, updated_at, deleted_at, national_id, city, district";
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
/// Folded search key: name, every digit form of both phones (E.164 digits plus the Turkish
/// national `0` form, so typing `0532...` still finds `+90532...`), and the city.
pub(crate) fn search_key(
    full_name: &str,
    phone: Option<&str>,
    phone_alt: Option<&str>,
    city: Option<&str>,
) -> String {
    fold(&format!(
        "{} {} {} {}",
        full_name,
        phone_terms(phone),
        phone_terms(phone_alt),
        city.unwrap_or_default()
    ))
}
fn phone_terms(phone: Option<&str>) -> String {
    let all = digits(phone).unwrap_or_default();
    match phone
        .and_then(|p| p.trim().strip_prefix("+90"))
        .filter(|national| national.len() == 10 && national.bytes().all(|b| b.is_ascii_digit()))
    {
        Some(national) => format!("{all} 0{national}"),
        None => all,
    }
}
/// Free-text fields trimmed (empty becomes NULL). Phones are canonicalised by the callers, so an
/// unrelated edit never rewrites a stored number.
fn stored(input: NewOwner) -> NewOwner {
    NewOwner {
        national_id: clean(input.national_id),
        city: clean(input.city),
        district: clean(input.district),
        ..input
    }
}
pub fn create(conn: &Connection, input: NewOwner) -> Result<Owner> {
    validate(&input)?;
    let phone_raw = input.phone.clone();
    let input = stored(NewOwner {
        phone: phone_value(input.phone.as_deref()),
        phone_alt: phone_value(input.phone_alt.as_deref()),
        ..input
    });
    let tx = transaction(conn)?;
    let stamp = now();
    let key = search_key(
        &input.full_name,
        input.phone.as_deref(),
        input.phone_alt.as_deref(),
        input.city.as_deref(),
    );
    tx.execute("INSERT INTO owners (full_name, phone, phone_raw, phone_alt, email, address, notes, national_id, city, district, search_text, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![input.full_name, input.phone, phone_raw, input.phone_alt, input.email, input.address, input.notes, input.national_id, input.city, input.district, key, stamp])?;
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
        Some(raw) => (phone_value(raw.as_deref()), raw),
        None => (old.phone, old.phone_raw),
    };
    let input = stored(NewOwner {
        full_name: patch.full_name.unwrap_or(old.full_name),
        phone,
        phone_alt: match patch.phone_alt {
            Some(raw) => phone_value(raw.as_deref()),
            None => old.phone_alt,
        },
        email: patch.email.unwrap_or(old.email),
        address: patch.address.unwrap_or(old.address),
        notes: patch.notes.unwrap_or(old.notes),
        national_id: patch.national_id.unwrap_or(old.national_id),
        city: patch.city.unwrap_or(old.city),
        district: patch.district.unwrap_or(old.district),
    });
    validate(&input)?;
    let stamp = now();
    let key = search_key(
        &input.full_name,
        input.phone.as_deref(),
        input.phone_alt.as_deref(),
        input.city.as_deref(),
    );
    tx.execute("UPDATE owners SET full_name = ?1, phone = ?2, phone_raw = ?3, phone_alt = ?4, email = ?5, address = ?6, notes = ?7, national_id = ?8, city = ?9, district = ?10, search_text = ?11, updated_at = ?12 WHERE id = ?13", params![input.full_name, input.phone, phone_raw, input.phone_alt, input.email, input.address, input.notes, input.national_id, input.city, input.district, key, stamp, id])?;
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
    let key = conn.query_row(
        "SELECT full_name, phone, phone_alt, city FROM owners WHERE id = ?1",
        [id],
        |r| {
            Ok(search_key(
                &r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?.as_deref(),
                r.get::<_, Option<String>>(2)?.as_deref(),
                r.get::<_, Option<String>>(3)?.as_deref(),
            ))
        },
    )?;
    conn.execute(
        "UPDATE owners SET deleted_at = ?1, updated_at = ?2, search_text = ?3 WHERE id = ?4",
        params![deleted, stamp, key, id],
    )?;
    Ok(())
}
