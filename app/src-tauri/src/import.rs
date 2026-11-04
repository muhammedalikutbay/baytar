use crate::{AppDatabase, CommandError, ErrorKind};
use baytar_core::legacy::{
    import::{self, ImportPreview, ImportReport, PreparedImport},
    LegacyError,
};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Manager, State};

#[derive(Default)]
pub struct ImportState {
    prepared: Mutex<HashMap<String, PreparedImport>>,
    completed: Mutex<HashMap<String, i64>>,
}

pub fn initialize(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let work = crate::data_dir(app.path())?.join("import-work");
    if work.exists() {
        fs::remove_dir_all(&work)?;
    }
    fs::create_dir_all(work)?;
    app.manage(ImportState::default());
    Ok(())
}
fn failure(code: &str, invalid: bool) -> CommandError {
    CommandError {
        kind: if invalid {
            ErrorKind::Invalid
        } else {
            ErrorKind::Internal
        },
        field: None,
        message: code.into(),
    }
}
fn legacy_error(error: LegacyError) -> CommandError {
    match error {
        LegacyError::NotLegacyDatabase => failure("not_legacy_database", true),
        LegacyError::Backup(_) => failure("import_backup_failed", false),
        _ => failure("import_failed", false),
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    path: String,
    file_name: String,
    size_bytes: u64,
    modified_at: Option<String>,
    location: String,
}
fn excluded(path: &Path, metadata: &fs::Metadata) -> bool {
    if path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        || metadata.is_symlink()
    {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & (2 | 4 | 1024) != 0 {
            return true;
        }
    }
    false
}
fn scan(
    dir: &Path,
    depth: u8,
    deadline: Instant,
    seen: &mut HashSet<PathBuf>,
    found: &mut Vec<Candidate>,
) {
    if Instant::now() >= deadline || !seen.insert(dir.to_path_buf()) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut files = Vec::new();
    let mut folders = Vec::new();
    let mut executable = false;
    for entry in entries.flatten() {
        if Instant::now() >= deadline {
            return;
        }
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if excluded(&path, &meta) {
            continue;
        }
        if meta.is_dir() {
            folders.push(path);
        } else if meta.is_file() {
            executable |= entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("PatientCare.exe");
            files.push((path, meta));
        }
    }
    for (path, meta) in files {
        if Instant::now() >= deadline {
            break;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if name.eq_ignore_ascii_case("HastaTakipDB.db")
            || name.eq_ignore_ascii_case("DBPatientCare.db")
            || (executable
                && path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("db")))
        {
            found.push(Candidate {
                path: path.to_string_lossy().into_owned(),
                file_name: name,
                size_bytes: meta.len(),
                modified_at: meta
                    .modified()
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
                location: dir.to_string_lossy().into_owned(),
            });
        }
    }
    if depth < 3 {
        for folder in folders {
            scan(&folder, depth + 1, deadline, seen, found);
        }
    }
}
#[tauri::command]
pub async fn import_candidates(app: tauri::AppHandle) -> Result<Vec<Candidate>, CommandError> {
    let paths = app.path();
    let roots = [
        paths.desktop_dir(),
        paths.document_dir(),
        paths.download_dir(),
        paths.home_dir(),
    ];
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for root in roots.into_iter().flatten() {
        scan(&root, 0, deadline, &mut seen, &mut found);
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found.dedup_by(|a, b| a.path == b.path);
    Ok(found)
}
#[derive(Serialize)]
pub struct Prepared {
    token: String,
    preview: ImportPreview,
}
#[tauri::command]
pub async fn import_prepare(
    app: tauri::AppHandle,
    state: State<'_, ImportState>,
    database: State<'_, AppDatabase>,
    path: String,
) -> Result<Prepared, CommandError> {
    let token = uuid::Uuid::new_v4().to_string();
    let work = crate::data_dir(app.path())
        .map_err(|_| CommandError::internal())?
        .join("import-work")
        .join(&token);
    // Acquire import state before the database consistently across all commands.
    let mut pending = state
        .prepared
        .lock()
        .map_err(|_| CommandError::internal())?;
    let result = (|| {
        let prepared = import::prepare(Path::new(&path), &work).map_err(legacy_error)?;
        let guard = database
            .database
            .lock()
            .map_err(|_| CommandError::internal())?;
        let db = guard.as_ref().ok_or_else(CommandError::internal)?;
        let preview = import::preview(&prepared, db).map_err(legacy_error)?;
        pending.insert(token.clone(), prepared);
        Ok(Prepared { token, preview })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(work);
    }
    result
}
#[tauri::command]
pub async fn import_execute(
    app: tauri::AppHandle,
    state: State<'_, ImportState>,
    database: State<'_, AppDatabase>,
    token: String,
) -> Result<ImportReport, CommandError> {
    let mut prepared = state
        .prepared
        .lock()
        .map_err(|_| CommandError::internal())?;
    let item = prepared
        .get(&token)
        .ok_or_else(|| failure("import_expired", true))?;
    let backups = crate::data_dir(app.path())
        .map_err(|_| CommandError::internal())?
        .join("backups");
    let mut guard = database
        .database
        .lock()
        .map_err(|_| CommandError::internal())?;
    let db = guard.as_mut().ok_or_else(CommandError::internal)?;
    let report = import::execute(item, db, &backups).map_err(legacy_error)?;
    state
        .completed
        .lock()
        .map_err(|_| CommandError::internal())?
        .insert(token.clone(), report.run_id);
    prepared.remove(&token);
    Ok(report)
}
#[tauri::command]
pub async fn import_cancel(
    state: State<'_, ImportState>,
    token: String,
) -> Result<(), CommandError> {
    state
        .prepared
        .lock()
        .map_err(|_| CommandError::internal())?
        .remove(&token);
    Ok(())
}

fn report_text(value: &serde_json::Value) -> String {
    let mut out = format!(
        "PatientsCare → Baytar\nDosya: {}\nEski dosyanız değiştirilmez.\n",
        value["sourceName"].as_str().unwrap_or_default()
    );
    for (key, label) in [
        ("found", "Dosyadaki kayıtlar"),
        ("toImport", "Aktarılacak kayıtlar"),
        ("inserted", "Aktarılan kayıtlar"),
        ("alreadyImported", "Daha önce aktarılan kayıtlar"),
    ] {
        if let Some(c) = value.get(key) {
            out.push_str(&format!(
                "{label}: {} sahip, {} hayvan, {} hatırlatma\n",
                c["owners"], c["patients"], c["todos"]
            ));
        }
    }
    if let Some(backup) = value["backupFile"].as_str() {
        out.push_str(&format!("Aktarma öncesi yedek: {backup}\n"));
    }
    out.push_str("\nDikkat gerektiren kayıtlar (eski değerler korunur):\n");
    if let Some(issues) = value["issues"].as_array() {
        for issue in issues {
            out.push_str(&format!(
                "{} #{} · {} · {} · {}\n",
                issue["table"],
                issue["sourceId"],
                issue["field"],
                issue_label(issue["kind"].as_str().unwrap_or_default()),
                issue["raw"]
            ));
        }
    }
    out
}
fn issue_label(kind: &str) -> &str {
    match kind {
        "phoneUnparsed" => "Telefon okunamadı",
        "dateUnparsed" => "Tarih okunamadı",
        "speciesUnrecognized" => "Tür diğer olarak saklandı",
        "sexUnrecognized" => "Cinsiyet bilinmiyor",
        "missingName" => "Ad eksik",
        "unusableId" => "Kayıt numarası yenilendi",
        "orphanPatient" => "Hayvanın sahibi bulunamadı",
        "orphanTodo" => "Hatırlatmanın sahibi bulunamadı",
        "duplicateOwner" => "Benzer sahip",
        "ownerMismatch" => "Sahip bilgileri farklı",
        "danglingPatientLink" => "Bağlı hayvan bulunamadı",
        "danglingOwnerLink" => "Bağlı sahip bulunamadı",
        _ => kind,
    }
}
#[tauri::command]
pub async fn import_save_report(
    state: State<'_, ImportState>,
    database: State<'_, AppDatabase>,
    token_or_run_id: String,
    path: String,
) -> Result<(), CommandError> {
    if !Path::new(&path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
    {
        return Err(failure("report_save_failed", true));
    }
    let prepared = state
        .prepared
        .lock()
        .map_err(|_| CommandError::internal())?;
    let guard = database
        .database
        .lock()
        .map_err(|_| CommandError::internal())?;
    let db = guard.as_ref().ok_or_else(CommandError::internal)?;
    let value = if let Some(item) = prepared.get(&token_or_run_id) {
        serde_json::to_value(import::preview(item, db).map_err(legacy_error)?)
            .map_err(|_| CommandError::internal())?
    } else {
        let id = state
            .completed
            .lock()
            .map_err(|_| CommandError::internal())?
            .get(&token_or_run_id)
            .copied()
            .or_else(|| token_or_run_id.parse::<i64>().ok())
            .ok_or_else(|| failure("import_expired", true))?;
        let json: String = db
            .conn()
            .query_row(
                "SELECT report_json FROM import_runs WHERE id = ?1 AND status = 'completed'",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| failure("import_expired", true))?;
        serde_json::from_str(&json).map_err(|_| CommandError::internal())?
    };
    fs::write(path, report_text(&value)).map_err(|_| failure("report_save_failed", false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use baytar_core::db::Database;

    #[test]
    fn discovery_respects_names_depth_hidden_folders_and_deadline() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path();
        for folder in ["clinic", ".hidden", "a/b/c", "a/b/c/d"] {
            fs::create_dir_all(dir.join(folder)).unwrap();
        }
        for file in [
            "hastatakipdb.DB",
            "other.db",
            "clinic/PATIENTCARE.EXE",
            "clinic/custom.DB",
            ".hidden/HastaTakipDB.db",
            "a/b/c/DBPatientCare.db",
            "a/b/c/d/HastaTakipDB.db",
        ] {
            fs::write(dir.join(file), b"synthetic").unwrap();
        }
        let mut found = Vec::new();
        scan(
            dir,
            0,
            Instant::now() + Duration::from_secs(2),
            &mut HashSet::new(),
            &mut found,
        );
        assert_eq!(found.len(), 3);
        assert!(found.iter().any(|c| c.file_name == "custom.DB"));
        assert!(found
            .iter()
            .all(|c| c.size_bytes == 9 && c.modified_at.is_some()));
        let mut expired = Vec::new();
        scan(dir, 0, Instant::now(), &mut HashSet::new(), &mut expired);
        assert!(expired.is_empty());
    }

    #[test]
    fn report_preserves_raw_values_and_backup_and_copy_is_disposable() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("HastaTakipDB.db");
        let conn = rusqlite::Connection::open(&source).unwrap();
        conn.execute_batch("CREATE TABLE PatientOwner(Id INTEGER, OwnerName TEXT, OwnerPhone TEXT); INSERT INTO PatientOwner VALUES(1, 'Synthetic owner', 'unknown phone');").unwrap();
        drop(conn);
        let original = fs::read(&source).unwrap();
        let work = root.path().join("work");
        let prepared = import::prepare(&source, &work).unwrap();
        let mut db =
            Database::open(&root.path().join("baytar.db"), &root.path().join("backups")).unwrap();
        let preview = import::preview(&prepared, &db).unwrap();
        assert_eq!(preview.to_import.owners, 1);
        let report = import::execute(&prepared, &mut db, &root.path().join("backups")).unwrap();
        let value = serde_json::to_value(&report).unwrap();
        let text = report_text(&value);
        assert!(text.contains("unknown phone"));
        assert!(text.contains("Telefon okunamadı"));
        assert!(text.contains(&report.backup_file));
        assert!(root
            .path()
            .join("backups")
            .join(&report.backup_file)
            .is_file());
        assert_eq!(
            import::preview(&prepared, &db).unwrap().to_import.total(),
            0
        );
        drop(prepared);
        assert_eq!(fs::read_dir(work).unwrap().count(), 0);
        assert_eq!(fs::read(source).unwrap(), original);
        let invalid = legacy_error(LegacyError::NotLegacyDatabase);
        assert!(matches!(invalid.kind, ErrorKind::Invalid));
        assert_eq!(invalid.message, "not_legacy_database");
    }
}
