use super::{now, transaction, RepoError, Result};
use crate::model::PrescriptionItem;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPrescription {
    pub visit_id: i64,
    #[serde(default)]
    pub medication_id: Option<i64>,

    #[serde(default)]
    pub medication_name: String,
    #[serde(default)]
    pub dose_mg: Option<f64>,
    #[serde(default)]
    pub dose_ml: Option<f64>,
    #[serde(default)]
    pub dose_text: Option<String>,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default)]
    pub duration_days: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visit_id: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub medication_id: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medication_name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub dose_mg: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub dose_ml: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub dose_text: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub frequency: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub duration_days: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub notes: Option<Option<String>>,
}
const COLUMNS: &str = "id, visit_id, medication_id, medication_name, dose_mg, dose_ml, dose_text, frequency, duration_days, notes, created_at, updated_at, deleted_at";
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<PrescriptionItem> {
    Ok(PrescriptionItem {
        id: r.get(0)?,
        visit_id: r.get(1)?,
        medication_id: r.get(2)?,
        medication_name: r.get(3)?,
        dose_mg: r.get(4)?,
        dose_ml: r.get(5)?,
        dose_text: r.get(6)?,
        frequency: r.get(7)?,
        duration_days: r.get(8)?,
        notes: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
        deleted_at: r.get(12)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<PrescriptionItem> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM prescription_items WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewPrescription) -> Result<PrescriptionItem> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewPrescription) -> Result<PrescriptionItem> {
    conn.execute("INSERT INTO prescription_items (visit_id, medication_id, medication_name, dose_mg, dose_ml, dose_text, frequency, duration_days, notes, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", params![input.visit_id, input.medication_id, input.medication_name, input.dose_mg, input.dose_ml, input.dose_text, input.frequency, input.duration_days, input.notes, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: PrescriptionPatch) -> Result<PrescriptionItem> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewPrescription {
        visit_id: patch.visit_id.unwrap_or(old.visit_id),
        medication_id: patch.medication_id.unwrap_or(old.medication_id),
        medication_name: patch.medication_name.unwrap_or(old.medication_name),
        dose_mg: patch.dose_mg.unwrap_or(old.dose_mg),
        dose_ml: patch.dose_ml.unwrap_or(old.dose_ml),
        dose_text: patch.dose_text.unwrap_or(old.dose_text),
        frequency: patch.frequency.unwrap_or(old.frequency),
        duration_days: patch.duration_days.unwrap_or(old.duration_days),
        notes: patch.notes.unwrap_or(old.notes),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE prescription_items SET visit_id = ?1, medication_id = ?2, medication_name = ?3, dose_mg = ?4, dose_ml = ?5, dose_text = ?6, frequency = ?7, duration_days = ?8, notes = ?9, updated_at = ?10 WHERE id = ?11", params![input.visit_id, input.medication_id, input.medication_name, input.dose_mg, input.dose_ml, input.dose_text, input.frequency, input.duration_days, input.notes, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<PrescriptionItem> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<PrescriptionItem> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<PrescriptionItem> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE prescription_items SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(conn: &Connection, input: &mut NewPrescription, id: Option<i64>) -> Result<()> {
    let visit = super::visits::get(conn, input.visit_id)?;
    if visit.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    super::active_patient(conn, visit.patient_id)?;
    if let Some(medication_id) = input.medication_id {
        // Preserve the historical snapshot unless selecting a different medication.
        let changed = match id {
            Some(id) => get(conn, id)?.medication_id != Some(medication_id),
            None => true,
        };
        if changed {
            let medication = super::medications::get(conn, medication_id)?;
            if medication.deleted_at.is_some() {
                return Err(RepoError::NotFound);
            }
            input.medication_name = medication.name;
        } else if let Some(id) = id {
            input.medication_name = get(conn, id)?.medication_name;
        }
    }
    super::required("medicationName", &input.medication_name)?;
    super::positive("doseMg", input.dose_mg)?;
    super::positive("doseMl", input.dose_ml)?;
    super::positive_days("durationDays", input.duration_days)?;
    Ok(())
}
pub fn list_by_visit(conn: &Connection, visit_id: i64) -> Result<Vec<PrescriptionItem>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM prescription_items WHERE visit_id = ?1 AND deleted_at IS NULL ORDER BY id"))?;
    let rows = stmt
        .query_map([visit_id], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
