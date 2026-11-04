//! Report and CSV export commands. Row contents are never logged; file IO runs after the
//! database lock is released.
use crate::{AppDatabase, CommandError};
use baytar_core::{
    export::{self, ExportTable},
    repo::{
        reports::{self, ReportOverview},
        RepoError,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;

fn invalid(field: &str, reason: &str) -> CommandError {
    RepoError::Invalid {
        field: field.into(),
        reason: reason.into(),
    }
    .into()
}

#[tauri::command]
pub fn reports_overview(
    state: State<'_, AppDatabase>,
    from: String,
    to: String,
    today: String,
) -> Result<ReportOverview, CommandError> {
    state.with(|c| reports::overview(c, &from, &to, &today))
}

/// A single-table export target: always a `.csv` file, never a database or its sidecars.
fn csv_target(path: &str) -> Result<PathBuf, CommandError> {
    let mut target = PathBuf::from(path);
    let extension = target
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ["db", "sqlite", "sqlite3", "db-wal", "db-shm"].contains(&extension.as_str()) {
        return Err(invalid("path", "cannot export over a database"));
    }
    if target.file_name().is_none() {
        return Err(invalid("path", "must name a file"));
    }
    if extension.is_empty() {
        target.set_extension("csv");
    }
    Ok(target)
}

/// `baytar-<table>-<date>.csv`, with a numeric suffix when a file of that name already exists.
fn folder_target(dir: &Path, table: ExportTable, today: &str) -> PathBuf {
    let stem = format!("baytar-{}-{today}", table.stem());
    let mut path = dir.join(format!("{stem}.csv"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem}-{n}.csv"));
        n += 1;
    }
    path
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), CommandError> {
    fs::write(path, bytes).map_err(|_| CommandError::internal())
}

#[tauri::command]
pub fn reports_export_csv(
    state: State<'_, AppDatabase>,
    table: ExportTable,
    path: String,
) -> Result<String, CommandError> {
    let target = csv_target(&path)?;
    let bytes = state.with(|c| export::export_csv(c, table))?;
    write(&target, &bytes)?;
    Ok(target.to_string_lossy().into_owned())
}

/// Writes every table into `dir` and returns the written paths.
#[tauri::command]
pub fn reports_export_all(
    state: State<'_, AppDatabase>,
    dir: String,
    today: String,
) -> Result<Vec<String>, CommandError> {
    let dir = PathBuf::from(dir);
    if !dir.is_dir() {
        return Err(invalid("path", "must be an existing folder"));
    }
    if today.len() != 10 || !today.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return Err(invalid("today", "expected a YYYY-MM-DD date"));
    }
    let mut written = Vec::new();
    for table in ExportTable::ALL {
        let bytes = state.with(|c| export::export_csv(c, table))?;
        let path = folder_target(&dir, table, &today);
        write(&path, &bytes)?;
        written.push(path.to_string_lossy().into_owned());
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_file_target_is_csv_and_never_a_database() {
        assert_eq!(
            csv_target("out/owners").unwrap(),
            PathBuf::from("out/owners.csv")
        );
        assert_eq!(
            csv_target("out/owners.txt").unwrap(),
            PathBuf::from("out/owners.txt")
        );
        assert!(csv_target("baytar.db").is_err());
        assert!(csv_target("x.SQLITE3").is_err());
        assert!(csv_target("").is_err());
    }

    #[test]
    fn folder_export_never_overwrites_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let first = folder_target(dir.path(), ExportTable::Owners, "2026-11-04");
        assert!(first.ends_with("baytar-owners-2026-11-04.csv"));
        fs::write(&first, b"x").unwrap();
        let second = folder_target(dir.path(), ExportTable::Owners, "2026-11-04");
        assert!(second.ends_with("baytar-owners-2026-11-04-2.csv"));
    }
}
