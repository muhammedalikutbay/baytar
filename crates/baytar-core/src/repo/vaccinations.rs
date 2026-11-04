use super::{now, transaction, RepoError, Result};
use crate::model::Vaccination;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVaccination {
    pub patient_id: i64,
    #[serde(default)]
    pub vaccine_type_id: Option<i64>,

    pub vaccine_name: String,

    pub given_on: String,
    #[serde(default)]
    pub next_due_on: Option<String>,
    #[serde(default)]
    pub batch_no: Option<String>,
    #[serde(default)]
    pub visit_id: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccinationPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub vaccine_type_id: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vaccine_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given_on: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub next_due_on: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub batch_no: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub visit_id: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub notes: Option<Option<String>>,
}
const COLUMNS: &str = "id, patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on, batch_no, visit_id, notes, created_at, updated_at, deleted_at";
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Vaccination> {
    Ok(Vaccination {
        id: r.get(0)?,
        patient_id: r.get(1)?,
        vaccine_type_id: r.get(2)?,
        vaccine_name: r.get(3)?,
        given_on: r.get(4)?,
        next_due_on: r.get(5)?,
        batch_no: r.get(6)?,
        visit_id: r.get(7)?,
        notes: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
        deleted_at: r.get(11)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Vaccination> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM vaccinations WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewVaccination) -> Result<Vaccination> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewVaccination) -> Result<Vaccination> {
    conn.execute("INSERT INTO vaccinations (patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on, batch_no, visit_id, notes, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![input.patient_id, input.vaccine_type_id, input.vaccine_name, input.given_on, input.next_due_on, input.batch_no, input.visit_id, input.notes, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: VaccinationPatch) -> Result<Vaccination> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewVaccination {
        patient_id: patch.patient_id.unwrap_or(old.patient_id),
        vaccine_type_id: patch.vaccine_type_id.unwrap_or(old.vaccine_type_id),
        vaccine_name: patch.vaccine_name.unwrap_or(old.vaccine_name),
        given_on: patch.given_on.unwrap_or(old.given_on),
        next_due_on: patch.next_due_on.unwrap_or(old.next_due_on),
        batch_no: patch.batch_no.unwrap_or(old.batch_no),
        visit_id: patch.visit_id.unwrap_or(old.visit_id),
        notes: patch.notes.unwrap_or(old.notes),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE vaccinations SET patient_id = ?1, vaccine_type_id = ?2, vaccine_name = ?3, given_on = ?4, next_due_on = ?5, batch_no = ?6, visit_id = ?7, notes = ?8, updated_at = ?9 WHERE id = ?10", params![input.patient_id, input.vaccine_type_id, input.vaccine_name, input.given_on, input.next_due_on, input.batch_no, input.visit_id, input.notes, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Vaccination> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Vaccination> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Vaccination> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE vaccinations SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(conn: &Connection, input: &mut NewVaccination, _id: Option<i64>) -> Result<()> {
    super::required("vaccineName", &input.vaccine_name)?;
    super::date("givenOn", &input.given_on)?;
    if let Some(due) = &input.next_due_on {
        super::date("nextDueOn", due)?;
    }
    super::active_patient(conn, input.patient_id)?;
    if let Some(id) = input.vaccine_type_id {
        super::vaccine_types::get(conn, id)?;
    }
    if let Some(id) = input.visit_id {
        let visit = super::visits::get(conn, id)?;
        if visit.patient_id != input.patient_id {
            return Err(super::invalid("visitId", "visit must belong to patient"));
        }
        if visit.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    Ok(())
}
pub fn suggest_next_due(conn: &Connection, type_id: i64, given_on: &str) -> Result<Option<String>> {
    let date = super::date("givenOn", given_on)?;
    let vaccine = super::vaccine_types::get(conn, type_id)?;
    vaccine
        .default_interval_days
        .map(|days| {
            let delta = chrono::Duration::try_days(days)
                .ok_or_else(|| super::invalid("defaultIntervalDays", "date overflow"))?;
            let next = date
                .checked_add_signed(delta)
                .ok_or_else(|| super::invalid("defaultIntervalDays", "date overflow"))?
                .format("%Y-%m-%d")
                .to_string();
            super::date("nextDueOn", &next)?;
            Ok(next)
        })
        .transpose()
}
pub fn list_by_patient(conn: &Connection, patient_id: i64) -> Result<Vec<Vaccination>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM vaccinations WHERE patient_id = ?1 AND deleted_at IS NULL ORDER BY given_on DESC, id DESC"))?;
    let rows = stmt
        .query_map([patient_id], row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DueVaccination {
    pub vaccination: Vaccination,
    pub patient_name: String,
    pub owner_name: String,
}
// Equal-day administrations are ordered by id, as in the patient's history.
const UNSUPERSEDED: &str = "NOT EXISTS (SELECT 1 FROM vaccinations later WHERE later.patient_id = v.patient_id AND later.deleted_at IS NULL AND ((v.vaccine_type_id IS NOT NULL AND later.vaccine_type_id = v.vaccine_type_id) OR lower(later.vaccine_name) = lower(v.vaccine_name)) AND (later.given_on > v.given_on OR (later.given_on = v.given_on AND later.id > v.id)))";
fn due(
    conn: &Connection,
    from: Option<&str>,
    to: &str,
    overdue: bool,
) -> Result<Vec<DueVaccination>> {
    let columns = COLUMNS
        .split(", ")
        .map(|c| format!("v.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn.prepare(&format!("SELECT {columns}, p.name, o.full_name FROM vaccinations v JOIN patients p ON p.id = v.patient_id JOIN owners o ON o.id = p.owner_id WHERE v.deleted_at IS NULL AND p.deleted_at IS NULL AND p.deceased_on IS NULL AND o.deleted_at IS NULL AND (?1 IS NULL OR v.next_due_on >= ?1) AND ((?3 AND v.next_due_on < ?2) OR (NOT ?3 AND v.next_due_on <= ?2)) AND {UNSUPERSEDED} ORDER BY v.next_due_on, v.id"))?;
    let rows = stmt
        .query_map(params![from, to, overdue], |r| {
            Ok(DueVaccination {
                vaccination: row(r)?,
                patient_name: r.get(12)?,
                owner_name: r.get(13)?,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
pub fn due_between(conn: &Connection, from: &str, to: &str) -> Result<Vec<DueVaccination>> {
    super::range(from, to)?;
    due(conn, Some(from), to, false)
}
pub fn overdue(conn: &Connection, today: &str) -> Result<Vec<DueVaccination>> {
    super::date("today", today)?;
    due(conn, None, today, true)
}
