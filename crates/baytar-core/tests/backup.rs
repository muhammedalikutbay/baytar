use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use baytar_core::backup::{self, BackupError, BackupKind, BackupPolicy};
use baytar_core::db::{check_integrity, migrations, Database, DbError};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, OpenFlags};

fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

fn database(path: &Path, dir: &Path) -> Database {
    let db = Database::open(path, dir).unwrap();
    db.conn()
        .execute_batch(
            "CREATE TABLE backup_fixture (id INTEGER PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO backup_fixture VALUES (1, 'Synthetic first'), (7, 'Synthetic second');",
        )
        .unwrap();
    db
}

fn rows(conn: &Connection) -> Vec<(i64, String)> {
    conn.prepare("SELECT id, value FROM backup_fixture ORDER BY id")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn read_only(path: &Path) -> Connection {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap()
}

fn synthetic_file(dir: &Path, kind: BackupKind, at: DateTime<Utc>) -> PathBuf {
    let path = dir.join(format!(
        "baytar-{}-{}.db",
        kind.as_str(),
        at.format("%Y%m%d-%H%M%SZ")
    ));
    fs::write(&path, b"synthetic retention fixture").unwrap();
    path
}

#[test]
fn daily_backup_is_created_once_per_utc_date() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = backup::backups_dir(tmp.path());
    assert_eq!(dir, tmp.path().join("backups"));
    let db = database(&tmp.path().join("clinic.db"), &dir);
    let now = time("2026-11-04T23:59:59.123Z");
    let first = backup::daily_if_due(&db, &dir, now).unwrap().unwrap();
    assert_eq!(first.kind, BackupKind::Daily);
    assert_eq!(first.created_at, time("2026-11-04T23:59:59Z"));
    assert_eq!(
        first.path.file_name().unwrap(),
        "baytar-daily-20261104-235959Z.db"
    );
    assert!(backup::daily_if_due(&db, &dir, now).unwrap().is_none());
    assert!(backup::daily_if_due(&db, &dir, now - Duration::hours(12))
        .unwrap()
        .is_none());
    assert!(backup::daily_if_due(&db, &dir, now + Duration::seconds(1))
        .unwrap()
        .is_some());
    assert_eq!(rows(&read_only(&first.path)), rows(db.conn()));
    assert_eq!(backup::list(&dir).unwrap().len(), 2);
}

#[test]
fn all_kinds_are_verified_listed_and_exported_without_partial_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = backup::backups_dir(tmp.path());
    let db = database(&tmp.path().join("clinic.db"), &dir);
    for kind in [
        BackupKind::Daily,
        BackupKind::Manual,
        BackupKind::PreMigration,
        BackupKind::PreRestore,
        BackupKind::PreImport,
        BackupKind::PreUpdate,
    ] {
        let info = backup::create(&db, &dir, kind).unwrap();
        assert!(info.size_bytes > 0);
        assert!(backup::list(&dir).unwrap().contains(&info));
        check_integrity(&read_only(&info.path)).unwrap();
    }
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 6);
    let export = backup::export_copy(&db, &tmp.path().join("external")).unwrap();
    assert_eq!(rows(&read_only(&export)), rows(db.conn()));
    assert_eq!(backup::list(&dir).unwrap().len(), 6);
}

#[test]
fn list_ignores_junk_partial_paths_and_invalid_timestamps_and_sorts_newest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let oldest = synthetic_file(dir, BackupKind::PreMigration, time("2025-12-31T23:59:59Z"));
    let newest = synthetic_file(dir, BackupKind::Manual, time("2026-11-04T12:00:00Z"));
    for name in [
        "unrelated.db",
        "baytar-unknown-20261104-120000Z.db",
        "baytar-daily-20261104-120000Z.db.partial",
        "baytar-daily-20260230-120000Z.db",
        "baytar-daily-20261104-120000Z.db.extra",
        "baytar-daily-20261104-1200Z.db",
        "baytar-daily-20261104-120000.db",
    ] {
        fs::write(dir.join(name), b"junk").unwrap();
    }
    fs::create_dir(dir.join("baytar-daily-20261104-120000Z.db")).unwrap();
    let listed = backup::list(dir).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].path, newest);
    assert_eq!(listed[1].path, oldest);
    assert_eq!(listed[0].size_bytes, fs::metadata(newest).unwrap().len());
    assert!(backup::list(&dir.join("missing")).unwrap().is_empty());
}

