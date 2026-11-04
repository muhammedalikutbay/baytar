//! Schema v2 migration: fresh creation, upgrade of a populated v1 database, atomicity.
//! All data is synthetic.

use std::path::{Path, PathBuf};

use baytar_core::db::{check_integrity, migrations, Database, DbError, Migration};
use rusqlite::{types::Value, Connection, OpenFlags};

fn v1() -> &'static [Migration] {
    &migrations()[..1]
}

fn backups(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(dir) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).collect(),
        Err(_) => vec![],
    }
}

fn version(conn: &Connection) -> u32 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

fn snapshot(conn: &Connection, table: &str, columns: &str) -> Vec<Vec<Value>> {
    let n = columns.split(',').count();
    let mut stmt = conn
        .prepare(&format!("SELECT {columns} FROM {table} ORDER BY 1"))
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

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

type PhoneRow = (i64, Option<String>, Option<String>, Option<String>);

fn phones(conn: &Connection) -> Vec<PhoneRow> {
    let mut stmt = conn
        .prepare("SELECT id, phone, phone_alt, phone_raw FROM owners ORDER BY id")
        .unwrap();
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows
}

/// Columns of each v1 table, in v1 order (new v2 columns are excluded on purpose).
const V1_TABLES: &[(&str, &str)] = &[
    (
        "owners",
        "id, full_name, phone_raw, email, address, notes, search_text, created_at, updated_at, deleted_at",
    ),
    (
        "patients",
        "id, owner_id, name, species, species_note, breed, sex, neutered, birth_date, birth_date_is_estimate, color, microchip, notes, deceased_on, search_text, created_at, updated_at, deleted_at",
    ),
    (
        "visits",
        "id, patient_id, visit_on, visit_time, reason, findings, diagnosis, treatment, weight_kg, temperature_c, notes, created_at, updated_at, deleted_at",
    ),
    (
        "vaccine_types",
        "id, name, species, default_interval_days, notes, is_active, created_at, updated_at, deleted_at",
    ),
    (
        "vaccinations",
        "id, patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on, batch_no, visit_id, notes, created_at, updated_at, deleted_at",
    ),
    (
        "medications",
        "id, name, active_ingredient, form, concentration_mg_per_ml, strength_mg_per_unit, dose_mg_per_kg_min, dose_mg_per_kg_max, route, notes, created_at, updated_at, deleted_at",
    ),
    (
        "prescription_items",
        "id, visit_id, medication_id, medication_name, dose_mg, dose_ml, dose_text, frequency, duration_days, notes, created_at, updated_at, deleted_at",
    ),
    (
        "appointments",
        "id, patient_id, owner_id, title, starts_on, starts_time, duration_min, kind, status, series_id, notes, created_at, updated_at, deleted_at",
    ),
    ("settings", "rowid, key, value, updated_at"),
    (
        "import_runs",
        "id, source_name, source_sha256, started_at, finished_at, status, report_json",
    ),
    (
        "legacy_map",
        "id, import_run_id, source_sha256, source_table, source_id, target_table, target_id",
    ),
];

const V1_SEED: &str = "
INSERT INTO owners (id, full_name, phone, phone_raw, phone_alt, email, address, notes, search_text, deleted_at) VALUES
 (1, 'Synthetic Local Mobile', '05551112233', '0555 111 22 33', '5559998877', 'a@example.test', 'Street 1', 'note', 'synthetic local mobile 05551112233', NULL),
 (2, 'Synthetic Lost Zero', '5551112233', '5551112233', NULL, NULL, NULL, NULL, 'synthetic lost zero 5551112233', NULL),
 (3, 'Synthetic Foreign', '+4915112345678', '+49 151 12345678', '+905551110000', NULL, NULL, NULL, 'synthetic foreign', NULL),
 (4, 'Synthetic Spaced Landline', '0212 555 11 22', '0212 555 11 22', '02125551122', NULL, NULL, NULL, 'synthetic spaced landline', NULL),
 (5, 'Synthetic Deleted Owner', '05553334455', '05553334455', NULL, NULL, NULL, NULL, 'synthetic deleted owner', '2026-10-01T09:00:00.000Z'),
 (6, 'Synthetic Odd Values', '4915112345678', 'call after noon', '2125551122', NULL, NULL, NULL, 'synthetic odd values', NULL),
 (7, 'Synthetic No Phone', NULL, NULL, NULL, NULL, NULL, NULL, 'synthetic no phone', NULL),
 (8, 'Synthetic Short And Long', '055511122334', '055511122334', '055511122', NULL, NULL, NULL, 'synthetic short long', NULL);
INSERT INTO patients (id, owner_id, name, species, species_note, breed, sex, neutered, birth_date, color, microchip, notes, deceased_on, search_text, deleted_at) VALUES
 (1, 1, 'Synthetic Cat', 'cat', NULL, 'Tabby', 'female', 1, '2020-01-02', 'grey', '900000000000001', 'n', NULL, 'synthetic cat tabby', NULL),
 (2, 1, 'Synthetic Dog', 'dog', NULL, NULL, 'male', 0, NULL, NULL, NULL, NULL, '2026-05-05', 'synthetic dog', NULL),
 (3, 2, 'Synthetic Bird', 'bird', NULL, NULL, 'unknown', NULL, NULL, NULL, NULL, NULL, NULL, 'synthetic bird', NULL),
 (4, 3, 'Synthetic Rabbit', 'rabbit', NULL, NULL, 'unknown', NULL, NULL, NULL, NULL, NULL, NULL, 'synthetic rabbit', NULL),
 (5, 4, 'Synthetic Other', 'other', 'iguana', NULL, 'unknown', NULL, NULL, NULL, NULL, NULL, NULL, 'synthetic other iguana', '2026-10-02T09:00:00.000Z'),
 (6, 5, 'Synthetic Orphan Of Deleted', 'cat', NULL, NULL, 'unknown', NULL, NULL, NULL, NULL, NULL, NULL, 'synthetic orphan', '2026-10-01T09:00:00.000Z');
INSERT INTO visits (id, patient_id, visit_on, visit_time, reason, weight_kg, temperature_c, deleted_at) VALUES
 (1, 1, '2026-09-01', '10:30', 'checkup', 4.2, 38.5, NULL),
 (2, 1, '2026-10-01', NULL, 'followup', NULL, NULL, NULL),
 (3, 2, '2026-10-02', '09:00', 'deleted visit', 20, NULL, '2026-10-03T00:00:00.000Z'),
 (4, 5, '2026-08-01', NULL, 'visit of deleted patient', 1, NULL, NULL);
INSERT INTO vaccine_types (id, name, species, default_interval_days, notes, is_active, deleted_at) VALUES
 (1, 'Synthetic Combo', 'dog', 365, NULL, 1, NULL),
 (2, 'Synthetic Rabies', NULL, 365, 'note', 1, NULL),
 (3, 'Synthetic Inactive', 'cat', NULL, NULL, 0, NULL),
 (4, 'Synthetic Gone', 'bird', 30, NULL, 1, '2026-10-01T00:00:00.000Z'),
 (5, 'Synthetic Rabbit Vaccine', 'rabbit', 180, NULL, 1, NULL);
INSERT INTO vaccinations (id, patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on, batch_no, visit_id, deleted_at) VALUES
 (1, 2, 1, 'Synthetic Combo', '2025-10-02', '2026-10-02', 'B1', 3, NULL),
 (2, 1, 2, 'Synthetic Rabies', '2026-09-01', '2027-09-01', NULL, 1, NULL),
 (3, 1, NULL, 'Historical Vaccine', '2024-01-01', NULL, NULL, NULL, '2026-10-05T00:00:00.000Z'),
 (4, 4, 5, 'Synthetic Rabbit Vaccine', '2026-06-01', '2026-12-01', NULL, NULL, NULL);
INSERT INTO medications (id, name, active_ingredient, form, dose_mg_per_kg_min, dose_mg_per_kg_max, deleted_at) VALUES
 (1, 'Synthetic Medication', 'synthetium', 'tablet', 2.0, 4.0, NULL),
 (2, 'Synthetic Deleted Medication', NULL, NULL, NULL, NULL, '2026-10-01T00:00:00.000Z');
INSERT INTO prescription_items (id, visit_id, medication_id, medication_name, dose_mg, deleted_at) VALUES
 (1, 1, 1, 'Synthetic Medication', 8.4, NULL),
 (2, 2, NULL, 'Historical Medication', NULL, NULL);
INSERT INTO appointments (id, patient_id, owner_id, title, starts_on, starts_time, kind, status, series_id, deleted_at) VALUES
 (1, 1, 1, 'Checkup', '2026-11-11', '10:30', 'visit', 'planned', NULL, NULL),
 (2, NULL, NULL, 'All-day reminder', '2026-11-12', NULL, 'reminder', 'planned', NULL, NULL),
 (3, 2, 1, 'Booster', '2026-12-01', '09:00', 'vaccine', 'done', 'series-a', NULL),
 (4, 5, NULL, 'Cancelled of deleted patient', '2026-10-10', NULL, 'other', 'cancelled', NULL, '2026-10-02T00:00:00.000Z');
INSERT INTO settings (key, value) VALUES ('theme', 'dark'), ('seeded_vaccine_types', '1');
INSERT INTO import_runs (id, source_name, source_sha256, status, report_json) VALUES
 (1, 'synthetic.db', 'sha-a', 'completed', '{}');
INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, target_table, target_id) VALUES
 (1, 'sha-a', 'Patient', 42, 'patients', 1),
 (1, 'sha-a', 'PatientOwner', 7, 'owners', 1);
