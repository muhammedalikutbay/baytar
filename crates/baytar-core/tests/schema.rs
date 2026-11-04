use baytar_core::db::{check_integrity, Database};
use rusqlite::{Connection, Error};

fn assert_constraint(conn: &Connection, sql: &str, extended_code: i32) {
    match conn.execute(sql, []) {
        Err(Error::SqliteFailure(error, _)) => assert_eq!(error.extended_code, extended_code),
        result => panic!("expected constraint {extended_code}, got {result:?}: {sql}"),
    }
}

fn seed_owner_and_patient(conn: &Connection) {
    conn.execute_batch(
        "INSERT INTO owners (id, full_name) VALUES (1, 'Synthetic Owner');
         INSERT INTO patients (id, owner_id, name, species) VALUES (1, 1, 'Synthetic Cat', 'cat');",
    )
    .unwrap();
}

#[test]
fn fresh_database_has_all_tables_strict_types_indexes_and_no_triggers() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(db.schema_version().unwrap(), 4);
    let conn = db.conn();
    let mut stmt = conn
        .prepare("SELECT name, strict FROM pragma_table_list WHERE schema = 'main' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap();
    let tables: Vec<(String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let expected = [
        "appointments",
        "attachment_data",
        "attachments",
        "charges",
        "import_runs",
        "inventory_items",
        "inventory_lots",
        "legacy_map",
        "medications",
        "owners",
        "patients",
        "payments",
        "prescription_items",
        "reminder_contacts",
        "services",
        "settings",
        "stock_movements",
        "vaccinations",
        "vaccine_types",
        "visit_templates",
        "visits",
    ];
    assert_eq!(tables.len(), expected.len());
    for ((name, strict), expected) in tables.iter().zip(expected) {
        assert_eq!(name, expected);
        assert_eq!(*strict, 1);
    }
    let triggers: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type = 'trigger'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(triggers, 0);
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_schema WHERE type = 'index' AND sql IS NOT NULL ORDER BY name",
        )
        .unwrap();
    let indexes: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        indexes,
        [
            "appointments_starts_on_idx",
            "appointments_vaccination_id_idx",
            "appointments_vaccine_type_id_idx",
            "attachments_patient_id_idx",
            "attachments_visit_id_idx",
            "charges_charged_on_idx",
            "charges_inventory_item_id_idx",
            "charges_owner_id_idx",
            "charges_patient_id_idx",
            "charges_service_id_idx",
            "charges_visit_id_idx",
            "inventory_items_medication_id_idx",
            "inventory_items_vaccine_type_id_idx",
            "inventory_lots_expires_on_idx",
            "inventory_lots_item_id_idx",
            "owners_search_text_idx",
            "patients_owner_id_idx",
            "patients_search_text_idx",
            "payments_owner_id_idx",
            "payments_paid_on_idx",
            "prescription_items_visit_id_idx",
            "reminder_contacts_owner_id_idx",
            "reminder_contacts_ref_key_idx",
            "stock_movements_item_id_idx",
            "stock_movements_lot_id_idx",
            "vaccinations_next_due_on_idx",
            "vaccinations_patient_id_idx",
            "vaccine_types_name_idx",
            "visits_patient_id_visit_on_idx",
        ]
    );
    check_integrity(conn).unwrap();
}