#[test]
fn prune_retains_daily_dates_and_weekly_snapshots_without_touching_other_files() {
    let now = time("2026-02-23T12:00:00Z"); // Monday, ISO week 9.
    for (policy, expected_count) in [
        (
            BackupPolicy {
                keep_daily: 14,
                keep_weekly: 0,
            },
            14,
        ),
        (
            BackupPolicy {
                keep_daily: 0,
                keep_weekly: 8,
            },
            8,
        ),
        // This Monday's 14 daily dates span three ISO weeks, overlapping weekly retention.
        (BackupPolicy::default(), 19),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        for age in 0..100 {
            synthetic_file(tmp.path(), BackupKind::Daily, now - Duration::days(age));
        }
        let mut protected = Vec::new();
        for kind in [
            BackupKind::Manual,
            BackupKind::PreMigration,
            BackupKind::PreRestore,
            BackupKind::PreImport,
            BackupKind::PreUpdate,
        ] {
            protected.push(synthetic_file(tmp.path(), kind, now - Duration::days(500)));
        }
        let junk = tmp.path().join("baytar-daily-20200101-120000Z.db.partial");
        fs::write(&junk, b"incomplete").unwrap();
        let removed = backup::prune(tmp.path(), &policy, now).unwrap();
        assert_eq!(removed.len(), 100 - expected_count);
        assert!(removed.iter().all(|path| !path.exists()));
        let remaining = backup::list(tmp.path()).unwrap();
        assert_eq!(remaining.len(), expected_count + 5);
        assert!(protected.iter().all(|path| path.exists()));
        assert!(junk.exists());
        if policy.keep_daily == 14 {
            for age in 0..14 {
                let at = now - Duration::days(age);
                assert!(remaining.iter().any(|info| info.created_at == at));
            }
        }
        if policy.keep_weekly == 8 {
            for age in [0, 1, 8, 15, 22, 29, 36, 43] {
                let at = now - Duration::days(age);
                assert!(remaining.iter().any(|info| info.created_at == at));
            }
        }
    }
}

#[test]
fn weekly_retention_uses_iso_year_and_preserves_future_backups() {
    let tmp = tempfile::tempdir().unwrap();
    let now = time("2026-01-05T12:00:00Z");
    let mut expected = HashSet::new();
    for date in [
        "2026-01-05T12:00:00Z",
        "2026-01-04T12:00:00Z",
        "2025-12-28T12:00:00Z",
        "2026-01-06T12:00:00Z",
    ] {
        expected.insert(synthetic_file(tmp.path(), BackupKind::Daily, time(date)));
    }
    let duplicate_week =
        synthetic_file(tmp.path(), BackupKind::Daily, time("2025-12-29T12:00:00Z"));
    let too_old = synthetic_file(tmp.path(), BackupKind::Daily, time("2025-12-21T12:00:00Z"));
    backup::prune(
        tmp.path(),
        &BackupPolicy {
            keep_daily: 0,
            keep_weekly: 3,
        },
        now,
    )
    .unwrap();
    assert!(!duplicate_week.exists());
    assert!(!too_old.exists());
    assert_eq!(
        backup::list(tmp.path())
            .unwrap()
            .into_iter()
            .map(|info| info.path)
            .collect::<HashSet<_>>(),
        expected
    );
}

#[test]
fn daily_retention_keeps_all_snapshots_on_retained_dates() {
    let tmp = tempfile::tempdir().unwrap();
    let now = time("2026-11-04T12:00:00Z");
    synthetic_file(tmp.path(), BackupKind::Daily, now);
    synthetic_file(tmp.path(), BackupKind::Daily, now - Duration::hours(1));
    synthetic_file(tmp.path(), BackupKind::Daily, now - Duration::days(1));
    let old = synthetic_file(tmp.path(), BackupKind::Daily, now - Duration::days(2));
    backup::prune(
        tmp.path(),
        &BackupPolicy {
            keep_daily: 2,
            keep_weekly: 0,
        },
        now,
    )
    .unwrap();
    assert!(!old.exists());
    assert_eq!(backup::list(tmp.path()).unwrap().len(), 3);
}

#[test]
fn restore_recovers_exact_rows_and_preserves_current_rows_in_pre_restore_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    let db = database(&path, &dir);
    let original = rows(db.conn());
    let snapshot = backup::create(&db, &dir, BackupKind::Manual).unwrap();
    db.conn()
        .execute_batch(
            "DELETE FROM backup_fixture WHERE id = 1;
        UPDATE backup_fixture SET value = 'Synthetic changed' WHERE id = 7;
        INSERT INTO backup_fixture VALUES (42, 'Synthetic new');",
        )
        .unwrap();
    let changed = rows(db.conn());
    drop(db);
    let previous = backup::restore(&path, &snapshot.path, &dir).unwrap();
    assert!(!tmp.path().join("clinic.db-wal").exists());
    assert!(!tmp.path().join("clinic.db-shm").exists());
    assert_eq!(previous.kind, BackupKind::PreRestore);
    assert!(previous.path.exists());
    assert_eq!(rows(&read_only(&previous.path)), changed);
    assert_eq!(rows(&read_only(&path)), original);
    check_integrity(&read_only(&path)).unwrap();
    assert_eq!(backup::list(&dir).unwrap().len(), 2);
}

