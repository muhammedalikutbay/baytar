mod billing;
mod import;
mod settings;
mod vaccines;

use baytar_core::{
    backup,
    db::{Database, DbError},
    model::{Owner, Patient},
    repo::{
        self,
        owners::{self, NewOwner, OwnerDetail, OwnerPatch},
        patients::{self, NewPatient, PatientPatch},
        search::SearchHit,
        ListQuery, RepoError,
    },
};
use baytar_core::{
    model::Appointment,
    repo::appointments::{self, AppointmentPatch, NewAppointment},
};
use baytar_core::{
    model::Medication,
    repo::medications::{self, MedicationPatch, NewMedication},
};
use baytar_core::{
    model::PrescriptionItem,
    repo::prescriptions::{self, NewPrescription, PrescriptionPatch},
};
use baytar_core::{
    model::Vaccination,
    repo::vaccinations::{self, NewVaccination, VaccinationPatch},
};
use baytar_core::{
    model::VaccineType,
    repo::vaccine_types::{self, NewVaccineType, VaccineTypePatch},
};
use baytar_core::{
    model::Visit,
    repo::visits::{self, NewVisit, VisitPatch},
};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum ErrorKind {
    Invalid,
    NotFound,
    Conflict,
    Internal,
}
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    kind: ErrorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    message: String,
}
impl CommandError {
    fn internal() -> Self {
        Self {
            kind: ErrorKind::Internal,
            field: None,
            message: "The database operation could not be completed.".into(),
        }
    }
}
impl From<RepoError> for CommandError {
    fn from(error: RepoError) -> Self {
        match error {
            RepoError::Invalid { field, reason } => Self {
                kind: ErrorKind::Invalid,
                field: Some(field),
                message: reason,
            },
            RepoError::NotFound => Self {
                kind: ErrorKind::NotFound,
                field: None,
                message: "Record not found.".into(),
            },
            RepoError::Conflict => Self {
                kind: ErrorKind::Conflict,
                field: None,
                message: "The record conflicts with its current state.".into(),
            },
            // Do not expose SQL or record values to the frontend or logs.
            RepoError::Database(_) => Self::internal(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub ready: bool,
    pub error: Option<CommandError>,
    /// Stable codes let the UI translate startup failures in its central locale file.
    pub code: Option<String>,
    pub found_version: Option<u32>,
    pub supported_version: Option<u32>,
}
pub struct AppDatabase {
    pub(crate) database: Mutex<Option<Database>>,
    status: StartupStatus,
}
impl AppDatabase {
    fn initialize(dir: &std::path::Path) -> Self {
        let result = (|| {
            std::fs::create_dir_all(dir)?;
            Database::open(&dir.join("baytar.db"), &dir.join("backups"))
        })();
        match result {
            Ok(db) => {
                if repo::vaccine_types::seed(db.conn()).is_err() {
                    return Self::failed(None);
                }
                let now = chrono::Utc::now();
                let backups = dir.join("backups");
                // Preserve a usable database if maintenance fails, and expose the warning.
                let maintenance_failed = backup::daily_if_due(&db, &backups, now).is_err()
                    || backup::prune(&backups, &backup::BackupPolicy::default(), now).is_err();
                let status = StartupStatus {
                    ready: true,
                    error: maintenance_failed.then(CommandError::internal),
                    code: maintenance_failed.then(|| "backup_failed".into()),
                    found_version: None,
                    supported_version: None,
                };
                Self {
                    database: Mutex::new(Some(db)),
                    status,
                }
            }
            Err(error) => Self::failed(Some(error)),
        }
    }
    fn failed(error: Option<DbError>) -> Self {
        let (code, found_version, supported_version, message) = match error {
            Some(DbError::NewerSchema { found, supported }) => (
                "newer_schema",
                Some(found),
                Some(supported),
                "This database requires a newer version of Baytar.",
            ),
            Some(DbError::NotBaytarDatabase) => (
                "not_baytar_database",
                None,
                None,
                "The file is not a Baytar database.",
            ),
            _ => (
                "database_unavailable",
                None,
                None,
                "The application database could not be opened.",
            ),
        };
        Self {
            database: Mutex::new(None),
            status: StartupStatus {
                ready: false,
                error: Some(CommandError {
                    kind: ErrorKind::Internal,
                    field: None,
                    message: message.into(),
                }),
                code: Some(code.into()),
                found_version,
                supported_version,
            },
        }
    }
    fn with<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> repo::Result<T>,
    ) -> Result<T, CommandError> {
        let guard = self.database.lock().map_err(|_| CommandError::internal())?;
        let db = guard.as_ref().ok_or_else(|| {
            self.status
                .error
                .clone()
                .unwrap_or_else(CommandError::internal)
        })?;
        f(db.conn()).map_err(Into::into)
    }
}
#[tauri::command]
fn app_version() -> &'static str {
    baytar_core::version()
}
#[tauri::command]
fn startup_status(state: State<'_, AppDatabase>) -> StartupStatus {
    state.status.clone()
}

#[tauri::command]
fn owners_list(
    state: State<'_, AppDatabase>,
    query: ListQuery,
) -> Result<Vec<Owner>, CommandError> {
    state.with(|conn| owners::list(conn, query))
}

#[tauri::command]
fn owner_get(state: State<'_, AppDatabase>, id: i64) -> Result<Owner, CommandError> {
    state.with(|conn| owners::get(conn, id))
}

#[tauri::command]
fn owner_detail(state: State<'_, AppDatabase>, id: i64) -> Result<OwnerDetail, CommandError> {
    state.with(|conn| owners::owner_detail(conn, id))
}

#[tauri::command]
fn owner_create(state: State<'_, AppDatabase>, input: NewOwner) -> Result<Owner, CommandError> {
    state.with(|conn| owners::create(conn, input))
}

#[tauri::command]
fn owner_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: OwnerPatch,
) -> Result<Owner, CommandError> {
    state.with(|conn| owners::update(conn, id, patch))
}

