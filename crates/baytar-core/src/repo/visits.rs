use super::{now, transaction, RepoError, Result};
use crate::model::Visit;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVisit {
    pub patient_id: i64,

    pub visit_on: String,
    #[serde(default)]
    pub visit_time: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub findings: Option<String>,
    #[serde(default)]
    pub diagnosis: Option<String>,
    #[serde(default)]
    pub treatment: Option<String>,
    #[serde(default)]
    pub weight_kg: Option<f64>,
    #[serde(default)]
    pub temperature_c: Option<f64>,
    #[serde(default)]
    pub heart_rate_bpm: Option<i64>,
    #[serde(default)]
    pub resp_rate_bpm: Option<i64>,
    #[serde(default)]
    pub body_condition: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visit_on: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub visit_time: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub reason: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub findings: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub diagnosis: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub treatment: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub weight_kg: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub temperature_c: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub heart_rate_bpm: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub resp_rate_bpm: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub body_condition: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub notes: Option<Option<String>>,
}
const COLUMNS: &str = "id, patient_id, visit_on, visit_time, reason, findings, diagnosis, treatment, weight_kg, temperature_c, notes, created_at, updated_at, deleted_at, heart_rate_bpm, resp_rate_bpm, body_condition";
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Visit> {
    Ok(Visit {
        id: r.get(0)?,
        patient_id: r.get(1)?,
        visit_on: r.get(2)?,
        visit_time: r.get(3)?,
        reason: r.get(4)?,
        findings: r.get(5)?,
        diagnosis: r.get(6)?,
        treatment: r.get(7)?,
        weight_kg: r.get(8)?,
        temperature_c: r.get(9)?,
        notes: r.get(10)?,
        created_at: r.get(11)?,
        updated_at: r.get(12)?,
        deleted_at: r.get(13)?,
        heart_rate_bpm: r.get(14)?,
        resp_rate_bpm: r.get(15)?,
        body_condition: r.get(16)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Visit> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM visits WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewVisit) -> Result<Visit> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewVisit) -> Result<Visit> {
    conn.execute("INSERT INTO visits (patient_id, visit_on, visit_time, reason, findings, diagnosis, treatment, weight_kg, temperature_c, heart_rate_bpm, resp_rate_bpm, body_condition, notes, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)", params![input.patient_id, input.visit_on, input.visit_time, input.reason, input.findings, input.diagnosis, input.treatment, input.weight_kg, input.temperature_c, input.heart_rate_bpm, input.resp_rate_bpm, input.body_condition, input.notes, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: VisitPatch) -> Result<Visit> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewVisit {
        patient_id: patch.patient_id.unwrap_or(old.patient_id),
        visit_on: patch.visit_on.unwrap_or(old.visit_on),
        visit_time: patch.visit_time.unwrap_or(old.visit_time),
        reason: patch.reason.unwrap_or(old.reason),
        findings: patch.findings.unwrap_or(old.findings),
        diagnosis: patch.diagnosis.unwrap_or(old.diagnosis),
        treatment: patch.treatment.unwrap_or(old.treatment),
        weight_kg: patch.weight_kg.unwrap_or(old.weight_kg),
        temperature_c: patch.temperature_c.unwrap_or(old.temperature_c),
        heart_rate_bpm: patch.heart_rate_bpm.unwrap_or(old.heart_rate_bpm),
        resp_rate_bpm: patch.resp_rate_bpm.unwrap_or(old.resp_rate_bpm),
        body_condition: patch.body_condition.unwrap_or(old.body_condition),
        notes: patch.notes.unwrap_or(old.notes),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE visits SET patient_id = ?1, visit_on = ?2, visit_time = ?3, reason = ?4, findings = ?5, diagnosis = ?6, treatment = ?7, weight_kg = ?8, temperature_c = ?9, heart_rate_bpm = ?10, resp_rate_bpm = ?11, body_condition = ?12, notes = ?13, updated_at = ?14 WHERE id = ?15", params![input.patient_id, input.visit_on, input.visit_time, input.reason, input.findings, input.diagnosis, input.treatment, input.weight_kg, input.temperature_c, input.heart_rate_bpm, input.resp_rate_bpm, input.body_condition, input.notes, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Visit> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Visit> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Visit> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE visits SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(conn: &Connection, input: &mut NewVisit, id: Option<i64>) -> Result<()> {
    super::date("visitOn", &input.visit_on)?;
    super::time("visitTime", input.visit_time.as_deref())?;
    super::positive("weightKg", input.weight_kg)?;
    if input.temperature_c.is_some_and(|v| !v.is_finite()) {
        return Err(super::invalid("temperatureC", "must be finite"));
    }
    super::positive_days("heartRateBpm", input.heart_rate_bpm)?;
    super::positive_days("respRateBpm", input.resp_rate_bpm)?;
    if input.body_condition.is_some_and(|v| !(1..=9).contains(&v)) {
        return Err(super::invalid("bodyCondition", "must be between 1 and 9"));
    }
    super::active_patient(conn, input.patient_id)?;
    if let Some(id) = id {
        let conflicts: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM vaccinations WHERE visit_id = ?1 AND patient_id != ?2)",
            params![id, input.patient_id],
            |r| r.get(0),
        )?;
        if conflicts {
            return Err(super::invalid(
                "patientId",
                "must match linked vaccinations",
            ));
        }
    }
    Ok(())
}
pub fn list_by_patient(conn: &Connection, patient_id: i64) -> Result<Vec<Visit>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM visits WHERE patient_id = ?1 AND deleted_at IS NULL ORDER BY visit_on DESC, visit_time DESC, id DESC"))?;
    let rows = stmt
        .query_map([patient_id], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestWeight {
    pub weight_kg: f64,
    pub visit_on: String,
}
pub fn latest_weight(conn: &Connection, patient_id: i64) -> Result<Option<LatestWeight>> {
    Ok(conn.query_row("SELECT weight_kg, visit_on FROM visits WHERE patient_id = ?1 AND deleted_at IS NULL AND weight_kg IS NOT NULL ORDER BY visit_on DESC, visit_time DESC, id DESC LIMIT 1", [patient_id], |r| Ok(LatestWeight { weight_kg: r.get(0)?, visit_on: r.get(1)? })).optional()?)
}
