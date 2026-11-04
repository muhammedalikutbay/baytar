//! The catalog migration (0003) ships built-in pet vaccines, medicines and services.

use baytar_core::db::Database;

fn builtin_counts(db: &Database) -> (i64, i64, i64) {
    let count = |table: &str| -> i64 {
        db.conn()
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM {table} WHERE is_builtin = 1 AND deleted_at IS NULL"
                ),
                [],
                |row| row.get(0),
            )
            .unwrap()
    };
    (
        count("vaccine_types"),
        count("medications"),
        count("services"),
    )
}

#[test]
fn fresh_database_has_the_catalog_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("baytar.db");
    let backups = dir.path().join("backups");

    let first = {
        let db = Database::open(&path, &backups).unwrap();
        builtin_counts(&db)
    };
    assert!(first.0 >= 30, "vaccines: {}", first.0);
    assert!(first.1 >= 100, "medicines: {}", first.1);
    assert!(first.2 >= 25, "services: {}", first.2);

    let db = Database::open(&path, &backups).unwrap();
    assert_eq!(
        builtin_counts(&db),
        first,
        "reopening must not duplicate the catalog"
    );
}

#[test]
fn catalog_is_pets_only() {
    let db = Database::open_in_memory().unwrap();
    let food_animal: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM vaccine_types WHERE species NOT IN \
             ('cat','dog','bird','rabbit','rodent','ferret','reptile','other')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(food_animal, 0);
}