#[tauri::command]
fn owner_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Vec<i64>, CommandError> {
    state.with(|conn| owners::soft_delete(conn, id))
}

#[tauri::command]
fn owner_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Owner, CommandError> {
    state.with(|conn| owners::restore(conn, id))
}

#[tauri::command]
fn patients_by_owner(
    state: State<'_, AppDatabase>,
    owner_id: i64,
) -> Result<Vec<Patient>, CommandError> {
    state.with(|conn| patients::list_by_owner(conn, owner_id))
}

#[tauri::command]
fn patient_get(state: State<'_, AppDatabase>, id: i64) -> Result<Patient, CommandError> {
    state.with(|conn| patients::get(conn, id))
}

#[tauri::command]
fn patient_create(
    state: State<'_, AppDatabase>,
    input: NewPatient,
) -> Result<Patient, CommandError> {
    state.with(|conn| patients::create(conn, input))
}

#[tauri::command]
fn patient_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: PatientPatch,
) -> Result<Patient, CommandError> {
    state.with(|conn| patients::update(conn, id, patch))
}

#[tauri::command]
fn patient_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Patient, CommandError> {
    state.with(|conn| patients::soft_delete(conn, id))
}

#[tauri::command]
fn patient_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Patient, CommandError> {
    state.with(|conn| patients::restore(conn, id))
}

#[tauri::command]
fn search(
    state: State<'_, AppDatabase>,
    query: String,
    limit: u32,
) -> Result<Vec<SearchHit>, CommandError> {
    state.with(|conn| repo::search::search(conn, &query, limit))
}

