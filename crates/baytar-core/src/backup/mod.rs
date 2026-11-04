//! Database backup and restore operations.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rusqlite::{Connection, OpenFlags};

use crate::db::{backup_to, check_integrity, migrations, Database, DbError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub kind: BackupKind,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupKind {
    Daily,
    Manual,
    PreMigration,
    PreRestore,
    PreImport,
    PreUpdate,
}

impl BackupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Manual => "manual",
            Self::PreMigration => "pre-migration",
            Self::PreRestore => "pre-restore",
            Self::PreImport => "pre-import",
            Self::PreUpdate => "pre-update",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BackupPolicy {
    pub keep_daily: usize,
    pub keep_weekly: usize,
}

impl Default for BackupPolicy {
    fn default() -> Self {
        Self {
            keep_daily: 14,
            keep_weekly: 8,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error(transparent)]
    Database(#[from] DbError),
    #[error("backup io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("a backup already exists or is being written for this kind and second: {0}")]
    NameCollision(PathBuf),
}

pub fn backups_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

pub fn create(db: &Database, dir: &Path, kind: BackupKind) -> Result<BackupInfo, BackupError> {
    create_at(db.conn(), dir, kind, Utc::now())
}

pub fn daily_if_due(
    db: &Database,
    dir: &Path,
    now: DateTime<Utc>,
) -> Result<Option<BackupInfo>, BackupError> {
    if list(dir)?.iter().any(|backup| {
        backup.kind == BackupKind::Daily && backup.created_at.date_naive() == now.date_naive()
    }) {
        log::debug!("daily backup already exists for {}", now.date_naive());
        return Ok(None);
    }
    create_at(db.conn(), dir, BackupKind::Daily, now).map(Some)
}

/// Returns only regular files with a recognized kind and a canonical UTC timestamp.
/// A missing backup directory is an empty collection.
pub fn list(dir: &Path) -> Result<Vec<BackupInfo>, BackupError> {
    log::debug!("listing backups in {}", dir.display());
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut backups = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some((kind, created_at)) = name.to_str().and_then(parse_name) else {
            continue;
        };
        backups.push(BackupInfo {
            path: entry.path(),
            created_at,
            kind,
            size_bytes: entry.metadata()?.len(),
        });
    }
    backups.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(backups)
}

/// Keeps all backups on the newest `keep_daily` distinct dates, plus one per ISO
/// week in the `keep_weekly` weeks ending in the current week. The two sets can
/// overlap. Future-dated backups are preserved to tolerate clock corrections.
pub fn prune(
    dir: &Path,
    policy: &BackupPolicy,
    now: DateTime<Utc>,
) -> Result<Vec<PathBuf>, BackupError> {
    log::info!("pruning daily backups in {}", dir.display());
    let mut days = HashSet::new();
    let mut weeks = HashSet::new();
    let current_monday =
        now.date_naive() - chrono::Duration::days(i64::from(now.weekday().num_days_from_monday()));
    let mut removed = Vec::new();
    for backup in list(dir)? {
        if backup.kind != BackupKind::Daily || backup.created_at > now {
            continue;
        }
        let date = backup.created_at.date_naive();
        if days.len() < policy.keep_daily {
            days.insert(date);
        }
        let monday =
            date - chrono::Duration::days(i64::from(date.weekday().num_days_from_monday()));
        let weeks_ago = (current_monday - monday).num_weeks();
        let in_weekly_window =
            usize::try_from(weeks_ago).is_ok_and(|weeks_ago| weeks_ago < policy.keep_weekly);
        let keep_weekly = in_weekly_window && weeks.insert(date.iso_week());
        if !days.contains(&date) && !keep_weekly {
            log::info!("removing daily backup {}", backup.path.display());
            fs::remove_file(&backup.path)?;
            removed.push(backup.path);
        }
    }
    Ok(removed)
}

/// Writes a verified manual backup directly to an external destination directory.
pub fn export_copy(db: &Database, dest_dir: &Path) -> Result<PathBuf, BackupError> {
    log::info!("exporting database backup to {}", dest_dir.display());
    Ok(create(db, dest_dir, BackupKind::Manual)?.path)
}

/// The caller must close every connection to the target before restoring.
pub fn restore(db_path: &Path, backup: &Path, dir: &Path) -> Result<BackupInfo, BackupError> {
    log::info!("validating restore source {}", backup.display());
    // SQLite treats a zero-byte file as an empty database. A published snapshot
    // must have a database header; otherwise truncation could silently erase data.
    if fs::metadata(backup)?.len() < 100 {
        return Err(
            DbError::Integrity("restore source has no complete SQLite header".into()).into(),
        );
    }
    let source = read_only(backup)?;
    check_integrity(&source)?;
    let found: u32 = source
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(DbError::from)?;
    let supported = migrations().last().map_or(0, |migration| migration.version);
    if found > supported {
        return Err(DbError::NewerSchema { found, supported }.into());
    }
    if found == 0 {
        let has_tables: bool = source
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%')",
                [],
                |row| row.get(0),
            )
            .map_err(DbError::from)?;
        if has_tables {
            return Err(DbError::NotBaytarDatabase.into());
        }
    }

    // Stage on the target filesystem so the final rename is atomic. Copy the
    // snapshot verbatim to preserve rowids as well as explicitly keyed rows.
    let staging = staging_dir(parent_dir(db_path))?;
    let partial = staging.path().join("restore.partial");
    log::info!("staging verified restore snapshot");
    // Do not inherit a read-only source file's permissions on the writable target.
    let mut source_file = File::open(backup)?;
    let mut staged_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)?;
    std::io::copy(&mut source_file, &mut staged_file)?;
    staged_file.sync_all()?;
    drop(staged_file);
    drop(source_file);
    check_integrity(&read_only(&partial)?)?;
    drop(source);

    log::info!("creating pre-restore backup of {}", db_path.display());
    let current = read_only(db_path)?;
    let previous = create_at(&current, dir, BackupKind::PreRestore, Utc::now())?;
    drop(current);

    log::info!("replacing database {}", db_path.display());
    fs::rename(&partial, db_path)?;
    for suffix in ["-wal", "-shm"] {
        let mut name = db_path.as_os_str().to_os_string();
        name.push(suffix);
        let sidecar = PathBuf::from(name);
        log::debug!("removing stale sidecar {}", sidecar.display());
        match fs::remove_file(sidecar) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    log::info!("opening restored database and applying pending migrations");
    let restored = Database::open(db_path, dir)?;
    check_integrity(restored.conn())?;
    log::info!("restore completed and verified");
    Ok(previous)
}

fn read_only(path: &Path) -> Result<Connection, DbError> {
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn staging_dir(dir: &Path) -> Result<tempfile::TempDir, std::io::Error> {
    tempfile::Builder::new()
        .prefix(".baytar-")
        .suffix(".partial")
        .tempdir_in(dir)
}

// Reserve the name across concurrent writers; the actual SQLite snapshot lives
// in a private staging directory. A crash can leave only ignored .partial paths.
struct Reservation {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Err(error) = fs::remove_file(&self.path) {
            log::warn!("could not remove backup reservation: {error}");
        }
    }
}

fn create_at(
    conn: &Connection,
    dir: &Path,
    kind: BackupKind,
    now: DateTime<Utc>,
) -> Result<BackupInfo, BackupError> {
    fs::create_dir_all(dir)?;
    let name = format!(
        "baytar-{}-{}.db",
        kind.as_str(),
        now.format("%Y%m%d-%H%M%SZ")
    );
    let path = dir.join(&name);
    let reservation_path = dir.join(format!("{name}.partial"));
    let reservation_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&reservation_path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                BackupError::NameCollision(path.clone())
            } else {
                error.into()
            }
        })?;
    let _reservation = Reservation {
        path: reservation_path,
        file: Some(reservation_file),
    };
    if path.try_exists()? {
        return Err(BackupError::NameCollision(path));
    }
    log::info!("creating {} backup {}", kind.as_str(), path.display());
    let staging = staging_dir(dir)?;
    let partial = staging.path().join("backup.partial");
    backup_to(conn, &partial)?;
    let file = OpenOptions::new().write(true).open(&partial)?;
    file.sync_all()?;
    let size_bytes = file.metadata()?.len();
    drop(file);
    fs::rename(&partial, &path)?;
    log::info!("backup published: {}", path.display());
    Ok(BackupInfo {
        path,
        created_at: now.with_nanosecond(0).unwrap_or(now),
        kind,
        size_bytes,
    })
}

fn parse_name(name: &str) -> Option<(BackupKind, DateTime<Utc>)> {
    let name = name.strip_prefix("baytar-")?.strip_suffix(".db")?;
    for kind in [
        BackupKind::Daily,
        BackupKind::Manual,
        BackupKind::PreMigration,
        BackupKind::PreRestore,
        BackupKind::PreImport,
        BackupKind::PreUpdate,
    ] {
        let Some(timestamp) = name
            .strip_prefix(kind.as_str())
            .and_then(|rest| rest.strip_prefix('-'))
        else {
            continue;
        };
        let parsed = NaiveDateTime::parse_from_str(timestamp, "%Y%m%d-%H%M%SZ").ok()?;
        if parsed.format("%Y%m%d-%H%M%SZ").to_string() != timestamp {
            return None;
        }
        return Some((kind, parsed.and_utc()));
    }
    None
}
