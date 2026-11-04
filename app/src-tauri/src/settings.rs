use crate::AppDatabase;
use baytar_core::{backup, db::Database};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

const KEYS: &[&str] = &[
    "clinic_name",
    "vet_name",
    "clinic_phone",
    "clinic_address",
    "clinic_title",
    "clinic_registry_no",
    "clinic_email",
    "clinic_tax_office",
    "clinic_tax_no",
    "clinic_logo",
    "update_check_enabled",
    "inventory_deduct_default",
];

fn paths(app: &tauri::AppHandle) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db = data.join("baytar.db");
    Ok((data.clone(), db, backup::backups_dir(&data)))
}
fn internal() -> String {
    "Settings operation failed.".into()
}

#[tauri::command]
pub fn settings_get_all(state: State<'_, AppDatabase>) -> Result<BTreeMap<String, String>, String> {
    let guard = state.database.lock().map_err(|_| internal())?;
    let db = guard.as_ref().ok_or_else(internal)?;
    let mut values = BTreeMap::from([("update_check_enabled".to_string(), "1".to_string())]);
    let mut statement = db
        .conn()
        .prepare("SELECT key, value FROM settings")
        .map_err(|_| internal())?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| internal())?;
    for row in rows {
        let (key, value) = row.map_err(|_| internal())?;
        if KEYS.contains(&key.as_str()) {
            values.insert(key, value);
        }
    }
    Ok(values)
}

#[tauri::command]
pub fn settings_set(
    state: State<'_, AppDatabase>,
    key: String,
    value: String,
) -> Result<(), String> {
    if !KEYS.contains(&key.as_str())
        || (key == "update_check_enabled" && !["1", "0"].contains(&value.as_str()))
    {
        return Err("Invalid setting.".into());
    }
    let guard = state.database.lock().map_err(|_| internal())?;
    let db = guard.as_ref().ok_or_else(internal)?;
    db.conn().execute("INSERT INTO settings(key, value, updated_at) VALUES (?1, ?2, datetime('now')) ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at", [&key, &value]).map_err(|_| internal())?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRow {
    path: String,
    created_at: String,
    kind: String,
    size_bytes: u64,
}
impl From<backup::BackupInfo> for BackupRow {
    fn from(b: backup::BackupInfo) -> Self {
        Self {
            path: b.path.to_string_lossy().into_owned(),
            created_at: b.created_at.to_rfc3339(),
            kind: b.kind.as_str().into(),
            size_bytes: b.size_bytes,
        }
    }
}
#[tauri::command]
pub fn backups_list(app: tauri::AppHandle) -> Result<Vec<BackupRow>, String> {
    let (_, _, dir) = paths(&app)?;
    backup::list(&dir)
        .map(|v| v.into_iter().map(Into::into).collect())
        .map_err(|_| internal())
}
#[tauri::command]
pub fn backup_create(
    app: tauri::AppHandle,
    state: State<'_, AppDatabase>,
) -> Result<BackupRow, String> {
    let (_, _, dir) = paths(&app)?;
    let guard = state.database.lock().map_err(|_| internal())?;
    backup::create(
        guard.as_ref().ok_or_else(internal)?,
        &dir,
        backup::BackupKind::Manual,
    )
    .map(Into::into)
    .map_err(|_| internal())
}
#[tauri::command]
pub fn backup_before_update(
    app: tauri::AppHandle,
    state: State<'_, AppDatabase>,
) -> Result<BackupRow, String> {
    let (_, _, dir) = paths(&app)?;
    create_pre_update_backup(&state, &dir)
}

fn create_pre_update_backup(state: &AppDatabase, dir: &Path) -> Result<BackupRow, String> {
    let guard = state.database.lock().map_err(|_| internal())?;
    backup::create(
        guard.as_ref().ok_or_else(internal)?,
        dir,
        backup::BackupKind::PreUpdate,
    )
    .map(Into::into)
    .map_err(|_| {
        log::error!("Pre-update backup failed; update must not be installed.");
        internal()
    })
}

#[tauri::command]
pub fn backup_export(
    app: tauri::AppHandle,
    state: State<'_, AppDatabase>,
    path: String,
) -> Result<String, String> {
    let (_, _, _) = paths(&app)?;
    let guard = state.database.lock().map_err(|_| internal())?;
    backup::export_copy(guard.as_ref().ok_or_else(internal)?, Path::new(&path))
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|_| internal())
}
#[tauri::command]
pub fn backup_restore(
    app: tauri::AppHandle,
    state: State<'_, AppDatabase>,
    path: String,
) -> Result<(), String> {
    let (_, db_path, dir) = paths(&app)?;
    let mut guard = state.database.lock().map_err(|_| internal())?;
    let db = guard.take().ok_or_else(internal)?;
    drop(db);
    let result = backup::restore(&db_path, Path::new(&path), &dir).map_err(|_| internal());
    match Database::open(&db_path, &dir) {
        Ok(db) => *guard = Some(db),
        Err(_) => return Err(internal()),
    }
    result.map(|_| ())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    data_folder: String,
    backups_folder: String,
    database_size_bytes: u64,
}
#[tauri::command]
pub fn app_info(app: tauri::AppHandle) -> Result<AppInfo, String> {
    let (data, db, backups) = paths(&app)?;
    let database_size_bytes = fs::metadata(db).map(|m| m.len()).unwrap_or(0);
    Ok(AppInfo {
        version: baytar_core::version(),
        data_folder: data.to_string_lossy().into_owned(),
        backups_folder: backups.to_string_lossy().into_owned(),
        database_size_bytes,
    })
}
#[tauri::command]
pub fn open_data_folder(app: tauri::AppHandle) -> Result<(), String> {
    let (data, _, _) = paths(&app)?;
    app.opener()
        .open_path(data.to_string_lossy(), None::<&str>)
        .map_err(|_| internal())
}

/// Opens a WhatsApp chat link in the default browser. Only `https://wa.me/` links are allowed.
#[tauri::command]
pub fn open_whatsapp(app: tauri::AppHandle, url: String) -> Result<(), String> {
    if !url.starts_with("https://wa.me/") {
        return Err(internal());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| internal())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_update_backup_preserves_current_database_and_returns_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppDatabase::initialize(dir.path());
        state
            .database
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .conn()
            .execute(
                "INSERT INTO settings(key, value) VALUES ('clinic_name', 'Synthetic clinic')",
                [],
            )
            .unwrap();

        let row = create_pre_update_backup(&state, &dir.path().join("backups")).unwrap();
        assert_eq!(row.kind, "pre-update");
        assert!(row.size_bytes > 0);
        assert!(chrono::DateTime::parse_from_rfc3339(&row.created_at).is_ok());
        let snapshot = rusqlite::Connection::open_with_flags(
            row.path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        baytar_core::db::check_integrity(&snapshot).unwrap();
        let value: String = snapshot
            .query_row(
                "SELECT value FROM settings WHERE key = 'clinic_name'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "Synthetic clinic");
    }

    #[test]
    fn pre_update_backup_reports_failure_without_changing_live_data() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppDatabase::initialize(dir.path());
        let blocked = dir.path().join("blocked");
        fs::write(&blocked, "synthetic file").unwrap();
        assert!(create_pre_update_backup(&state, &blocked).is_err());
        let guard = state.database.lock().unwrap();
        baytar_core::db::check_integrity(guard.as_ref().unwrap().conn()).unwrap();
        assert!(create_pre_update_backup(&AppDatabase::failed(None), dir.path()).is_err());
    }
}