#[tauri::command]
fn visit_get(state: State<'_, AppDatabase>, id: i64) -> Result<Visit, CommandError> {
    state.with(|conn| visits::get(conn, id))
}
#[tauri::command]
fn visit_create(state: State<'_, AppDatabase>, input: NewVisit) -> Result<Visit, CommandError> {
    state.with(|conn| visits::create(conn, input))
}
#[tauri::command]
fn visit_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: VisitPatch,
) -> Result<Visit, CommandError> {
    state.with(|conn| visits::update(conn, id, patch))
}
#[tauri::command]
fn visit_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Visit, CommandError> {
    state.with(|conn| visits::soft_delete(conn, id))
}
#[tauri::command]
fn visit_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Visit, CommandError> {
    state.with(|conn| visits::restore(conn, id))
}
#[tauri::command]
fn visits_by_patient(
    state: State<'_, AppDatabase>,
    patient_id: i64,
) -> Result<Vec<Visit>, CommandError> {
    state.with(|conn| visits::list_by_patient(conn, patient_id))
}
#[tauri::command]
fn visit_latest_weight(
    state: State<'_, AppDatabase>,
    patient_id: i64,
) -> Result<Option<visits::LatestWeight>, CommandError> {
    state.with(|conn| visits::latest_weight(conn, patient_id))
}
#[tauri::command]
fn vaccine_type_get(state: State<'_, AppDatabase>, id: i64) -> Result<VaccineType, CommandError> {
    state.with(|conn| vaccine_types::get(conn, id))
}
#[tauri::command]
fn vaccine_type_create(
    state: State<'_, AppDatabase>,
    input: NewVaccineType,
) -> Result<VaccineType, CommandError> {
    state.with(|conn| vaccine_types::create(conn, input))
}
#[tauri::command]
fn vaccine_type_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: VaccineTypePatch,
) -> Result<VaccineType, CommandError> {
    state.with(|conn| vaccine_types::update(conn, id, patch))
}
#[tauri::command]
fn vaccine_type_delete(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<VaccineType, CommandError> {
    state.with(|conn| vaccine_types::soft_delete(conn, id))
}
#[tauri::command]
fn vaccine_type_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<VaccineType, CommandError> {
    state.with(|conn| vaccine_types::restore(conn, id))
}
#[tauri::command]
fn vaccine_types_list(state: State<'_, AppDatabase>) -> Result<Vec<VaccineType>, CommandError> {
    state.with(vaccine_types::list)
}
#[tauri::command]
fn vaccination_get(state: State<'_, AppDatabase>, id: i64) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::get(conn, id))
}
#[tauri::command]
fn vaccination_create(
    state: State<'_, AppDatabase>,
    input: NewVaccination,
) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::create(conn, input))
}
#[tauri::command]
fn vaccination_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: VaccinationPatch,
) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::update(conn, id, patch))
}
#[tauri::command]
fn vaccination_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::soft_delete(conn, id))
}
#[tauri::command]
fn vaccination_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::restore(conn, id))
}
#[tauri::command]
fn vaccinations_by_patient(
    state: State<'_, AppDatabase>,
    patient_id: i64,
) -> Result<Vec<Vaccination>, CommandError> {
    state.with(|conn| vaccinations::list_by_patient(conn, patient_id))
}
#[tauri::command]
fn vaccination_suggest_next_due(
    state: State<'_, AppDatabase>,
    type_id: i64,
    given_on: String,
) -> Result<Option<String>, CommandError> {
    state.with(|conn| vaccinations::suggest_next_due(conn, type_id, &given_on))
}
#[tauri::command]
fn vaccinations_due_between(
    state: State<'_, AppDatabase>,
    from: String,
    to: String,
) -> Result<Vec<vaccinations::DueVaccination>, CommandError> {
    state.with(|conn| vaccinations::due_between(conn, &from, &to))
}
#[tauri::command]
fn vaccinations_overdue(
    state: State<'_, AppDatabase>,
    today: String,
) -> Result<Vec<vaccinations::DueVaccination>, CommandError> {
    state.with(|conn| vaccinations::overdue(conn, &today))
}
#[tauri::command]
fn dose_calculate(
    input: baytar_core::dose::DoseInput,
) -> Result<baytar_core::dose::DoseResult, CommandError> {
    baytar_core::dose::calculate(&input).map_err(|error| CommandError {
        kind: ErrorKind::Invalid,
        field: match &error {
            baytar_core::dose::DoseError::Invalid { field, .. } => Some((*field).into()),
            baytar_core::dose::DoseError::OutOfRange => None,
        },
        message: error.to_string(),
    })
}