";

fn seed_v1(path: &Path, backups_dir: &Path) {
    let db = Database::open_with(path, backups_dir, v1()).unwrap();
    assert_eq!(db.schema_version().unwrap(), 1);
    db.conn().execute_batch(V1_SEED).unwrap();
}

#[test]
fn fresh_database_reaches_v2_with_new_tables_columns_and_indexes() {
    let tmp = tempfile::tempdir().unwrap();
    let bdir = tmp.path().join("backups");
    let db = Database::open(&tmp.path().join("a.db"), &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 2);
    assert!(backups(&bdir).is_empty(), "no backup for a brand-new file");
    let conn = db.conn();

    let strict: Vec<(String, i64)> = {
        let mut stmt = conn
            .prepare("SELECT name, strict FROM pragma_table_list WHERE schema = 'main' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    };
    for (name, strict) in &strict {
        assert_eq!(*strict, 1, "{name} must be STRICT");
    }
    let names: Vec<&str> = strict.iter().map(|(n, _)| n.as_str()).collect();
    for table in [
        "services",
        "inventory_items",
        "inventory_lots",
        "stock_movements",
        "charges",
        "payments",
        "attachments",
        "attachment_data",
        "visit_templates",
    ] {
        assert!(names.contains(&table), "missing table {table}");
    }
    assert!(
        !names.iter().any(|n| n.ends_with("_v2")),
        "temporary rebuild table left behind"
    );

    for (table, column) in [
        ("owners", "national_id"),
        ("owners", "city"),
        ("owners", "district"),
        ("patients", "alerts"),
        ("visits", "heart_rate_bpm"),
        ("visits", "resp_rate_bpm"),
        ("visits", "body_condition"),
        ("vaccine_types", "is_builtin"),
        ("vaccine_types", "manufacturer"),
        ("vaccine_types", "diseases"),
        ("vaccine_types", "primary_doses"),
        ("vaccine_types", "primary_interval_days"),
        ("vaccine_types", "min_age_weeks"),
        ("medications", "is_builtin"),
        ("medications", "category"),
        ("medications", "manufacturer"),
        ("medications", "target_species"),
        ("medications", "prescription_required"),
        ("medications", "pack_unit"),
        ("appointments", "vaccine_type_id"),
        ("appointments", "vaccination_id"),
    ] {
        let n = count(
            conn,
            &format!("SELECT count(*) FROM pragma_table_info('{table}') WHERE name = '{column}'"),
        );
        assert_eq!(n, 1, "{table}.{column}");
    }

    // Shared audit columns on every new user-facing table.
    for table in [
        "services",
        "inventory_items",
        "inventory_lots",
        "stock_movements",
        "charges",
        "payments",
        "attachments",
        "visit_templates",
    ] {
        let n = count(
            conn,
            &format!("SELECT count(*) FROM pragma_table_info('{table}') WHERE name IN ('id','created_at','updated_at','deleted_at')"),
        );
        assert_eq!(n, 4, "{table} audit columns");
    }

    let indexes: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'index' AND sql IS NOT NULL")
            .unwrap();
        let rows = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    };
    for index in [
        "patients_search_text_idx",
        "patients_owner_id_idx",
        "vaccine_types_name_idx",
        "charges_owner_id_idx",
        "payments_owner_id_idx",
        "inventory_lots_item_id_idx",
        "stock_movements_item_id_idx",
        "attachments_patient_id_idx",
        "appointments_vaccine_type_id_idx",
    ] {
        assert!(indexes.iter().any(|i| i == index), "missing index {index}");
    }
    check_integrity(conn).unwrap();
}

#[test]
fn populated_v1_database_migrates_with_every_row_and_foreign_key_intact() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed_v1(&path, &bdir);

    // Everything the v1 schema can see, captured before the upgrade.
    let before: Vec<_> = {
        let conn = Connection::open(&path).unwrap();
        V1_TABLES
            .iter()
            .map(|(t, c)| (*t, *c, snapshot(&conn, t, c)))
            .collect()
    };
    let phones_before = {
        let conn = Connection::open(&path).unwrap();
        phones(&conn)
    };
    assert!(backups(&bdir).is_empty());

    let db = Database::open(&path, &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 2);
    let conn = db.conn();

    // Every pre-existing value of every pre-existing column is unchanged (phone/phone_alt are
    // compared separately below).
    for (table, columns, rows) in &before {
        let after = snapshot(conn, table, columns);
        assert_eq!(&after, rows, "table {table} changed during migration");
    }
    assert_eq!(count(conn, "SELECT count(*) FROM owners"), 8);
    assert_eq!(count(conn, "SELECT count(*) FROM patients"), 6);
    assert_eq!(
        count(
            conn,
            "SELECT count(*) FROM patients WHERE deleted_at IS NOT NULL"
        ),
        2
    );

    // Phones: only the two documented patterns convert; phone_raw is never touched.
    let after = phones(conn);
    let expect: Vec<(i64, Option<&str>, Option<&str>)> = vec![
        (1, Some("+905551112233"), Some("+905559998877")),
        (2, Some("+905551112233"), None),
        (3, Some("+4915112345678"), Some("+905551110000")),
        (4, Some("0212 555 11 22"), Some("+902125551122")),
        (5, Some("+905553334455"), None),
        (6, Some("4915112345678"), Some("2125551122")),
        (7, None, None),
        (8, Some("055511122334"), Some("055511122")),
    ];
    assert_eq!(after.len(), expect.len());
    for ((id, phone, alt, raw), (eid, ephone, ealt)) in after.iter().zip(expect) {
        assert_eq!(*id, eid);
        assert_eq!(phone.as_deref(), ephone, "owner {id} phone");
        assert_eq!(alt.as_deref(), ealt, "owner {id} phone_alt");
        let raw_before = &phones_before.iter().find(|p| p.0 == *id).unwrap().3;
        assert_eq!(raw, raw_before, "owner {id} phone_raw must stay untouched");
    }

    // New columns start out empty / defaulted.
    assert_eq!(
        count(conn, "SELECT count(*) FROM owners WHERE national_id IS NOT NULL OR city IS NOT NULL OR district IS NOT NULL"),
        0
    );
    assert_eq!(
        count(
            conn,
            "SELECT count(*) FROM patients WHERE alerts IS NOT NULL"
        ),
        0
    );
    assert_eq!(
        count(
            conn,
            "SELECT count(*) FROM vaccine_types WHERE is_builtin != 0 OR manufacturer IS NOT NULL"
        ),
        0
    );
    assert_eq!(
        count(
            conn,
            "SELECT count(*) FROM medications WHERE is_builtin != 0 OR category IS NOT NULL"
        ),
        0
    );
    assert_eq!(
        count(conn, "SELECT count(*) FROM appointments WHERE vaccine_type_id IS NOT NULL OR vaccination_id IS NOT NULL"),
        0
    );

    // Referential structure: children still point at `patients` / `vaccine_types`, not at the
    // temporary rebuild names, and enforcement still works.
    assert_eq!(
        count(conn, "SELECT count(*) FROM sqlite_schema WHERE sql LIKE '%patients_v2%' OR sql LIKE '%vaccine_types_v2%'"),
        0
    );
    for (child, parent) in [
        ("visits", "patients"),
        ("vaccinations", "patients"),
        ("vaccinations", "vaccine_types"),
        ("appointments", "patients"),
        ("appointments", "vaccine_types"),
        ("appointments", "vaccinations"),
        ("attachments", "patients"),
        ("charges", "patients"),
    ] {
        let n = count(
            conn,
            &format!("SELECT count(*) FROM pragma_foreign_key_list('{child}') WHERE \"table\" = '{parent}'"),
        );
        assert_eq!(n, 1, "{child} -> {parent}");
    }
    assert_eq!(count(conn, "PRAGMA foreign_keys"), 1);
    assert!(conn
        .execute(
            "INSERT INTO visits (patient_id, visit_on) VALUES (999, '2026-11-04')",
            []
        )
        .is_err());
    assert!(conn
        .execute("DELETE FROM patients WHERE id = 1", [])
        .is_err());
    assert!(conn
        .execute("DELETE FROM vaccine_types WHERE id = 1", [])
        .is_err());

    // Search and uniqueness indexes survived the rebuilds.
    assert!(conn
        .execute(
            "INSERT INTO vaccine_types (name) VALUES ('synthetic combo')",
            []
        )
        .is_err());
    let plan: String = conn
        .query_row(
            "EXPLAIN QUERY PLAN SELECT id FROM patients WHERE owner_id = 1",
            [],
            |r| r.get(3),
        )
        .unwrap();
    assert!(plan.contains("patients_owner_id_idx"), "{plan}");

    check_integrity(conn).unwrap();
    assert_eq!(
        count(conn, "SELECT count(*) FROM pragma_foreign_key_check"),
        0
    );

    // A PreMigration backup of the untouched v1 file exists and is readable.
    let b = backups(&bdir);
    assert_eq!(b.len(), 1);
    let name = b[0].file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("pre-migration-v1-to-v2-"), "{name}");
    let old = Connection::open_with_flags(&b[0], OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(version(&old), 1);
    assert_eq!(phones(&old), phones_before);
    assert_eq!(count(&old, "SELECT count(*) FROM patients"), 6);
    check_integrity(&old).unwrap();

    // The migrated file keeps working with new data.
    conn.execute_batch(
        "UPDATE owners SET city = 'Synthetic City' WHERE id = 1;
         INSERT INTO visits (patient_id, visit_on, heart_rate_bpm, resp_rate_bpm, body_condition) VALUES (1, '2026-11-04', 120, 30, 5);
         INSERT INTO appointments (patient_id, title, starts_on, kind, vaccine_type_id, vaccination_id) VALUES (1, 'Planned', '2026-12-12', 'vaccine', 1, 1);",
    )
    .unwrap();
    check_integrity(conn).unwrap();
}

