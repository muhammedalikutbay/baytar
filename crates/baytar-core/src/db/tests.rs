use super::*;

const M1: Migration = Migration {
    version: 1,
    name: "init",
    sql: "CREATE TABLE owner (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
          CREATE TABLE pet (id INTEGER PRIMARY KEY, owner_id INTEGER NOT NULL REFERENCES owner(id));",
};
const M2_OK: Migration = Migration {
    version: 2,
    name: "add_phone",
    sql: "ALTER TABLE owner ADD COLUMN phone TEXT;",
};
const M2_BAD: Migration = Migration {
    version: 2,
    name: "bad",
    sql: "ALTER TABLE owner ADD COLUMN phone TEXT; THIS IS NOT SQL;",
};
const M2_FK: Migration = Migration {
    version: 2,
    name: "fk_violation",
    sql: "PRAGMA defer_foreign_keys = ON; INSERT INTO pet (owner_id) VALUES (999);",
};

fn seed(path: &Path, dir: &Path) {
    let db = Database::open_with(path, dir, &[M1]).unwrap();
    db.conn()
        .execute("INSERT INTO owner (name) VALUES ('Synthetic Owner')", [])
        .unwrap();
}

fn backups(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(dir) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).collect(),
        Err(_) => vec![],
    }
}

fn owner_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM owner", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn new_file_gets_all_migrations_without_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let bdir = tmp.path().join("backups");
    let db = Database::open(&tmp.path().join("a.db"), &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), migrations().len() as u32);
    assert!(db.path().is_some());
    assert!(backups(&bdir).is_empty());
    let fk: i64 = db
        .conn()
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}

#[test]
fn in_memory_applies_all_migrations() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(db.schema_version().unwrap(), migrations().len() as u32);
    assert!(db.path().is_none());
}

#[test]
fn upgrade_takes_backup_and_keeps_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed(&path, &bdir);
    assert!(backups(&bdir).is_empty());

    let db = Database::open_with(&path, &bdir, &[M1, M2_OK]).unwrap();
    assert_eq!(db.schema_version().unwrap(), 2);
    assert_eq!(owner_count(db.conn()), 1);
    db.conn().prepare("SELECT phone FROM owner").unwrap();

    let b = backups(&bdir);
    assert_eq!(b.len(), 1);
    let name = b[0].file_name().unwrap().to_str().unwrap().to_string();
    assert!(name.starts_with("pre-migration-v1-to-v2-"), "{name}");
    let old = Connection::open_with_flags(&b[0], OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(owner_count(&old), 1);
    assert_eq!(read_version(&old).unwrap(), 1);
    assert!(old.prepare("SELECT phone FROM owner").is_err());
}

#[test]
fn failing_migration_rolls_back_and_keeps_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed(&path, &bdir);

    let err = Database::open_with(&path, &bdir, &[M1, M2_BAD])
        .err()
        .unwrap();
    assert!(
        matches!(err, DbError::MigrationFailed { version: 2, .. }),
        "{err}"
    );

    let conn = Connection::open(&path).unwrap();
    assert_eq!(read_version(&conn).unwrap(), 1);
    assert_eq!(owner_count(&conn), 1);
    assert!(conn.prepare("SELECT phone FROM owner").is_err());
    assert_eq!(backups(&bdir).len(), 1);
}

#[test]
fn newer_schema_is_refused_and_file_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE t (x); PRAGMA user_version = 99;")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    let err = Database::open(&path, &bdir).err().unwrap();
    assert!(matches!(
        err,
        DbError::NewerSchema {
            found: 99,
            supported: 1
        }
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(backups(&bdir).is_empty());
}

#[test]
fn foreign_key_violation_rolls_back() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed(&path, &bdir);

    let err = Database::open_with(&path, &bdir, &[M1, M2_FK])
        .err()
        .unwrap();
    assert!(matches!(err, DbError::MigrationFailed { .. }), "{err}");

    let conn = Connection::open(&path).unwrap();
    assert_eq!(read_version(&conn).unwrap(), 1);
    let pets: i64 = conn
        .query_row("SELECT count(*) FROM pet", [], |r| r.get(0))
        .unwrap();
    assert_eq!(pets, 0);
}

#[test]
fn backup_to_produces_valid_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed(&path, &bdir);
    let db = Database::open_with(&path, &bdir, &[M1]).unwrap();
    let dest = tmp.path().join("copy.db");
    backup_to(db.conn(), &dest).unwrap();
    let copy = Connection::open(&dest).unwrap();
    check_integrity(&copy).unwrap();
    assert_eq!(owner_count(&copy), 1);
    // Refuses to overwrite an existing file.
    assert!(backup_to(db.conn(), &dest).is_err());
}

#[test]
fn invalid_migration_list_is_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let err = Database::open_with(&tmp.path().join("a.db"), tmp.path(), &[M2_OK])
        .err()
        .unwrap();
    assert!(matches!(err, DbError::InvalidMigrations(_)));
}

#[test]
fn foreign_sqlite_file_is_refused_and_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    {
        let c = Connection::open(&path).unwrap();
        c.execute_batch("CREATE TABLE PatientOwner (Id INTEGER PRIMARY KEY, OwnerName TEXT);")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    let err = Database::open_with(&path, &dir.path().join("backups"), &[M1, M2_OK]).err();
    assert!(matches!(err, Some(DbError::NotBaytarDatabase)));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
