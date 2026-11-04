use super::{clean, invalid, owners};
use super::{now, required, transaction, RepoError, Result};
use crate::{model::Patient, text::fold};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPatient {
    pub owner_id: i64,
    pub name: String,
    pub species: String,
    #[serde(default)]
    pub species_note: Option<String>,
    #[serde(default)]
    pub breed: Option<String>,
    #[serde(default = "unknown_sex")]
    pub sex: String,
    #[serde(default)]
    pub neutered: Option<bool>,
    #[serde(default)]
    pub birth_date: Option<String>,
    #[serde(default)]
    pub birth_date_is_estimate: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub microchip: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub deceased_on: Option<String>,
    #[serde(default)]
    pub alerts: Option<String>,
}
fn unknown_sex() -> String {
    "unknown".into()
}
/// Omitted fields are unchanged; null clears nullable fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub species_note: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub breed: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub neutered: Option<Option<bool>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub birth_date: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_date_is_estimate: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub color: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub microchip: Option<Option<String>>,
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
    pub deceased_on: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub alerts: Option<Option<String>>,
}
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Patient> {
    Ok(Patient {
        id: r.get(0)?,
        owner_id: r.get(1)?,
        name: r.get(2)?,
        species: r.get::<_, String>(3)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?,
        species_note: r.get(4)?,
        breed: r.get(5)?,
        sex: r.get::<_, String>(6)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
        })?,
        neutered: r.get(7)?,
        birth_date: r.get(8)?,
        birth_date_is_estimate: r.get(9)?,
        color: r.get(10)?,
        microchip: r.get(11)?,
        notes: r.get(12)?,
        deceased_on: r.get(13)?,
        search_text: r.get(14)?,
        created_at: r.get(15)?,
        updated_at: r.get(16)?,
        deleted_at: r.get(17)?,
        alerts: r.get(18)?,
    })
}
const COLUMNS: &str = "id, owner_id, name, species, species_note, breed, sex, neutered, birth_date, birth_date_is_estimate, color, microchip, notes, deceased_on, search_text, created_at, updated_at, deleted_at, alerts";
pub fn get(conn: &Connection, id: i64) -> Result<Patient> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM patients WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list_by_owner(conn: &Connection, owner_id: i64) -> Result<Vec<Patient>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM patients WHERE owner_id = ?1 AND deleted_at IS NULL AND EXISTS (SELECT 1 FROM owners WHERE id = ?1 AND deleted_at IS NULL) ORDER BY search_text, id"))?;
    let rows = stmt
        .query_map([owner_id], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
fn validate(input: &NewPatient) -> Result<()> {
    required("name", &input.name)?;
    input
        .species
        .parse::<crate::model::Species>()
        .map_err(|_| invalid("species", "unknown species"))?;
    input
        .sex
        .parse::<crate::model::Sex>()
        .map_err(|_| invalid("sex", "unknown sex"))?;
    for (field, value) in [
        ("birthDate", &input.birth_date),
        ("deceasedOn", &input.deceased_on),
    ] {
        if let Some(value) = value {
            let valid = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .is_ok_and(|date| date.format("%Y-%m-%d").to_string() == *value);
            if !valid {
                return Err(invalid(field, "expected a valid YYYY-MM-DD date"));
            }
        }
    }
    Ok(())
}
fn search_text(input: &NewPatient) -> String {
    fold(&format!(
        "{} {} {}",
        input.name,
        input.breed.as_deref().unwrap_or_default(),
        input.species_note.as_deref().unwrap_or_default()
    ))
}
pub fn create(conn: &Connection, input: NewPatient) -> Result<Patient> {
    validate(&input)?;
    let tx = transaction(conn)?;
    if owners::get(&tx, input.owner_id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let stamp = now();
    let key = search_text(&input);
    tx.execute("INSERT INTO patients (owner_id, name, species, species_note, breed, sex, neutered, birth_date, birth_date_is_estimate, color, microchip, notes, deceased_on, alerts, search_text, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)", params![input.owner_id, input.name, input.species, input.species_note, input.breed, input.sex, input.neutered, input.birth_date, input.birth_date_is_estimate, input.color, input.microchip, input.notes, input.deceased_on, clean(input.alerts), key, stamp])?;
    let id = tx.last_insert_rowid();
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, patch: PatientPatch) -> Result<Patient> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let input = NewPatient {
        owner_id: patch.owner_id.unwrap_or(old.owner_id),
        name: patch.name.unwrap_or(old.name),
        species: patch.species.unwrap_or(old.species.as_str().to_string()),
        species_note: patch.species_note.unwrap_or(old.species_note),
        breed: patch.breed.unwrap_or(old.breed),
        sex: patch.sex.unwrap_or(old.sex.as_str().to_string()),
        neutered: patch.neutered.unwrap_or(old.neutered),
        birth_date: patch.birth_date.unwrap_or(old.birth_date),
        birth_date_is_estimate: patch
            .birth_date_is_estimate
            .unwrap_or(old.birth_date_is_estimate),
        color: patch.color.unwrap_or(old.color),
        microchip: patch.microchip.unwrap_or(old.microchip),
        notes: patch.notes.unwrap_or(old.notes),
        deceased_on: patch.deceased_on.unwrap_or(old.deceased_on),
        alerts: patch.alerts.unwrap_or(old.alerts),
    };
    validate(&input)?;
    if owners::get(&tx, input.owner_id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let stamp = now();
    let key = search_text(&input);
    tx.execute("UPDATE patients SET owner_id = ?1, name = ?2, species = ?3, species_note = ?4, breed = ?5, sex = ?6, neutered = ?7, birth_date = ?8, birth_date_is_estimate = ?9, color = ?10, microchip = ?11, notes = ?12, deceased_on = ?13, alerts = ?14, search_text = ?15, updated_at = ?16 WHERE id = ?17", params![input.owner_id, input.name, input.species, input.species_note, input.breed, input.sex, input.neutered, input.birth_date, input.birth_date_is_estimate, input.color, input.microchip, input.notes, input.deceased_on, clean(input.alerts), key, stamp, id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Patient> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::Conflict);
    }
    if owners::get(&tx, old.owner_id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let stamp = now();
    set_deleted(&tx, id, Some(&stamp), &stamp)?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Patient> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_none() {
        return Err(RepoError::Conflict);
    }
    if owners::get(&tx, old.owner_id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let stamp = now();
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
    let raw: String = conn.query_row("SELECT name || ' ' || coalesce(breed, '') || ' ' || coalesce(species_note, '') FROM patients WHERE id = ?1", [id], |r| r.get(0))?;
    conn.execute(
        "UPDATE patients SET deleted_at = ?1, updated_at = ?2, search_text = ?3 WHERE id = ?4",
        params![deleted, stamp, fold(&raw), id],
    )?;
    Ok(())
}

impl Default for NewPatient {
    fn default() -> Self {
        Self {
            owner_id: 0,
            name: String::new(),
            species: String::new(),
            sex: unknown_sex(),
            birth_date_is_estimate: false,
            species_note: None,
            breed: None,
            neutered: None,
            birth_date: None,
            color: None,
            microchip: None,
            notes: None,
            deceased_on: None,
            alerts: None,
        }
    }
}