#[test]
fn reopening_a_v2_database_is_a_no_op() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed_v1(&path, &bdir);
    drop(Database::open(&path, &bdir).unwrap());
    assert_eq!(backups(&bdir).len(), 1);

    let db = Database::open(&path, &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 2);
    assert_eq!(backups(&bdir).len(), 1, "no second backup");
    assert_eq!(
        phones(db.conn())[0],
        (
            1,
            Some("+905551112233".into()),
            Some("+905559998877".into()),
            Some("0555 111 22 33".into())
        )
    );
    check_integrity(db.conn()).unwrap();
}

#[test]
fn species_extension_accepts_new_values_and_rejects_unknown_ones() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    conn.execute(
        "INSERT INTO owners (id, full_name) VALUES (1, 'Synthetic Owner')",
        [],
    )
    .unwrap();
    for (i, species) in [
        "cat", "dog", "bird", "rabbit", "rodent", "ferret", "reptile", "other",
    ]
    .iter()
    .enumerate()
    {
        conn.execute(
            "INSERT INTO patients (owner_id, name, species) VALUES (1, ?1, ?2)",
            rusqlite::params![format!("P{i}"), species],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO vaccine_types (name, species) VALUES (?1, ?2)",
            rusqlite::params![format!("V{i}"), species],
        )
        .unwrap();
    }
    for sql in [
        "INSERT INTO patients (owner_id, name, species) VALUES (1, 'X', 'dragon')",
        "INSERT INTO patients (owner_id, name, species) VALUES (1, 'X', 'horse')",
        "INSERT INTO vaccine_types (name, species) VALUES ('X', 'cattle')",
        "INSERT INTO patients (owner_id, name, species) VALUES (1, 'X', 'Cat')",
        "INSERT INTO vaccine_types (name, species) VALUES ('X', 'dragon')",
        "UPDATE patients SET species = 'unicorn' WHERE id = 1",
        "UPDATE vaccine_types SET species = '' WHERE id = 1",
    ] {
        let err = conn.execute(sql, []).unwrap_err();
        match err {
            rusqlite::Error::SqliteFailure(e, _) => {
                assert_eq!(
                    e.extended_code,
                    rusqlite::ffi::SQLITE_CONSTRAINT_CHECK,
                    "{sql}"
                )
            }
            other => panic!("unexpected {other:?} for {sql}"),
        }
    }
    // species stays NOT NULL on patients and nullable on vaccine_types.
    assert!(conn
        .execute(
            "INSERT INTO patients (owner_id, name, species) VALUES (1, 'X', NULL)",
            []
        )
        .is_err());
    conn.execute(
        "INSERT INTO vaccine_types (name, species) VALUES ('Any species', NULL)",
        [],
    )
    .unwrap();
}

