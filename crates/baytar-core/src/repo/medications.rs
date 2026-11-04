use super::{now, transaction, RepoError, Result};
use crate::model::Medication;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMedication {
    pub name: String,
    #[serde(default)]
    pub active_ingredient: Option<String>,
    #[serde(default)]
    pub form: Option<String>,
    #[serde(default)]
    pub concentration_mg_per_ml: Option<f64>,
    #[serde(default)]
    pub strength_mg_per_unit: Option<f64>,
    #[serde(default)]
    pub dose_mg_per_kg_min: Option<f64>,
    #[serde(default)]
    pub dose_mg_per_kg_max: Option<f64>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub target_species: Option<String>,
    #[serde(default)]
    pub prescription_required: Option<bool>,
    #[serde(default)]
    pub pack_unit: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub active_ingredient: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub form: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub concentration_mg_per_ml: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub strength_mg_per_unit: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub dose_mg_per_kg_min: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub dose_mg_per_kg_max: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub route: Option<Option<String>>,
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
    pub category: Option<Option<String>>,
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
    pub target_species: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub prescription_required: Option<Option<bool>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub pack_unit: Option<Option<String>>,
}
const COLUMNS: &str = "id, name, active_ingredient, form, concentration_mg_per_ml, strength_mg_per_unit, dose_mg_per_kg_min, dose_mg_per_kg_max, route, notes, created_at, updated_at, deleted_at, is_builtin, category, manufacturer, target_species, prescription_required, pack_unit";
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Medication> {
    Ok(Medication {
        id: r.get(0)?,
        name: r.get(1)?,
        active_ingredient: r.get(2)?,
        form: r
            .get::<_, Option<String>>(3)?
            .map(|v| v.parse())
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        concentration_mg_per_ml: r.get(4)?,
        strength_mg_per_unit: r.get(5)?,
        dose_mg_per_kg_min: r.get(6)?,
        dose_mg_per_kg_max: r.get(7)?,
        route: r.get(8)?,
        notes: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
        deleted_at: r.get(12)?,
        is_builtin: r.get(13)?,
        category: r
            .get::<_, Option<String>>(14)?
            .map(|v| v.parse())
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    14,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        manufacturer: r.get(15)?,
        target_species: r.get(16)?,
        prescription_required: r.get(17)?,
        pack_unit: r.get(18)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Medication> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM medications WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewMedication) -> Result<Medication> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewMedication) -> Result<Medication> {
    conn.execute("INSERT INTO medications (name, active_ingredient, form, concentration_mg_per_ml, strength_mg_per_unit, dose_mg_per_kg_min, dose_mg_per_kg_max, route, notes, is_builtin, category, manufacturer, target_species, prescription_required, pack_unit, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)", params![input.name, input.active_ingredient, input.form, input.concentration_mg_per_ml, input.strength_mg_per_unit, input.dose_mg_per_kg_min, input.dose_mg_per_kg_max, input.route, input.notes, input.is_builtin, input.category, input.manufacturer, input.target_species, input.prescription_required, input.pack_unit, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: MedicationPatch) -> Result<Medication> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewMedication {
        name: patch.name.unwrap_or(old.name),
        active_ingredient: patch.active_ingredient.unwrap_or(old.active_ingredient),
        form: patch
            .form
            .unwrap_or(old.form.map(|v| v.as_str().to_string())),
        concentration_mg_per_ml: patch
            .concentration_mg_per_ml
            .unwrap_or(old.concentration_mg_per_ml),
        strength_mg_per_unit: patch
            .strength_mg_per_unit
            .unwrap_or(old.strength_mg_per_unit),
        dose_mg_per_kg_min: patch.dose_mg_per_kg_min.unwrap_or(old.dose_mg_per_kg_min),
        dose_mg_per_kg_max: patch.dose_mg_per_kg_max.unwrap_or(old.dose_mg_per_kg_max),
        route: patch.route.unwrap_or(old.route),
        notes: patch.notes.unwrap_or(old.notes),
        is_builtin: old.is_builtin,
        category: patch
            .category
            .unwrap_or(old.category.map(|v| v.as_str().to_string())),
        manufacturer: patch.manufacturer.unwrap_or(old.manufacturer),
        target_species: patch.target_species.unwrap_or(old.target_species),
        prescription_required: patch
            .prescription_required
            .unwrap_or(old.prescription_required),
        pack_unit: patch.pack_unit.unwrap_or(old.pack_unit),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE medications SET name = ?1, active_ingredient = ?2, form = ?3, concentration_mg_per_ml = ?4, strength_mg_per_unit = ?5, dose_mg_per_kg_min = ?6, dose_mg_per_kg_max = ?7, route = ?8, notes = ?9, category = ?10, manufacturer = ?11, target_species = ?12, prescription_required = ?13, pack_unit = ?14, updated_at = ?15 WHERE id = ?16", params![input.name, input.active_ingredient, input.form, input.concentration_mg_per_ml, input.strength_mg_per_unit, input.dose_mg_per_kg_min, input.dose_mg_per_kg_max, input.route, input.notes, input.category, input.manufacturer, input.target_species, input.prescription_required, input.pack_unit, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Medication> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Medication> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Medication> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE medications SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(_conn: &Connection, input: &mut NewMedication, _id: Option<i64>) -> Result<()> {
    super::required("name", &input.name)?;
    if let Some(form) = &input.form {
        form.parse::<crate::model::MedicationForm>()
            .map_err(|_| super::invalid("form", "unknown medication form"))?;
    }
    if let Some(category) = &input.category {
        category
            .parse::<crate::model::MedicationCategory>()
            .map_err(|_| super::invalid("category", "unknown medication category"))?;
    }
    for (field, value) in [
        ("concentrationMgPerMl", input.concentration_mg_per_ml),
        ("strengthMgPerUnit", input.strength_mg_per_unit),
        ("doseMgPerKgMin", input.dose_mg_per_kg_min),
        ("doseMgPerKgMax", input.dose_mg_per_kg_max),
    ] {
        super::positive(field, value)?;
    }
    if let (Some(min), Some(max)) = (input.dose_mg_per_kg_min, input.dose_mg_per_kg_max) {
        if max < min {
            return Err(super::invalid(
                "doseMgPerKgMax",
                "must be at least doseMgPerKgMin",
            ));
        }
    }
    Ok(())
}
pub fn list(conn: &Connection) -> Result<Vec<Medication>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM medications WHERE deleted_at IS NULL"
    ))?;
    let mut rows = stmt
        .query_map([], row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    rows.sort_by_key(|m| (crate::text::fold(&m.name), m.id));
    Ok(rows)
}
pub fn search(conn: &Connection, query: &str) -> Result<Vec<Medication>> {
    let query = crate::text::fold(query);
    Ok(list(conn)?
        .into_iter()
        .filter(|m| {
            crate::text::fold(&m.name).contains(&query)
                || crate::text::fold(m.active_ingredient.as_deref().unwrap_or_default())
                    .contains(&query)
        })
        .collect())
}
