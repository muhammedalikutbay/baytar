use super::{now, transaction, RepoError, Result};
use crate::model::VaccineType;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVaccineType {
    pub name: String,
    #[serde(default)]
    pub species: Option<String>,
    #[serde(default)]
    pub default_interval_days: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub diseases: Option<String>,
    #[serde(default)]
    pub primary_doses: Option<i64>,
    #[serde(default)]
    pub primary_interval_days: Option<i64>,
    #[serde(default)]
    pub min_age_weeks: Option<i64>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccineTypePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub species: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub default_interval_days: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub notes: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub manufacturer: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub diseases: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub primary_doses: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub primary_interval_days: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub min_age_weeks: Option<Option<i64>>,
}
fn default_is_active() -> bool {
    true
}
impl Default for NewVaccineType {
    fn default() -> Self {
        Self {
            name: Default::default(),
            species: Default::default(),
            default_interval_days: Default::default(),
            notes: Default::default(),
            is_active: true,
            is_builtin: false,
            manufacturer: Default::default(),
            diseases: Default::default(),
            primary_doses: Default::default(),
            primary_interval_days: Default::default(),
            min_age_weeks: Default::default(),
        }
    }
}
const COLUMNS: &str = "id, name, species, default_interval_days, notes, is_active, created_at, updated_at, deleted_at, is_builtin, manufacturer, diseases, primary_doses, primary_interval_days, min_age_weeks";
pub fn available_for_patient(conn: &Connection, id: i64, patient_id: i64) -> Result<VaccineType> {
    let patient = super::active_patient(conn, patient_id)?;
    let vaccine = get(conn, id)?;
    if vaccine.deleted_at.is_some() || !vaccine.is_active {
        return Err(RepoError::NotFound);
    }
    if vaccine
        .species
        .is_some_and(|species| species != patient.species)
    {
        return Err(super::invalid(
            "vaccineTypeId",
            "vaccine species must match patient",
        ));
    }
    Ok(vaccine)
}
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<VaccineType> {
    Ok(VaccineType {
        id: r.get(0)?,
        name: r.get(1)?,
        species: r
            .get::<_, Option<String>>(2)?
            .map(|v| v.parse())
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        default_interval_days: r.get(3)?,
        notes: r.get(4)?,
        is_active: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
        deleted_at: r.get(8)?,
        is_builtin: r.get(9)?,
        manufacturer: r.get(10)?,
        diseases: r.get(11)?,
        primary_doses: r.get(12)?,
        primary_interval_days: r.get(13)?,
        min_age_weeks: r.get(14)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<VaccineType> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM vaccine_types WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewVaccineType) -> Result<VaccineType> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewVaccineType) -> Result<VaccineType> {
    conn.execute("INSERT INTO vaccine_types (name, species, default_interval_days, notes, is_active, is_builtin, manufacturer, diseases, primary_doses, primary_interval_days, min_age_weeks, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![input.name, input.species, input.default_interval_days, input.notes, input.is_active, input.is_builtin, input.manufacturer, input.diseases, input.primary_doses, input.primary_interval_days, input.min_age_weeks, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: VaccineTypePatch) -> Result<VaccineType> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewVaccineType {
        name: patch.name.unwrap_or(old.name),
        species: patch
            .species
            .unwrap_or(old.species.map(|v| v.as_str().to_string())),
        default_interval_days: patch
            .default_interval_days
            .unwrap_or(old.default_interval_days),
        notes: patch.notes.unwrap_or(old.notes),
        is_active: patch.is_active.unwrap_or(old.is_active),
        is_builtin: old.is_builtin,
        manufacturer: patch.manufacturer.unwrap_or(old.manufacturer),
        diseases: patch.diseases.unwrap_or(old.diseases),
        primary_doses: patch.primary_doses.unwrap_or(old.primary_doses),
        primary_interval_days: patch
            .primary_interval_days
            .unwrap_or(old.primary_interval_days),
        min_age_weeks: patch.min_age_weeks.unwrap_or(old.min_age_weeks),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE vaccine_types SET name = ?1, species = ?2, default_interval_days = ?3, notes = ?4, is_active = ?5, manufacturer = ?6, diseases = ?7, primary_doses = ?8, primary_interval_days = ?9, min_age_weeks = ?10, updated_at = ?11 WHERE id = ?12", params![input.name, input.species, input.default_interval_days, input.notes, input.is_active, input.manufacturer, input.diseases, input.primary_doses, input.primary_interval_days, input.min_age_weeks, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<VaccineType> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<VaccineType> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<VaccineType> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE vaccine_types SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(_conn: &Connection, input: &mut NewVaccineType, _id: Option<i64>) -> Result<()> {
    super::required("name", &input.name)?;
    if let Some(species) = &input.species {
        species
            .parse::<crate::model::Species>()
            .map_err(|_| super::invalid("species", "unknown species"))?;
    }
    super::positive_days("defaultIntervalDays", input.default_interval_days)?;
    super::positive_days("primaryDoses", input.primary_doses)?;
    super::positive_days("primaryIntervalDays", input.primary_interval_days)?;
    if input.min_age_weeks.is_some_and(|v| v < 0) {
        return Err(super::invalid("minAgeWeeks", "must not be negative"));
    }
    Ok(())
}
/// The marker is recorded even if a preexisting catalog prevents initial seeding.
/// Deleting all catalog entries later must never reintroduce defaults.
pub fn seed(conn: &Connection) -> Result<()> {
    let tx = transaction(conn)?;
    let seeded: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM settings WHERE key = 'seeded_vaccine_types' AND value = '1')",
        [],
        |r| r.get(0),
    )?;
    if !seeded {
        let empty: bool =
            tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM vaccine_types)", [], |r| {
                r.get(0)
            })?;
        if empty {
            for (name, interval, species) in [
                ("Karma aşı", 365, Some("dog")),
                ("Kuduz aşısı", 365, None),
                ("İç parazit", 90, None),
                ("Dış parazit", 30, None),
                ("Kedi karma aşısı", 365, Some("cat")),
                ("Lösemi aşısı (FeLV)", 365, Some("cat")),
            ] {
                insert(
                    &tx,
                    NewVaccineType {
                        name: name.into(),
                        default_interval_days: Some(interval),
                        species: species.map(str::to_string),
                        ..Default::default()
                    },
                )?;
            }
        }
        tx.execute("INSERT INTO settings(key, value, updated_at) VALUES ('seeded_vaccine_types', '1', ?1) ON CONFLICT(key) DO UPDATE SET value = '1', updated_at = excluded.updated_at", [now()])?;
    }
    tx.commit()?;
    Ok(())
}
pub fn list(conn: &Connection) -> Result<Vec<VaccineType>> {
    seed(conn)?;
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM vaccine_types WHERE deleted_at IS NULL ORDER BY is_active DESC, name, id"))?;
    let rows = stmt
        .query_map([], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
