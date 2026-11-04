mod support;

use std::{fs, time::SystemTime};

use baytar_core::legacy::inspect::inspect;
use rusqlite::types::Value;
use support::{legacy_db, LegacySpec, Owner, Patient, TempDir, Todo};

fn sample() -> LegacySpec {
    LegacySpec {
        owners: vec![
            Owner {
                id: Some(1),
                name: Some("Synthetic Owner Alpha".into()),
                address: Some("Synthetic Address".into()),
                phone: Some(Value::Integer(5321234567)),
            },
            Owner {
                id: Some(2),
                name: Some("Synthetic Owner Beta".into()),
                address: None,
                phone: Some(Value::Text("0532 123 4567".into())),
            },
        ],
        patients: vec![
            Patient {
                id: Some(1),
                owner_id: Some(1),
                name: Some("Synthetic Patient One".into()),
                patient_type: Some("Kedi".into()),
                gender: Some("Erkek".into()),
                birth_date: Some("4 Kas\u{0131}m 2026 \u{00C7}ar\u{015F}amba".into()),
                registration_date: Some("04-11-2026".into()),
                note: Some("Synthetic private note".into()),
            },
            Patient {
                id: Some(2),
                owner_id: Some(99),
                name: Some("Synthetic Patient Two".into()),
                patient_type: Some("SyntheticFreeSpecies".into()),
                gender: Some("Di\u{015F}i".into()),
                birth_date: Some("2026-11-04".into()),
                registration_date: Some("04.11.2026".into()),
                note: None,
            },
        ],
        todos: vec![Todo {
            id: Some(1),
            owner_id: Some(1),
            patient_id: Some(99),
            name: Some("Synthetic reminder".into()),
            date: Some("2026-11-04".into()),
            note: Some("Synthetic todo note".into()),
        }],
    }
}

#[test]
fn reports_format_phone_combo_and_orphan_aggregates() {
    let temp = TempDir::new();
    let path = legacy_db(temp.path(), &sample());
    let report = inspect(&path).expect("inspect synthetic db");

    assert_eq!(report.user_version, 0);
    assert_eq!(
        report
            .tables
            .iter()
            .find(|table| table.name == "Patient")
            .unwrap()
            .row_count,
        2
    );
    let owners = &report.known_tables["PatientOwner"];
    let phones = owners.phone.as_ref().unwrap();
    assert_eq!(phones.storage_classes["integer"], 1);
    assert_eq!(phones.storage_classes["text"], 1);
    assert_eq!(phones.digit_length_buckets["10"], 1);
    assert_eq!(phones.digit_length_buckets["11"], 1);
    assert_eq!(phones.ten_digit_starting_with_five, 1);
    assert_eq!(owners.duplicate_looking_owners, Some(2));
    let patients = &report.known_tables["Patient"];
    assert_eq!(patients.dates["BirthDate"]["long-tr"], 1);
    assert_eq!(patients.dates["BirthDate"]["yyyy-MM-dd"], 1);
    assert_eq!(patients.dates["RegistrationDate"]["dd-MM-yyyy"], 1);
    assert_eq!(patients.dates["RegistrationDate"]["dd.MM.yyyy"], 1);
    assert_eq!(patients.combo_values["PatientType"]["Kedi"], 1);
    assert_eq!(patients.combo_values["PatientType"]["other"], 1);
    assert_eq!(patients.orphan_patients, Some(1));
    assert_eq!(
        report.known_tables["ToDo"].dates["ToDoDate"]["yyyy-MM-dd"],
        1
    );
    assert_eq!(report.known_tables["ToDo"].orphan_todos, Some(1));
}

#[test]
fn json_report_never_contains_names_phones_or_notes() {
    let temp = TempDir::new();
    let path = legacy_db(temp.path(), &sample());
    let report = inspect(&path).expect("inspect synthetic db");
    let json = serde_json::to_string(&report).expect("serialize report");
    for private_value in [
        "Synthetic Owner Alpha",
        "Synthetic Patient One",
        "Synthetic Address",
        "0532 123 4567",
        "05321234567",
        "5321234567",
        "Synthetic private note",
        "Synthetic todo note",
        "SyntheticFreeSpecies",
    ] {
        assert!(
            !json.contains(private_value),
            "report leaked {private_value}"
        );
    }
}

#[test]
fn inspection_leaves_database_bytes_time_and_sidecars_unchanged() {
    let temp = TempDir::new();
    let path = legacy_db(temp.path(), &sample());
    let before_bytes = fs::read(&path).expect("read db");
    let before_modified: SystemTime = fs::metadata(&path)
        .expect("db metadata")
        .modified()
        .expect("db modified time");
    inspect(&path).expect("inspect synthetic db");
    assert_eq!(fs::read(&path).expect("read db again"), before_bytes);
    assert_eq!(
        fs::metadata(&path)
            .expect("db metadata after")
            .modified()
            .expect("modified time after"),
        before_modified
    );
    let path_text = path.to_string_lossy();
    assert!(!std::path::PathBuf::from(format!("{path_text}-wal")).exists());
    assert!(!std::path::PathBuf::from(format!("{path_text}-shm")).exists());
}
