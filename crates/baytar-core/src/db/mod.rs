//! Database schema, migrations, and repositories.
//!
//! Safety rules for every open:
//! * a database written by a newer app is never touched,
//! * pending migrations on a database with data take a verified backup first,
//! * all pending migrations run in one transaction and are integrity-checked before commit.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, TransactionBehavior};

#[cfg(test)]
mod tests;

/// One schema step. `version` must equal its 1-based position in the list.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

static MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "init",
    sql: include_str!("migrations/0001_init.sql"),
}];

/// Ordered list of all known migrations.
pub fn migrations() -> &'static [Migration] {
    MIGRATIONS
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database schema version {found} is newer than this app supports ({supported})")]
    NewerSchema { found: u32, supported: u32 },
    #[error("migration to version {version} failed: {source}")]
    MigrationFailed {
        version: u32,
        #[source]
        source: Box<DbError>,
    },
    #[error("integrity check failed: {0}")]
    Integrity(String),
    #[error("foreign key check failed: {0}")]
    ForeignKey(String),
    #[error("backup failed: {0}")]
    Backup(String),
    #[error("invalid migration list: {0}")]
    InvalidMigrations(String),
    #[error("not a Baytar database: it has tables but no schema version")]
    NotBaytarDatabase,
}

pub struct Database {
    conn: Connection,
    path: Option<PathBuf>,
    #[allow(dead_code)]
    backups_dir: Option<PathBuf>,
}

impl Database {
    /// Opens (or creates) the database file and applies pending migrations safely.
    pub fn open(path: &Path, backups_dir: &Path) -> Result<Database, DbError> {
        Self::open_with(path, backups_dir, migrations())
    }

    /// Fresh in-memory database with all migrations applied (tests, previews).
    pub fn open_in_memory() -> Result<Database, DbError> {
        let conn = Connection::open_in_memory()?;
        configure(&conn)?;
        let list = migrations();
        validate(list)?;
        apply(&conn, list, 0)?;
        Ok(Database {
            conn,
            path: None,
            backups_dir: None,
        })
    }

    #[doc(hidden)]
    pub fn open_with(
        path: &Path,
        backups_dir: &Path,
        list: &[Migration],
    ) -> Result<Database, DbError> {
        validate(list)?;
        let supported = list.last().map_or(0, |m| m.version);
        let conn = Connection::open(path)?;
        // Read the version before any pragma that could modify the file header.
        let found = read_version(&conn)?;
        if found > supported {
            log::error!("database at schema v{found} is newer than supported v{supported}");
            return Err(DbError::NewerSchema { found, supported });
        }
        if found == 0 && has_user_tables(&conn)? {
            // A foreign SQLite file (e.g. a legacy PatientsCare database) must never be migrated in place.
            log::error!("refusing to migrate a database with tables but no schema version");
            return Err(DbError::NotBaytarDatabase);
        }
        configure(&conn)?;
        if found < supported {
            if found > 0 {
                let dest = backup_file_name(found, supported, backups_dir);
                log::info!("pre-migration backup: v{found} -> v{supported}");
                std::fs::create_dir_all(backups_dir)?;
                backup_to(&conn, &dest)?;
            } else {
                log::info!("new database, applying {supported} migration(s)");
            }
            apply(&conn, list, found)?;
        } else {
            log::info!("database schema v{found} is current");
        }
        Ok(Database {
            conn,
            path: Some(path.to_path_buf()),
            backups_dir: Some(backups_dir.to_path_buf()),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn schema_version(&self) -> Result<u32, DbError> {
        read_version(&self.conn)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

fn read_version(conn: &Connection) -> Result<u32, DbError> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

fn has_user_tables(conn: &Connection) -> Result<bool, DbError> {
    let n: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

fn configure(conn: &Connection) -> Result<(), DbError> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

fn validate(list: &[Migration]) -> Result<(), DbError> {
    for (i, m) in list.iter().enumerate() {
        let expected = u32::try_from(i + 1).unwrap_or(u32::MAX);
        if m.version != expected {
            return Err(DbError::InvalidMigrations(format!(
                "migration '{}' has version {} but position {}",
                m.name, m.version, expected
            )));
        }
    }
    Ok(())
}

fn backup_file_name(from: u32, to: u32, dir: &Path) -> PathBuf {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let base = format!("pre-migration-v{from}-to-v{to}-{ts}");
    let mut candidate = dir.join(format!("{base}.db"));
    let mut n = 1;
    while candidate.exists() {
        candidate = dir.join(format!("{base}-{n}.db"));
        n += 1;
    }
    candidate
}

/// Runs all migrations newer than `from` in one transaction; rolls back on any failure.
fn apply(conn: &Connection, list: &[Migration], from: u32) -> Result<(), DbError> {
    let target = list.last().map_or(0, |m| m.version);
    let result = (|| -> Result<(), (u32, DbError)> {
        let tx = rusqlite::Transaction::new_unchecked(conn, TransactionBehavior::Immediate)
            .map_err(|e| (from + 1, e.into()))?;
        for m in list.iter().filter(|m| m.version > from) {
            log::info!("applying migration {:04}_{}", m.version, m.name);
            tx.execute_batch(m.sql).map_err(|e| (m.version, e.into()))?;
        }
        tx.pragma_update(None, "user_version", target)
            .map_err(|e| (target, e.into()))?;
        check_integrity(&tx).map_err(|e| (target, e))?;
        tx.commit().map_err(|e| (target, e.into()))?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            log::info!("migrated v{from} -> v{target}");
            Ok(())
        }
        Err((version, source)) => {
            log::error!("migration failed at v{version} (rolled back): {source}");
            Err(DbError::MigrationFailed {
                version,
                source: Box::new(source),
            })
        }
    }
}

/// Writes a consistent copy with `VACUUM INTO` and verifies it read-only.
/// `dest` must not exist. A copy that fails verification is removed.
pub fn backup_to(conn: &Connection, dest: &Path) -> Result<(), DbError> {
    let dest_str = dest
        .to_str()
        .ok_or_else(|| DbError::Backup("destination path is not valid UTF-8".into()))?;
    if dest.exists() {
        return Err(DbError::Backup(format!(
            "destination already exists: {}",
            dest.display()
        )));
    }
    conn.execute("VACUUM INTO ?1", [dest_str])?;
    let verify = Connection::open_with_flags(dest, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(DbError::from)
        .and_then(|c| check_integrity(&c));
    if let Err(e) = verify {
        log::error!(
            "backup verification failed, removing {}: {e}",
            dest.display()
        );
        let _ = std::fs::remove_file(dest);
        return Err(DbError::Backup(format!("verification failed: {e}")));
    }
    log::info!("backup written and verified: {}", dest.display());
    Ok(())
}

/// `PRAGMA integrity_check` followed by `PRAGMA foreign_key_check`.
pub fn check_integrity(conn: &Connection) -> Result<(), DbError> {
    let mut stmt = conn.prepare("PRAGMA integrity_check")?;
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    if rows.len() != 1 || rows[0] != "ok" {
        return Err(DbError::Integrity(rows.join("; ")));
    }
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let violations: Vec<String> = stmt
        .query_map([], |r| {
            let table: String = r.get(0)?;
            let rowid: Option<i64> = r.get(1)?;
            let parent: String = r.get(2)?;
            Ok(format!("{table} row {rowid:?} -> {parent}"))
        })?
        .collect::<Result<_, _>>()?;
    if !violations.is_empty() {
        return Err(DbError::ForeignKey(violations.join("; ")));
    }
    Ok(())
}