#[test]
fn new_table_constraints_hold() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    conn.execute_batch(
        "INSERT INTO owners (id, full_name) VALUES (1, 'Synthetic Owner');
         INSERT INTO patients (id, owner_id, name, species) VALUES (1, 1, 'P', 'reptile');
         INSERT INTO inventory_items (id, name, kind) VALUES (1, 'Item', 'supply');",
    )
    .unwrap();
    for sql in [
        "INSERT INTO visits (patient_id, visit_on, body_condition) VALUES (1, '2026-11-04', 0)",
        "INSERT INTO visits (patient_id, visit_on, body_condition) VALUES (1, '2026-11-04', 10)",
        "INSERT INTO medications (name, category) VALUES ('M', 'poison')",
        "INSERT INTO medications (name, prescription_required) VALUES ('M', 2)",
        "INSERT INTO services (name, category) VALUES ('S', 'magic')",
        "INSERT INTO services (name, price_minor) VALUES ('S', -1)",
        "INSERT INTO inventory_items (name, kind) VALUES ('I', 'gadget')",
        "INSERT INTO stock_movements (item_id, delta, reason, moved_on) VALUES (1, 0, 'adjust', '2026-11-04')",
        "INSERT INTO stock_movements (item_id, delta, reason, moved_on) VALUES (1, 1, 'gift', '2026-11-04')",
        "INSERT INTO charges (owner_id, description, quantity, unit_price_minor, total_minor, charged_on) VALUES (1, 'c', 0, 100, 0, '2026-11-04')",
        "INSERT INTO charges (owner_id, description, quantity, unit_price_minor, total_minor, charged_on) VALUES (1, 'c', 1, -1, 0, '2026-11-04')",
        "INSERT INTO payments (owner_id, paid_on, amount_minor, method) VALUES (1, '2026-11-04', 0, 'cash')",
        "INSERT INTO payments (owner_id, paid_on, amount_minor, method) VALUES (1, '2026-11-04', 10, 'barter')",
        "INSERT INTO attachments (patient_id, kind, file_name, mime) VALUES (1, 'video', 'f', 'x/y')",
        "INSERT INTO charges (owner_id, description, unit_price_minor, total_minor, charged_on) VALUES (99, 'c', 1, 1, '2026-11-04')",
    ] {
        assert!(conn.execute(sql, []).is_err(), "should be rejected: {sql}");
    }
    conn.execute_batch(
        "INSERT INTO charges (id, owner_id, patient_id, description, quantity, unit_price_minor, total_minor, charged_on)
             VALUES (1, 1, 1, 'Exam', 1, 50000, 50000, '2026-11-04');
         INSERT INTO payments (owner_id, paid_on, amount_minor, method) VALUES (1, '2026-11-04', 20000, 'card');
         INSERT INTO inventory_lots (id, item_id, quantity) VALUES (1, 1, 5);
         INSERT INTO stock_movements (item_id, lot_id, delta, reason, moved_on) VALUES (1, 1, -2, 'use', '2026-11-04');
         INSERT INTO attachments (id, patient_id, kind, file_name, mime, size_bytes) VALUES (1, 1, 'photo', 'a.jpg', 'image/jpeg', 3);
         INSERT INTO attachment_data (attachment_id, data) VALUES (1, x'010203');
         INSERT INTO visit_templates (name) VALUES ('Template');",
    )
    .unwrap();
    // RESTRICT everywhere: parents with children cannot be hard-deleted.
    for sql in [
        "DELETE FROM owners WHERE id = 1",
        "DELETE FROM inventory_items WHERE id = 1",
        "DELETE FROM inventory_lots WHERE id = 1",
        "DELETE FROM attachments WHERE id = 1",
    ] {
        assert!(conn.execute(sql, []).is_err(), "{sql}");
    }
    assert!(conn
        .execute("UPDATE inventory_lots SET quantity = -1 WHERE id = 1", [])
        .is_err());
    check_integrity(conn).unwrap();
}