#[tauri::command]
fn medication_get(state: State<'_, AppDatabase>, id: i64) -> Result<Medication, CommandError> {
    state.with(|conn| medications::get(conn, id))
}
#[tauri::command]
fn medication_create(
    state: State<'_, AppDatabase>,
    input: NewMedication,
) -> Result<Medication, CommandError> {
    state.with(|conn| medications::create(conn, input))
}
#[tauri::command]
fn medication_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: MedicationPatch,
) -> Result<Medication, CommandError> {
    state.with(|conn| medications::update(conn, id, patch))
}
#[tauri::command]
fn medication_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Medication, CommandError> {
    state.with(|conn| medications::soft_delete(conn, id))
}
#[tauri::command]
fn medication_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Medication, CommandError> {
    state.with(|conn| medications::restore(conn, id))
}
#[tauri::command]
fn medications_list(state: State<'_, AppDatabase>) -> Result<Vec<Medication>, CommandError> {
    state.with(medications::list)
}
#[tauri::command]
fn medications_search(
    state: State<'_, AppDatabase>,
    query: String,
) -> Result<Vec<Medication>, CommandError> {
    state.with(|conn| medications::search(conn, &query))
}
#[tauri::command]
fn prescription_get(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<PrescriptionItem, CommandError> {
    state.with(|conn| prescriptions::get(conn, id))
}
#[tauri::command]
fn prescription_create(
    state: State<'_, AppDatabase>,
    input: NewPrescription,
) -> Result<PrescriptionItem, CommandError> {
    state.with(|conn| prescriptions::create(conn, input))
}
#[tauri::command]
fn prescription_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: PrescriptionPatch,
) -> Result<PrescriptionItem, CommandError> {
    state.with(|conn| prescriptions::update(conn, id, patch))
}
#[tauri::command]
fn prescription_delete(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<PrescriptionItem, CommandError> {
    state.with(|conn| prescriptions::soft_delete(conn, id))
}
#[tauri::command]
fn prescription_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<PrescriptionItem, CommandError> {
    state.with(|conn| prescriptions::restore(conn, id))
}
#[tauri::command]
fn prescriptions_by_visit(
    state: State<'_, AppDatabase>,
    visit_id: i64,
) -> Result<Vec<PrescriptionItem>, CommandError> {
    state.with(|conn| prescriptions::list_by_visit(conn, visit_id))
}
#[tauri::command]
fn appointment_get(state: State<'_, AppDatabase>, id: i64) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::get(conn, id))
}
#[tauri::command]
fn appointment_create(
    state: State<'_, AppDatabase>,
    input: NewAppointment,
) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::create(conn, input))
}
#[tauri::command]
fn appointment_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: AppointmentPatch,
) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::update(conn, id, patch))
}
#[tauri::command]
fn appointment_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::soft_delete(conn, id))
}
#[tauri::command]
fn appointment_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::restore(conn, id))
}
#[tauri::command]
fn appointment_set_status(
    state: State<'_, AppDatabase>,
    id: i64,
    status: String,
) -> Result<Appointment, CommandError> {
    state.with(|conn| appointments::set_status(conn, id, &status))
}
#[tauri::command]
fn appointment_create_series(
    state: State<'_, AppDatabase>,
    base: NewAppointment,
    count: u32,
    every_days: i64,
) -> Result<Vec<Appointment>, CommandError> {
    state.with(|conn| appointments::create_series(conn, base, count, every_days))
}
#[tauri::command]
fn appointments_between(
    state: State<'_, AppDatabase>,
    from: String,
    to: String,
) -> Result<Vec<appointments::CalendarAppointment>, CommandError> {
    state.with(|conn| appointments::between(conn, &from, &to))
}
#[tauri::command]
fn appointments_today(
    state: State<'_, AppDatabase>,
    today: String,
) -> Result<Vec<appointments::CalendarAppointment>, CommandError> {
    state.with(|conn| appointments::today(conn, &today))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!(
        "Starting {} {}",
        baytar_core::APP_NAME,
        baytar_core::version()
    );
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            import::initialize(app)?;
            let state = match app.path().app_data_dir() {
                Ok(dir) => AppDatabase::initialize(&dir),
                Err(_) => AppDatabase::failed(None),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            import::import_candidates,
            import::import_prepare,
            import::import_execute,
            import::import_cancel,
            import::import_save_report,
            settings::settings_get_all,
            settings::settings_set,
            settings::backups_list,
            settings::backup_create,
            settings::backup_before_update,
            settings::backup_export,
            settings::backup_restore,
            settings::app_info,
            settings::open_data_folder,
            settings::open_whatsapp,
            visit_get,
            visit_create,
            visit_update,
            visit_delete,
            visit_restore,
            visits_by_patient,
            visit_latest_weight,
            vaccine_type_get,
            vaccine_type_create,
            vaccine_type_update,
            vaccine_type_delete,
            vaccine_type_restore,
            vaccine_types_list,
            vaccination_get,
            vaccination_create,
            vaccination_update,
            vaccination_delete,
            vaccination_restore,
            vaccinations_by_patient,
            vaccination_suggest_next_due,
            vaccinations_due_between,
            vaccinations_overdue,
            medication_get,
            dose_calculate,
            medication_create,
            medication_update,
            medication_delete,
            medication_restore,
            medications_list,
            medications_search,
            prescription_get,
            prescription_create,
            prescription_update,
            prescription_delete,
            prescription_restore,
            prescriptions_by_visit,
            appointment_get,
            appointment_create,
            appointment_update,
            appointment_delete,
            appointment_restore,
            appointment_set_status,
            appointment_create_series,
            appointments_between,
            appointments_today,
            app_version,
            startup_status,
            owners_list,
            owner_get,
            owner_detail,
            owner_create,
            owner_update,
            owner_delete,
            owner_restore,
            patients_by_owner,
            patient_get,
            patient_create,
            patient_update,
            patient_delete,
            patient_restore,
            search,
            billing::service_get,
            billing::service_create,
            billing::service_update,
            billing::service_delete,
            billing::service_restore,
            billing::services_list,
            billing::charge_get,
            billing::charge_create,
            billing::charge_update,
            billing::charge_delete,
            billing::charge_restore,
            billing::charges_list,
            billing::payment_get,
            billing::payment_create,
            billing::payment_update,
            billing::payment_delete,
            billing::payment_restore,
            billing::payments_list,
            billing::charges_by_visit,
            billing::account_balance,
            billing::account_statement,
            billing::accounts_receivables,
            billing::billing_daily_summary,
            billing::billing_charge_sources,
            vaccines::vaccines_due_list,
            vaccines::vaccines_give,
            vaccines::vaccines_plan,
            vaccines::vaccines_suggest,
            vaccines::vaccines_calendar,
            vaccines::vaccines_postpone
        ])
        .run(tauri::generate_context!())
        .expect("error while running Baytar");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_failure_is_serializable_and_commands_remain_available() {
        let state = AppDatabase::failed(Some(DbError::NewerSchema {
            found: 99,
            supported: 1,
        }));
        assert!(!state.status.ready);
        assert_eq!(state.status.code.as_deref(), Some("newer_schema"));
        assert_eq!(state.status.found_version, Some(99));
        assert!(state.with(|_| Ok(())).is_err());
    }
    #[test]
    fn fresh_startup_creates_database_and_daily_backup() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppDatabase::initialize(dir.path());
        assert!(state.status.ready);
        assert!(state.status.error.is_none());
        assert_eq!(backup::list(&dir.path().join("backups")).unwrap().len(), 1);
    }
}
