use super::{now, transaction, RepoError, Result};
use crate::model::Appointment;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAppointment {
    #[serde(default)]
    pub patient_id: Option<i64>,
    #[serde(default)]
    pub owner_id: Option<i64>,

    pub title: String,

    pub starts_on: String,
    #[serde(default)]
    pub starts_time: Option<String>,
    #[serde(default)]
    pub duration_min: Option<i64>,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub series_id: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub vaccine_type_id: Option<i64>,
    #[serde(default)]
    pub vaccination_id: Option<i64>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentPatch {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub patient_id: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub owner_id: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_on: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub starts_time: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub duration_min: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub series_id: Option<Option<String>>,
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
    pub vaccine_type_id: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::nullable"
    )]
    pub vaccination_id: Option<Option<i64>>,
}
fn default_kind() -> String {
    "other".into()
}
fn default_status() -> String {
    "planned".into()
}
impl Default for NewAppointment {
    fn default() -> Self {
        Self {
            patient_id: Default::default(),
            owner_id: Default::default(),
            title: Default::default(),
            starts_on: Default::default(),
            starts_time: Default::default(),
            duration_min: Default::default(),
            kind: "other".into(),
            status: "planned".into(),
            series_id: Default::default(),
            notes: Default::default(),
            vaccine_type_id: Default::default(),
            vaccination_id: Default::default(),
        }
    }
}
const COLUMNS: &str = "id, patient_id, owner_id, title, starts_on, starts_time, duration_min, kind, status, series_id, notes, created_at, updated_at, deleted_at, vaccine_type_id, vaccination_id";
pub(crate) fn row(r: &Row<'_>) -> rusqlite::Result<Appointment> {
    Ok(Appointment {
        id: r.get(0)?,
        patient_id: r.get(1)?,
        owner_id: r.get(2)?,
        title: r.get(3)?,
        starts_on: r.get(4)?,
        starts_time: r.get(5)?,
        duration_min: r.get(6)?,
        kind: r.get::<_, String>(7)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
        })?,
        status: r.get::<_, String>(8)?.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
        })?,
        series_id: r.get(9)?,
        notes: r.get(10)?,
        created_at: r.get(11)?,
        updated_at: r.get(12)?,
        deleted_at: r.get(13)?,
        vaccine_type_id: r.get(14)?,
        vaccination_id: r.get(15)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<Appointment> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM appointments WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn create(conn: &Connection, mut input: NewAppointment) -> Result<Appointment> {
    let tx = transaction(conn)?;
    prepare(&tx, &mut input, None)?;
    let result = insert(&tx, input)?;
    tx.commit()?;
    Ok(result)
}
fn insert(conn: &Connection, input: NewAppointment) -> Result<Appointment> {
    conn.execute("INSERT INTO appointments (patient_id, owner_id, title, starts_on, starts_time, duration_min, kind, status, series_id, notes, vaccine_type_id, vaccination_id, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", params![input.patient_id, input.owner_id, input.title, input.starts_on, input.starts_time, input.duration_min, input.kind, input.status, input.series_id, input.notes, input.vaccine_type_id, input.vaccination_id, now()])?;
    get(conn, conn.last_insert_rowid())
}
pub fn update(conn: &Connection, id: i64, patch: AppointmentPatch) -> Result<Appointment> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let mut input = NewAppointment {
        patient_id: patch.patient_id.unwrap_or(old.patient_id),
        owner_id: patch.owner_id.unwrap_or(old.owner_id),
        title: patch.title.unwrap_or(old.title),
        starts_on: patch.starts_on.unwrap_or(old.starts_on),
        starts_time: patch.starts_time.unwrap_or(old.starts_time),
        duration_min: patch.duration_min.unwrap_or(old.duration_min),
        kind: patch.kind.unwrap_or(old.kind.as_str().to_string()),
        status: patch.status.unwrap_or(old.status.as_str().to_string()),
        series_id: patch.series_id.unwrap_or(old.series_id),
        notes: patch.notes.unwrap_or(old.notes),
        vaccine_type_id: patch.vaccine_type_id.unwrap_or(old.vaccine_type_id),
        vaccination_id: patch.vaccination_id.unwrap_or(old.vaccination_id),
    };
    prepare(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE appointments SET patient_id = ?1, owner_id = ?2, title = ?3, starts_on = ?4, starts_time = ?5, duration_min = ?6, kind = ?7, status = ?8, series_id = ?9, notes = ?10, vaccine_type_id = ?11, vaccination_id = ?12, updated_at = ?13 WHERE id = ?14", params![input.patient_id, input.owner_id, input.title, input.starts_on, input.starts_time, input.duration_min, input.kind, input.status, input.series_id, input.notes, input.vaccine_type_id, input.vaccination_id, now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Appointment> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Appointment> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Appointment> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE appointments SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn prepare(conn: &Connection, input: &mut NewAppointment, _id: Option<i64>) -> Result<()> {
    super::required("title", &input.title)?;
    super::date("startsOn", &input.starts_on)?;
    super::time("startsTime", input.starts_time.as_deref())?;
    super::positive_days("durationMin", input.duration_min)?;
    input
        .kind
        .parse::<crate::model::AppointmentKind>()
        .map_err(|_| super::invalid("kind", "unknown appointment kind"))?;
    input
        .status
        .parse::<crate::model::AppointmentStatus>()
        .map_err(|_| super::invalid("status", "unknown appointment status"))?;
    if let Some(id) = input.patient_id {
        let patient = super::active_patient(conn, id)?;
        if input.owner_id.is_some_and(|id| id != patient.owner_id) {
            return Err(super::invalid(
                "ownerId",
                "owner must match patient's owner",
            ));
        }
    }
    if let Some(id) = input.owner_id {
        if super::owners::get(conn, id)?.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    if let Some(id) = input.vaccine_type_id {
        if super::vaccine_types::get(conn, id)?.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    if let Some(id) = input.vaccination_id {
        let vaccination = super::vaccinations::get(conn, id)?;
        if input
            .patient_id
            .is_some_and(|patient| patient != vaccination.patient_id)
        {
            return Err(super::invalid(
                "vaccinationId",
                "vaccination must belong to the appointment's patient",
            ));
        }
    }
    Ok(())
}
pub fn set_status(conn: &Connection, id: i64, status: &str) -> Result<Appointment> {
    update(
        conn,
        id,
        AppointmentPatch {
            status: Some(status.into()),
            ..Default::default()
        },
    )
}
pub fn create_series(
    conn: &Connection,
    mut base: NewAppointment,
    count: u32,
    every_days: i64,
) -> Result<Vec<Appointment>> {
    if !(1..=52).contains(&count) {
        return Err(super::invalid("count", "must be between 1 and 52"));
    }
    super::positive_days("everyDays", Some(every_days))?;
    let tx = transaction(conn)?;
    prepare(&tx, &mut base, None)?;
    let start = super::date("startsOn", &base.starts_on)?;
    let series: String = tx.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))?;
    base.series_id = Some(series);
    let mut result = Vec::new();
    for index in 0..count {
        let days = every_days
            .checked_mul(i64::from(index))
            .and_then(chrono::Duration::try_days)
            .ok_or_else(|| super::invalid("everyDays", "date overflow"))?;
        let date = start
            .checked_add_signed(days)
            .ok_or_else(|| super::invalid("everyDays", "date overflow"))?
            .format("%Y-%m-%d")
            .to_string();
        super::date("startsOn", &date)?;
        let mut input = base.clone();
        input.starts_on = date;
        result.push(insert(&tx, input)?);
    }
    tx.commit()?;
    Ok(result)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAppointment {
    pub appointment: Appointment,
    pub patient_name: Option<String>,
    pub owner_name: Option<String>,
}
pub fn between(conn: &Connection, from: &str, to: &str) -> Result<Vec<CalendarAppointment>> {
    super::range(from, to)?;
    let columns = COLUMNS
        .split(", ")
        .map(|c| format!("a.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn.prepare(&format!("SELECT {columns}, p.name, o.full_name FROM appointments a LEFT JOIN patients p ON p.id = a.patient_id LEFT JOIN owners o ON o.id = coalesce(a.owner_id, p.owner_id) WHERE a.deleted_at IS NULL AND a.starts_on BETWEEN ?1 AND ?2 ORDER BY a.starts_on, a.starts_time, a.id"))?;
    let rows = stmt
        .query_map(params![from, to], |r| {
            Ok(CalendarAppointment {
                appointment: row(r)?,
                patient_name: r.get(16)?,
                owner_name: r.get(17)?,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}
pub fn today(conn: &Connection, today: &str) -> Result<Vec<CalendarAppointment>> {
    super::date("today", today)?;
    between(conn, today, today)
}
