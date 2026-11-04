mod import;

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
    database: Mutex<Option<Database>>,
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
            search
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