#[test]
fn connected_clinical_records_and_import_provenance_can_be_inserted() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    // These tests use fixed ids; start from an empty catalog instead of the seeded one (0003).
    conn.execute_batch("DELETE FROM vaccine_types; DELETE FROM medications; DELETE FROM services;")
        .unwrap();
    seed_owner_and_patient(conn);
    conn.execute_batch(
        "INSERT INTO visits (id, patient_id, visit_on, visit_time, weight_kg, temperature_c)
             VALUES (1, 1, '2026-11-04', '10:30', 4.2, 38.5);
         INSERT INTO vaccine_types (id, name, species, default_interval_days)
             VALUES (1, 'Synthetic Vaccine', 'cat', 365);
         INSERT INTO vaccinations (patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on, visit_id)
             VALUES (1, 1, 'Synthetic Vaccine Snapshot', '2026-11-04', '2027-11-04', 1);
         INSERT INTO vaccinations (patient_id, vaccine_name, given_on)
             VALUES (1, 'Historical Vaccine', '2025-11-04');
         INSERT INTO medications (id, name, form, dose_mg_per_kg_min, dose_mg_per_kg_max)
             VALUES (1, 'Synthetic Medication', 'tablet', 2.0, 4.0);
         INSERT INTO prescription_items (visit_id, medication_id, medication_name, dose_mg)
             VALUES (1, 1, 'Synthetic Medication Snapshot', 8.4);
         INSERT INTO prescription_items (visit_id, medication_name, dose_text)
             VALUES (1, 'Historical Medication', 'Historical instructions');
         INSERT INTO appointments (patient_id, owner_id, title, starts_on, starts_time, kind)
             VALUES (1, 1, 'Synthetic Checkup', '2026-11-11', '10:30', 'visit');
         INSERT INTO appointments (title, starts_on) VALUES ('All-day reminder', '2026-11-12');
         INSERT INTO settings (key, value) VALUES ('theme', 'system');
         INSERT INTO import_runs (id, source_name, source_sha256, status, report_json)
             VALUES (1, 'synthetic.db', 'synthetic-source-hash', 'completed', '{}');
         INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, target_table, target_id)
             VALUES (1, 'synthetic-source-hash', 'Patient', 42, 'patients', 1);",
    )
    .unwrap();
    let defaults: (String, i64, Option<i64>, String) = conn
        .query_row(
            "SELECT sex, birth_date_is_estimate, neutered, search_text FROM patients WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(defaults, ("unknown".into(), 0, None, String::new()));
    let appointment: (String, String, Option<String>) = conn
        .query_row(
            "SELECT kind, status, starts_time FROM appointments WHERE title = 'All-day reminder'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(appointment, ("other".into(), "planned".into(), None));
    let audit: (String, String, Option<String>) = conn
        .query_row(
            "SELECT created_at, updated_at, deleted_at FROM owners WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    for timestamp in [&audit.0, &audit.1] {
        assert_eq!(timestamp.len(), 24);
        assert!(timestamp.ends_with('Z'));
        chrono::DateTime::parse_from_rfc3339(timestamp).unwrap();
    }
    assert_eq!(audit.2, None);
    let mut fk = conn.prepare("PRAGMA foreign_key_check").unwrap();
    assert!(fk.query([]).unwrap().next().unwrap().is_none());
    check_integrity(conn).unwrap();
}

#[test]
fn checks_reject_invalid_enums_booleans_and_numeric_bounds() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    seed_owner_and_patient(conn);
    for sql in [
        "INSERT INTO patients (owner_id, name, species) VALUES (1, 'Synthetic', 'dragon')",
        "UPDATE patients SET sex = 'invalid' WHERE id = 1",
        "UPDATE patients SET neutered = 2 WHERE id = 1",
        "UPDATE patients SET birth_date_is_estimate = -1 WHERE id = 1",
        "INSERT INTO visits (patient_id, visit_on, weight_kg) VALUES (1, '2026-11-04', -1)",
        "INSERT INTO visits (patient_id, visit_on, weight_kg) VALUES (1, '2026-11-04', 0)",
        "INSERT INTO vaccine_types (name, species) VALUES ('Synthetic', 'dragon')",
        "INSERT INTO vaccine_types (name, is_active) VALUES ('Synthetic', 2)",
        "INSERT INTO medications (name, form) VALUES ('Synthetic', 'invalid')",
        "INSERT INTO medications (name, dose_mg_per_kg_min, dose_mg_per_kg_max) VALUES ('Synthetic', 4, 2)",
        "INSERT INTO appointments (title, starts_on, kind) VALUES ('Synthetic', '2026-11-04', 'invalid')",
        "INSERT INTO appointments (title, starts_on, status) VALUES ('Synthetic', '2026-11-04', 'invalid')",
        "INSERT INTO import_runs (source_sha256, status) VALUES ('synthetic', 'invalid')",
    ] {
        assert_constraint(conn, sql, rusqlite::ffi::SQLITE_CONSTRAINT_CHECK);
    }
    conn.execute_batch(
        "INSERT INTO visits (patient_id, visit_on) VALUES (1, '2026-11-04');
         INSERT INTO medications (name, dose_mg_per_kg_min) VALUES ('Min only', 2);
         INSERT INTO medications (name, dose_mg_per_kg_max) VALUES ('Max only', 4);
         INSERT INTO medications (name, dose_mg_per_kg_min, dose_mg_per_kg_max) VALUES ('Equal bounds', 2, 2);",
    )
    .unwrap();
    assert_constraint(
        conn,
        "UPDATE patients SET neutered = 'invalid' WHERE id = 1",
        rusqlite::ffi::SQLITE_CONSTRAINT_DATATYPE,
    );
}

