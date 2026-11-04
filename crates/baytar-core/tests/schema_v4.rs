//! Schema v4 migration: a populated v3 database gains `reminder_contacts` with every row intact.
//! All data is synthetic.

use std::path::{Path, PathBuf};

use baytar_core::db::{check_integrity, migrations, Database, Migration};
use rusqlite::{types::Value, Connection, OpenFlags};

fn v3() -> &'static [Migration] {
    &migrations()[..3]
}

fn backups(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(dir) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).collect(),
        Err(_) => vec![],
    }
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn snapshot(conn: &Connection, table: &str) -> Vec<Vec<Value>> {
    let n = count(
        conn,
        &format!("SELECT count(*) FROM pragma_table_info('{table}')"),
    ) as usize;
    let mut stmt = conn
        .prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
        .unwrap();
    let rows = stmt
        .query_map([], |r| {
            (0..n)
                .map(|i| r.get::<_, Value>(i))
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows
}

const TABLES: &[&str] = &[
    "owners",
    "patients",
    "visits",
    "vaccine_types",
    "vaccinations",
    "medications",
    "services",
    "charges",
    "payments",
    "appointments",
    "settings",
];

const V3_SEED: &str = "
INSERT INTO owners (id, full_name, phone, phone_raw) VALUES
 (1, 'Synthetic Owner', '+905551112233', '0555 111 22 33'),
 (2, 'Synthetic Debtor', NULL, NULL);
INSERT INTO patients (id, owner_id, name, species) VALUES (1, 1, 'Synthetic Cat', 'cat');
INSERT INTO visits (id, patient_id, visit_on, reason) VALUES (1, 1, '2026-10-01', 'checkup');
INSERT INTO vaccinations (id, patient_id, vaccine_name, given_on, next_due_on) VALUES (1, 1, 'Synthetic Vaccine', '2025-11-01', '2026-11-01');
INSERT INTO appointments (id, patient_id, owner_id, title, starts_on, kind, status) VALUES (1, 1, 1, 'Control', '2026-11-12', 'visit', 'planned');
INSERT INTO charges (id, owner_id, description, quantity, unit_price_minor, total_minor, charged_on) VALUES (1, 2, 'Old charge', 1, 5000, 5000, '2026-08-01');
INSERT INTO settings (key, value) VALUES ('clinic_name', 'Synthetic Clinic');
";

#[test]
fn populated_v3_database_upgrades_to_v4_with_data_intact() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    {
        let db = Database::open_with(&path, &bdir, v3()).unwrap();
        assert_eq!(db.schema_version().unwrap(), 3);
        db.conn().execute_batch(V3_SEED).unwrap();
    }
    let before: Vec<_> = {
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            count(
                &conn,
                "SELECT count(*) FROM sqlite_schema WHERE name = 'reminder_contacts'"
            ),
            0
        );
        TABLES.iter().map(|t| (*t, snapshot(&conn, t))).collect()
    };
    assert!(backups(&bdir).is_empty());

    let db = Database::open(&path, &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 4);
    assert_eq!(migrations().len(), 4);
    assert_eq!(migrations()[3].name, "reminders");
    let conn = db.conn();
    for (table, rows) in &before {
        assert_eq!(&snapshot(conn, table), rows, "table {table} changed");
    }

    // The new table is STRICT, empty and indexed.
    assert_eq!(
        count(
            conn,
            "SELECT strict FROM pragma_table_list WHERE name = 'reminder_contacts'"
        ),
        1
    );
    assert_eq!(count(conn, "SELECT count(*) FROM reminder_contacts"), 0);
    assert_eq!(
        count(
            conn,
            "SELECT count(*) FROM sqlite_schema WHERE type = 'index' AND name IN ('reminder_contacts_ref_key_idx', 'reminder_contacts_owner_id_idx')"
        ),
        2
    );
    let plan: String = conn
        .query_row(
            "EXPLAIN QUERY PLAN SELECT id FROM reminder_contacts WHERE ref_key = 'debt:2:2026-08-01'",
            [],
            |r| r.get(3),
        )
        .unwrap();
    assert!(plan.contains("reminder_contacts_ref_key_idx"), "{plan}");

    // Constraints hold, and the table works with the migrated rows.
    for sql in [
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('sms', 'sms:1', 1, 'phone')",
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('debt', 'debt:1', 1, 'pigeon')",
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('debt', '  ', 1, 'phone')",
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('debt', 'debt:99', 99, 'phone')",
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('debt', 'debt:1', NULL, 'phone')",
    ] {
        assert!(conn.execute(sql, []).is_err(), "should be rejected: {sql}");
    }
    conn.execute(
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel, note) VALUES ('debt', 'debt:2:2026-08-01', 2, 'whatsapp', 'synthetic')",
        [],
    )
    .unwrap();
    let stamp: String = conn
        .query_row("SELECT contacted_at FROM reminder_contacts", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(stamp.ends_with('Z') && stamp.len() == 24, "{stamp}");
    assert!(conn.execute("DELETE FROM owners WHERE id = 2", []).is_err());
    check_integrity(conn).unwrap();
    assert_eq!(
        count(conn, "SELECT count(*) FROM pragma_foreign_key_check"),
        0
    );

    // The pre-migration backup is the untouched v3 file.
    let b = backups(&bdir);
    assert_eq!(b.len(), 1);
    let name = b[0].file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("pre-migration-v3-to-v4-"), "{name}");
    let old = Connection::open_with_flags(&b[0], OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(count(&old, "PRAGMA user_version"), 3);
    assert_eq!(
        count(
            &old,
            "SELECT count(*) FROM sqlite_schema WHERE name = 'reminder_contacts'"
        ),
        0
    );
    assert_eq!(snapshot(&old, "owners"), before[0].1);
    check_integrity(&old).unwrap();

    // Reopening is a no-op.
    drop(db);
    let db = Database::open(&path, &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 4);
    assert_eq!(backups(&bdir).len(), 1);
    assert_eq!(
        count(db.conn(), "SELECT count(*) FROM reminder_contacts"),
        1
    );
}

#[test]
fn fresh_database_reaches_v4() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(db.schema_version().unwrap(), 4);
    assert_eq!(
        count(
            db.conn(),
            "SELECT count(*) FROM pragma_table_info('reminder_contacts')"
        ),
        7
    );
}