#[test]
fn corrupted_restore_changes_nothing_and_creates_no_pre_restore_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    let db = database(&path, &dir);
    let snapshot = backup::create(&db, &dir, BackupKind::Manual).unwrap();
    let corrupted = tmp.path().join("corrupted.db");
    fs::copy(&snapshot.path, &corrupted).unwrap();
    db.conn()
        .execute(
            "INSERT INTO backup_fixture VALUES (99, 'Synthetic current')",
            [],
        )
        .unwrap();
    let expected = rows(db.conn());
    drop(db);
    let before = fs::read(&path).unwrap();
    for length in [100, 0] {
        fs::OpenOptions::new()
            .write(true)
            .open(&corrupted)
            .unwrap()
            .set_len(length)
            .unwrap();
        let corrupt_before = fs::read(&corrupted).unwrap();
        assert!(backup::restore(&path, &corrupted, &dir).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read(&corrupted).unwrap(), corrupt_before);
    }
    assert_eq!(rows(&read_only(&path)), expected);
    assert_eq!(backup::list(&dir).unwrap(), vec![snapshot]);
}

#[test]
fn newer_schema_restore_returns_engine_error_without_changing_target() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    drop(database(&path, &dir));
    let newer = tmp.path().join("newer.db");
    let supported = migrations().last().unwrap().version;
    let conn = Connection::open(&newer).unwrap();
    conn.pragma_update(None, "user_version", supported + 1)
        .unwrap();
    drop(conn);
    let before = fs::read(&path).unwrap();
    let newer_before = fs::read(&newer).unwrap();
    let error = backup::restore(&path, &newer, &dir).unwrap_err();
    assert!(
        matches!(error, BackupError::Database(DbError::NewerSchema { found, supported: version }) if found == supported + 1 && version == supported)
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read(&newer).unwrap(), newer_before);
    assert!(backup::list(&dir).unwrap().is_empty());
}

#[test]
fn older_empty_database_is_migrated_after_restore() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    drop(database(&path, &dir));
    let older = tmp.path().join("older.db");
    Connection::open(&older)
        .unwrap()
        .execute_batch("VACUUM")
        .unwrap();
    backup::restore(&path, &older, &dir).unwrap();
    let restored = Database::open(&path, &dir).unwrap();
    assert_eq!(
        restored.schema_version().unwrap(),
        migrations().last().unwrap().version
    );
    check_integrity(restored.conn()).unwrap();
    assert_eq!(
        read_only(&older)
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        0
    );
}

#[test]
fn foreign_database_restore_is_refused_before_changing_target() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    drop(database(&path, &dir));
    let foreign = tmp.path().join("foreign.db");
    Connection::open(&foreign)
        .unwrap()
        .execute_batch("CREATE TABLE legacy_fixture (id INTEGER PRIMARY KEY)")
        .unwrap();
    let before = fs::read(&path).unwrap();
    assert!(matches!(
        backup::restore(&path, &foreign, &dir),
        Err(BackupError::Database(DbError::NotBaytarDatabase))
    ));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(backup::list(&dir).unwrap().is_empty());
}

#[test]
fn failed_snapshot_is_not_published_and_cleans_up_partial_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = backup::backups_dir(tmp.path());
    let db = database(&tmp.path().join("clinic.db"), &dir);
    // VACUUM INTO cannot run inside a transaction.
    db.conn().execute_batch("BEGIN").unwrap();
    assert!(backup::create(&db, &dir, BackupKind::Manual).is_err());
    db.conn().execute_batch("ROLLBACK").unwrap();
    assert!(backup::list(&dir).unwrap().is_empty());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

#[test]
fn in_progress_snapshot_is_never_overwritten_or_deleted() {
    let tmp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let now = time("2026-11-04T12:00:00Z");
    let partial = tmp.path().join("baytar-daily-20261104-120000Z.db.partial");
    fs::write(&partial, b"another writer's reservation").unwrap();
    assert!(matches!(
        backup::daily_if_due(&db, tmp.path(), now),
        Err(BackupError::NameCollision(_))
    ));
    assert_eq!(fs::read(&partial).unwrap(), b"another writer's reservation");
    assert!(backup::list(tmp.path()).unwrap().is_empty());
}

#[test]
fn read_only_restore_source_produces_a_writable_database() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("clinic.db");
    let dir = backup::backups_dir(tmp.path());
    let db = database(&path, &dir);
    let snapshot = backup::create(&db, &dir, BackupKind::Manual).unwrap();
    drop(db);
    let original_permissions = fs::metadata(&snapshot.path).unwrap().permissions();
    let mut read_only_permissions = original_permissions.clone();
    read_only_permissions.set_readonly(true);
    fs::set_permissions(&snapshot.path, read_only_permissions).unwrap();
    let result = backup::restore(&path, &snapshot.path, &dir);
    fs::set_permissions(&snapshot.path, original_permissions).unwrap();
    result.unwrap();
    let restored = Database::open(&path, &dir).unwrap();
    restored
        .conn()
        .execute(
            "INSERT INTO backup_fixture VALUES (99, 'Synthetic writable')",
            [],
        )
        .unwrap();
    assert_eq!(rows(restored.conn()).len(), 3);
}