#[test]
fn foreign_keys_restrict_deletion_even_after_soft_delete() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    seed_owner_and_patient(conn);
    assert_constraint(
        conn,
        "INSERT INTO patients (owner_id, name, species) VALUES (999, 'Synthetic', 'cat')",
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_constraint(
        conn,
        "DELETE FROM owners WHERE id = 1",
        rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER,
    );
    conn.execute_batch(
        "UPDATE patients SET deleted_at = '2026-11-04T12:00:00.000Z' WHERE id = 1;
                        UPDATE owners SET deleted_at = '2026-11-04T12:00:00.000Z' WHERE id = 1;",
    )
    .unwrap();
    assert_constraint(
        conn,
        "DELETE FROM owners WHERE id = 1",
        rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER,
    );
    check_integrity(conn).unwrap();
}

#[test]
fn vaccine_name_uniqueness_excludes_only_soft_deleted_rows() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    // These tests use fixed ids; start from an empty catalog instead of the seeded one (0003).
    conn.execute_batch("DELETE FROM vaccine_types; DELETE FROM medications; DELETE FROM services;")
        .unwrap();
    conn.execute(
        "INSERT INTO vaccine_types (name, is_active) VALUES ('Synthetic Vaccine', 0)",
        [],
    )
    .unwrap();
    assert_constraint(
        conn,
        "INSERT INTO vaccine_types (name) VALUES ('SYNTHETIC VACCINE')",
        rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE,
    );
    conn.execute_batch(
        "UPDATE vaccine_types SET deleted_at = '2026-11-04T12:00:00.000Z';
         INSERT INTO vaccine_types (name) VALUES ('SYNTHETIC VACCINE');",
    )
    .unwrap();
    assert_constraint(
        conn,
        "UPDATE vaccine_types SET deleted_at = NULL WHERE id = 1",
        rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE,
    );
}

#[test]
fn legacy_identity_is_unique_across_runs_of_the_same_source() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    conn.execute_batch(
        "INSERT INTO import_runs (id, source_sha256, status) VALUES (1, 'synthetic-a', 'completed');
         INSERT INTO import_runs (id, source_sha256, status) VALUES (2, 'synthetic-a', 'previewed');
         INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, target_table, target_id)
             VALUES (1, 'synthetic-a', 'Patient', 42, 'patients', 1);",
    )
    .unwrap();
    assert_constraint(
        conn,
        "INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, target_table, target_id)
             VALUES (2, 'synthetic-a', 'Patient', 42, 'patients', 2)",
        rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE,
    );
    conn.execute_batch(
        "INSERT INTO import_runs (id, source_sha256, status) VALUES (3, 'synthetic-b', 'completed');
         INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, target_table, target_id)
             VALUES (3, 'synthetic-b', 'Patient', 42, 'patients', 2);",
    )
    .unwrap();
}