#[test]
fn failed_v2_migration_rolls_back_to_an_intact_v1_database() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.db");
    let bdir = tmp.path().join("backups");
    seed_v1(&path, &bdir);
    let before: Vec<_> = {
        let conn = Connection::open(&path).unwrap();
        V1_TABLES
            .iter()
            .map(|(t, c)| (*t, snapshot(&conn, t, c)))
            .collect()
    };
    let phones_before = phones(&Connection::open(&path).unwrap());

    // The real 0002 followed by a migration that fails: the runner applies both in one
    // transaction, so the already executed table rebuilds and phone updates must vanish.
    let mut list = migrations().to_vec();
    list.push(Migration {
        version: 3,
        name: "broken",
        sql: "CREATE TABLE half_done (x INTEGER); THIS IS NOT SQL;",
    });
    let err = Database::open_with(&path, &bdir, &list).err().unwrap();
    assert!(
        matches!(err, DbError::MigrationFailed { version: 3, .. }),
        "{err}"
    );

    // A violation that only surfaces at commit time (deferred FK) also rolls everything back.
    let mut list = migrations().to_vec();
    list.push(Migration {
        version: 3,
        name: "dangling_fk",
        sql: "PRAGMA defer_foreign_keys = ON; INSERT INTO visits (patient_id, visit_on) VALUES (12345, '2026-01-01');",
    });
    let err = Database::open_with(&path, &bdir, &list).err().unwrap();
    assert!(matches!(err, DbError::MigrationFailed { .. }), "{err}");

    let conn = Connection::open(&path).unwrap();
    assert_eq!(version(&conn), 1);
    for ((table, columns), (_, rows)) in V1_TABLES.iter().zip(&before) {
        assert_eq!(&snapshot(&conn, table, columns), rows, "{table}");
    }
    assert_eq!(
        phones(&conn),
        phones_before,
        "phones must be exactly as in v1"
    );
    assert_eq!(
        count(&conn, "SELECT count(*) FROM sqlite_schema WHERE name IN ('half_done','patients_v2','vaccine_types_v2','charges')"),
        0
    );
    assert_eq!(
        count(
            &conn,
            "SELECT count(*) FROM pragma_table_info('owners') WHERE name = 'city'"
        ),
        0
    );
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE name = 'patients'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!sql.contains("rodent"), "v1 species CHECK must be intact");
    check_integrity(&conn).unwrap();
    assert_eq!(backups(&bdir).len(), 2, "one backup per attempt, both kept");

    // The same file still upgrades cleanly afterwards.
    drop(conn);
    let db = Database::open(&path, &bdir).unwrap();
    assert_eq!(db.schema_version().unwrap(), 2);
    check_integrity(db.conn()).unwrap();
}
