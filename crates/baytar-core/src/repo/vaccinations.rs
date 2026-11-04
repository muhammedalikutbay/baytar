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

/// One actionable reminder per patient/vaccine; plans take precedence over inferred due dates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccineDue {
    pub patient_id: i64,
    pub patient_name: String,
    pub species: crate::model::Species,
    pub owner_name: String,
    pub phone: Option<String>,
    pub vaccine_type_id: Option<i64>,
    pub vaccine_name: String,
    pub due_on: String,
    pub appointment_id: Option<i64>,
    pub vaccination_id: Option<i64>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VaccineDueQuery {
    pub patient_id: Option<i64>,
    pub species: Option<String>,
    pub vaccine_type_id: Option<i64>,
    pub from: Option<String>,
    pub to: Option<String>,
}
pub fn merged_due(conn: &Connection, query: VaccineDueQuery) -> Result<Vec<VaccineDue>> {
    if let Some(d) = &query.from {
        super::date("from", d)?;
    }
    if let Some(d) = &query.to {
        super::date("to", d)?;
    }
    if let (Some(from), Some(to)) = (&query.from, &query.to) {
        super::range(from, to)?;
    }
    if let Some(s) = &query.species {
        s.parse::<crate::model::Species>()
            .map_err(|_| super::invalid("species", "unknown species"))?;
    }
    // Rank before date filtering: an out-of-range plan must still suppress an older due date.
    let sql = format!(
        r#"
        WITH candidates AS (
          SELECT a.patient_id, a.vaccine_type_id, coalesce(t.name, a.title) vaccine_name,
            a.starts_on due_on, a.id appointment_id, NULL vaccination_id,
            'type:' || a.vaccine_type_id vaccine_key, 0 priority
          FROM appointments a LEFT JOIN vaccine_types t ON t.id = a.vaccine_type_id
          WHERE a.kind = 'vaccine' AND a.status = 'planned' AND a.deleted_at IS NULL
            AND a.patient_id IS NOT NULL AND a.vaccine_type_id IS NOT NULL
          UNION ALL
          SELECT v.patient_id, v.vaccine_type_id, v.vaccine_name, v.next_due_on, NULL, v.id,
            CASE WHEN v.vaccine_type_id IS NOT NULL THEN 'type:' || v.vaccine_type_id ELSE 'name:' || lower(v.vaccine_name) END, 1
          FROM vaccinations v WHERE v.deleted_at IS NULL AND v.next_due_on IS NOT NULL AND {UNSUPERSEDED}
        ), ranked AS (
          SELECT *, row_number() OVER (PARTITION BY patient_id, vaccine_key ORDER BY priority, due_on, appointment_id) rank
          FROM candidates
        )
        SELECT r.patient_id, p.name, p.species, o.full_name, coalesce(o.phone, o.phone_raw),
          r.vaccine_type_id, r.vaccine_name, r.due_on, r.appointment_id, r.vaccination_id
        FROM ranked r JOIN patients p ON p.id = r.patient_id JOIN owners o ON o.id = p.owner_id
        WHERE rank = 1 AND p.deleted_at IS NULL AND p.deceased_on IS NULL AND o.deleted_at IS NULL
          AND (?1 IS NULL OR r.patient_id = ?1) AND (?2 IS NULL OR p.species = ?2)
          AND (?3 IS NULL OR r.vaccine_type_id = ?3) AND (?4 IS NULL OR r.due_on >= ?4)
          AND (?5 IS NULL OR r.due_on <= ?5) ORDER BY r.due_on, p.name, r.vaccine_name
    "#
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(
            params![
                query.patient_id,
                query.species,
                query.vaccine_type_id,
                query.from,
                query.to
            ],
            |r| {
                let species: String = r.get(2)?;
                Ok(VaccineDue {
                    patient_id: r.get(0)?,
                    patient_name: r.get(1)?,
                    species: species.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    owner_name: r.get(3)?,
                    phone: r.get(4)?,
                    vaccine_type_id: r.get(5)?,
                    vaccine_name: r.get(6)?,
                    due_on: r.get(7)?,
                    appointment_id: r.get(8)?,
                    vaccination_id: r.get(9)?,
                })
            },
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The count includes the dose being entered, so the last primary dose switches to boosters.
pub fn suggest_for_patient(
    conn: &Connection,
    patient_id: i64,
    type_id: i64,
    given_on: &str,
) -> Result<Option<String>> {
    let date = super::date("givenOn", given_on)?;
    let vaccine = super::vaccine_types::available_for_patient(conn, type_id, patient_id)?;
    let count: i64 = conn.query_row("SELECT count(*) FROM vaccinations WHERE patient_id = ?1 AND vaccine_type_id = ?2 AND deleted_at IS NULL AND given_on <= ?3", params![patient_id, type_id, given_on], |r| r.get(0))?;
    let interval = if vaccine.primary_doses.is_some_and(|doses| count + 1 < doses) {
        vaccine
            .primary_interval_days
            .or(vaccine.default_interval_days)
    } else {
        vaccine.default_interval_days
    };
    interval
        .map(|days| {
            let next = chrono::Duration::try_days(days)
                .and_then(|delta| date.checked_add_signed(delta))
                .ok_or_else(|| super::invalid("givenOn", "date overflow"))?
                .format("%Y-%m-%d")
                .to_string();
            super::date("nextDueOn", &next)?;
            Ok(next)
        })
        .transpose()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GiveVaccination {
    pub vaccination: NewVaccination,
    pub appointment_id: Option<i64>,
    #[serde(default)]
    pub plan_next: bool,
}
/// The given dose, completed appointment and optional next plan commit together.
pub fn give(conn: &Connection, input: GiveVaccination) -> Result<Vaccination> {
    let tx = transaction(conn)?;
    let mut dose = input.vaccination;
    prepare(&tx, &mut dose, None)?;
    let patient = super::active_patient(&tx, dose.patient_id)?;
    if patient.deceased_on.is_some() {
        return Err(super::invalid("patientId", "patient is deceased"));
    }
    if let Some(id) = dose.vaccine_type_id {
        dose.vaccine_name =
            super::vaccine_types::available_for_patient(&tx, id, dose.patient_id)?.name;
    }
    if dose
        .next_due_on
        .as_ref()
        .is_some_and(|d| d <= &dose.given_on)
    {
        return Err(super::invalid("nextDueOn", "must be after givenOn"));
    }
    if let Some(id) = input.appointment_id {
        let appointment = super::appointments::get(&tx, id)?;
        if appointment.deleted_at.is_some()
            || appointment.status != crate::model::AppointmentStatus::Planned
            || appointment.vaccination_id.is_some()
        {
            return Err(RepoError::Conflict);
        }
        if appointment.kind != crate::model::AppointmentKind::Vaccine
            || appointment.patient_id != Some(dose.patient_id)
            || appointment.vaccine_type_id != dose.vaccine_type_id
        {
            return Err(super::invalid(
                "appointmentId",
                "appointment must match patient and vaccine",
            ));
        }
    }
    let result = insert(&tx, dose)?;
    if let Some(id) = input.appointment_id {
        tx.execute("UPDATE appointments SET status = 'done', vaccination_id = ?1, updated_at = ?2 WHERE id = ?3", params![result.id, now(), id])?;
    }
    if input.plan_next {
        if let Some(date) = &result.next_due_on {
            let type_id = result
                .vaccine_type_id
                .ok_or_else(|| super::invalid("vaccineTypeId", "required to plan next dose"))?;
            let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM appointments WHERE patient_id = ?1 AND vaccine_type_id = ?2 AND kind = 'vaccine' AND status = 'planned' AND deleted_at IS NULL)", params![result.patient_id, type_id], |r| r.get(0))?;
            if !exists {
                super::appointments::insert(
                    &tx,
                    super::appointments::NewAppointment {
                        patient_id: Some(patient.id),
                        owner_id: Some(patient.owner_id),
                        vaccine_type_id: Some(type_id),
                        title: format!("{} — {}", result.vaccine_name, patient.name),
                        starts_on: date.clone(),
                        kind: "vaccine".into(),
                        ..Default::default()
                    },
                )?;
            }
        }
    }
    tx.commit()?;
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanVaccination {
    pub patient_id: i64,
    pub vaccine_type_id: i64,
    pub starts_on: String,
    pub starts_time: Option<String>,
    pub notes: Option<String>,
}
pub fn plan(conn: &Connection, input: PlanVaccination) -> Result<crate::model::Appointment> {
    let tx = transaction(conn)?;
    let patient = super::active_patient(&tx, input.patient_id)?;
    if patient.deceased_on.is_some() {
        return Err(super::invalid("patientId", "patient is deceased"));
    }
    let vaccine =
        super::vaccine_types::available_for_patient(&tx, input.vaccine_type_id, input.patient_id)?;
    super::date("startsOn", &input.starts_on)?;
    super::time("startsTime", input.starts_time.as_deref())?;
    let appointment = super::appointments::insert(
        &tx,
        super::appointments::NewAppointment {
            patient_id: Some(patient.id),
            owner_id: Some(patient.owner_id),
            vaccine_type_id: Some(vaccine.id),
            title: format!("{} — {}", vaccine.name, patient.name),
            starts_on: input.starts_on,
            starts_time: input.starts_time,
            notes: super::clean(input.notes),
            kind: "vaccine".into(),
            ..Default::default()
        },
    )?;
    tx.commit()?;
    Ok(appointment)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccineCalendarEvent {
    pub patient_id: i64,
    pub patient_name: String,
    pub vaccine_name: String,
    pub on: String,
    pub planned: bool,
}
pub fn calendar(conn: &Connection, from: &str, to: &str) -> Result<Vec<VaccineCalendarEvent>> {
    super::range(from, to)?;
    let mut stmt = conn.prepare("SELECT v.patient_id, p.name, v.vaccine_name, v.given_on, 0 FROM vaccinations v JOIN patients p ON p.id = v.patient_id JOIN owners o ON o.id = p.owner_id WHERE v.deleted_at IS NULL AND p.deleted_at IS NULL AND o.deleted_at IS NULL AND v.given_on BETWEEN ?1 AND ?2 UNION ALL SELECT a.patient_id, p.name, coalesce(t.name, a.title), a.starts_on, 1 FROM appointments a JOIN patients p ON p.id = a.patient_id JOIN owners o ON o.id = p.owner_id LEFT JOIN vaccine_types t ON t.id = a.vaccine_type_id WHERE a.deleted_at IS NULL AND a.kind = 'vaccine' AND a.status = 'planned' AND p.deleted_at IS NULL AND p.deceased_on IS NULL AND o.deleted_at IS NULL AND a.starts_on BETWEEN ?1 AND ?2 ORDER BY 4, 2")?;
    let rows = stmt
        .query_map(params![from, to], |r| {
            Ok(VaccineCalendarEvent {
                patient_id: r.get(0)?,
                patient_name: r.get(1)?,
                vaccine_name: r.get(2)?,
                on: r.get(3)?,
                planned: r.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn postpone(
    conn: &Connection,
    appointment_id: Option<i64>,
    vaccination_id: Option<i64>,
    on: &str,
) -> Result<crate::model::Appointment> {
    super::date("startsOn", on)?;
    let tx = transaction(conn)?;
    let id = if let Some(id) = appointment_id {
        let a = super::appointments::get(&tx, id)?;
        if a.deleted_at.is_some()
            || a.kind != crate::model::AppointmentKind::Vaccine
            || a.status != crate::model::AppointmentStatus::Planned
        {
            return Err(RepoError::Conflict);
        }
        id
    } else if let Some(id) = vaccination_id {
        let dose = get(&tx, id)?;
        if dose.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
        let type_id = dose
            .vaccine_type_id
            .ok_or_else(|| super::invalid("vaccineTypeId", "select a catalog vaccine to plan"))?;
        let patient = super::active_patient(&tx, dose.patient_id)?;
        super::vaccine_types::available_for_patient(&tx, type_id, patient.id)?;
        let existing = tx.query_row("SELECT id FROM appointments WHERE patient_id = ?1 AND vaccine_type_id = ?2 AND kind = 'vaccine' AND status = 'planned' AND deleted_at IS NULL ORDER BY starts_on, id LIMIT 1", params![patient.id, type_id], |r| r.get::<_, i64>(0)).optional()?;
        if let Some(id) = existing {
            id
        } else {
            super::appointments::insert(
                &tx,
                super::appointments::NewAppointment {
                    patient_id: Some(patient.id),
                    owner_id: Some(patient.owner_id),
                    vaccine_type_id: Some(type_id),
                    title: format!("{} — {}", dose.vaccine_name, patient.name),
                    starts_on: on.into(),
                    kind: "vaccine".into(),
                    ..Default::default()
                },
            )?
            .id
        }
    } else {
        return Err(super::invalid("appointmentId", "a reminder is required"));
    };
    tx.execute(
        "UPDATE appointments SET starts_on = ?1, updated_at = ?2 WHERE id = ?3",
        params![on, now(), id],
    )?;
    let result = super::appointments::get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

#[cfg(test)]
mod schedule_tests {
    use super::*;
    use crate::{
        db::Database,
        repo::{appointments, owners, patients, vaccine_types},
    };
    fn fixture() -> (Database, i64, i64) {
        let db = Database::open_in_memory().unwrap();
        let owner = owners::create(
            db.conn(),
            owners::NewOwner {
                full_name: "Synthetic owner".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let patient = patients::create(
            db.conn(),
            patients::NewPatient {
                owner_id: owner.id,
                name: "Synthetic cat".into(),
                species: "cat".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let vaccine = vaccine_types::create(
            db.conn(),
            vaccine_types::NewVaccineType {
                name: "Synthetic vaccine".into(),
                species: Some("cat".into()),
                primary_doses: Some(2),
                primary_interval_days: Some(21),
                default_interval_days: Some(365),
                ..Default::default()
            },
        )
        .unwrap();
        (db, patient.id, vaccine.id)
    }
    fn dose(patient_id: i64, type_id: i64, on: &str, due: Option<&str>) -> NewVaccination {
        NewVaccination {
            patient_id,
            vaccine_type_id: Some(type_id),
            vaccine_name: "Synthetic vaccine".into(),
            given_on: on.into(),
            next_due_on: due.map(str::to_owned),
            ..Default::default()
        }
    }
    fn plan_input(patient_id: i64, vaccine_type_id: i64, on: &str) -> PlanVaccination {
        PlanVaccination {
            patient_id,
            vaccine_type_id,
            starts_on: on.into(),
            starts_time: None,
            notes: None,
        }
    }
    #[test]
    fn primary_series_then_booster_ignores_deleted_and_future_doses() {
        let (db, patient, vaccine) = fixture();
        let c = db.conn();
        assert_eq!(
            suggest_for_patient(c, patient, vaccine, "2026-01-01")
                .unwrap()
                .as_deref(),
            Some("2026-01-22")
        );
        let first = create(c, dose(patient, vaccine, "2026-01-01", None)).unwrap();
        assert_eq!(
            suggest_for_patient(c, patient, vaccine, "2026-01-22")
                .unwrap()
                .as_deref(),
            Some("2027-01-22")
        );
        soft_delete(c, first.id).unwrap();
        create(c, dose(patient, vaccine, "2027-01-01", None)).unwrap();
        assert_eq!(
            suggest_for_patient(c, patient, vaccine, "2026-01-22")
                .unwrap()
                .as_deref(),
            Some("2026-02-12")
        );
    }
    #[test]
    fn merge_deduplicates_before_date_filter_and_hides_superseded_doses() {
        let (db, patient, vaccine) = fixture();
        let c = db.conn();
        create(c, dose(patient, vaccine, "2025-01-01", Some("2026-01-01"))).unwrap();
        let latest = create(c, dose(patient, vaccine, "2025-02-01", Some("2026-02-01"))).unwrap();
        let rows = merged_due(
            c,
            VaccineDueQuery {
                to: Some("2026-02-02".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].vaccination_id, Some(latest.id));
        let appointment = plan(c, plan_input(patient, vaccine, "2026-03-01")).unwrap();
        assert!(merged_due(
            c,
            VaccineDueQuery {
                to: Some("2026-02-02".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .is_empty());
        let rows = merged_due(c, Default::default()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].appointment_id, Some(appointment.id));
        appointments::set_status(c, appointment.id, "cancelled").unwrap();
        assert_eq!(
            merged_due(c, Default::default()).unwrap()[0].vaccination_id,
            Some(latest.id)
        );
        create(c, dose(patient, vaccine, "2026-02-02", None)).unwrap();
        assert!(merged_due(c, Default::default()).unwrap().is_empty());
    }
    #[test]
    fn give_completes_and_links_once_and_rolls_back_every_write_on_failure() {
        let (db, patient, vaccine) = fixture();
        let c = db.conn();
        let appointment = plan(c, plan_input(patient, vaccine, "2026-02-01")).unwrap();
        let input = GiveVaccination {
            vaccination: dose(patient, vaccine, "2026-02-01", Some("2027-02-01")),
            appointment_id: Some(appointment.id),
            plan_next: true,
        };
        c.execute_batch("CREATE TRIGGER fail_next_plan BEFORE INSERT ON appointments BEGIN SELECT RAISE(ABORT, 'synthetic failure'); END;").unwrap();
        assert!(give(c, input.clone()).is_err());
        assert!(list_by_patient(c, patient).unwrap().is_empty());
        let original = appointments::get(c, appointment.id).unwrap();
        assert_eq!(original.status, crate::model::AppointmentStatus::Planned);
        assert_eq!(original.vaccination_id, None);
        c.execute_batch("DROP TRIGGER fail_next_plan").unwrap();
        let vaccination = give(c, input.clone()).unwrap();
        let completed = appointments::get(c, appointment.id).unwrap();
        assert_eq!(completed.status, crate::model::AppointmentStatus::Done);
        assert_eq!(completed.vaccination_id, Some(vaccination.id));
        assert_eq!(
            merged_due(c, Default::default()).unwrap()[0].due_on,
            "2027-02-01"
        );
        assert!(give(c, input).is_err());
        assert_eq!(list_by_patient(c, patient).unwrap().len(), 1);
        let events = calendar(c, "2026-01-01", "2027-12-31").unwrap();
        assert_eq!(events.len(), 2);
        assert!(!events[0].planned);
        assert!(events[1].planned);
    }
    #[test]
    fn overdue_boundaries_postpone_and_species_validation() {
        let (db, patient, vaccine) = fixture();
        let c = db.conn();
        let old = create(c, dose(patient, vaccine, "2025-01-01", Some("2026-01-01"))).unwrap();
        assert_eq!(
            merged_due(
                c,
                VaccineDueQuery {
                    to: Some("2025-12-31".into()),
                    ..Default::default()
                }
            )
            .unwrap()
            .len(),
            0
        );
        assert_eq!(
            merged_due(
                c,
                VaccineDueQuery {
                    to: Some("2026-01-01".into()),
                    ..Default::default()
                }
            )
            .unwrap()
            .len(),
            1
        );
        let moved = postpone(c, None, Some(old.id), "2026-02-01").unwrap();
        assert_eq!(
            merged_due(c, Default::default()).unwrap()[0].appointment_id,
            Some(moved.id)
        );
        assert_eq!(
            get(c, old.id).unwrap().next_due_on.as_deref(),
            Some("2026-01-01")
        );
        let dog = vaccine_types::create(
            c,
            vaccine_types::NewVaccineType {
                name: "Dog only".into(),
                species: Some("dog".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(plan(c, plan_input(patient, dog.id, "2026-02-01")).is_err());
        assert!(give(
            c,
            GiveVaccination {
                vaccination: dose(patient, dog.id, "2026-01-01", None),
                appointment_id: Some(moved.id),
                plan_next: false
            }
        )
        .is_err());
        c.execute(
            "UPDATE patients SET deceased_on = '2026-01-02' WHERE id = ?1",
            [patient],
        )
        .unwrap();
        assert!(merged_due(c, Default::default()).unwrap().is_empty());
    }
}
