//! End-to-end tests of the legacy importer. All data is synthetic.

mod support;

use std::{fs, path::Path};

use baytar_core::{
    backup::{self, BackupKind},
    db::Database,
    legacy::{
        import::{
            self, format_preview, format_report, ImportIssueKind, ImportReport, UNKNOWN_OWNER_NAME,
        },
        LegacyError,
    },
};
use rusqlite::{params, types::Value, Connection};
use support::{legacy_db, LegacySpec, Owner, Patient, TempDir, Todo};

fn owner(id: i64, name: &str, phone: Value) -> Owner {
    Owner {
        id: Some(id),
        name: Some(name.into()),
        address: None,
        phone: Some(phone),
    }
}

fn patient(id: i64, owner_id: Option<i64>, name: &str) -> Patient {
    Patient {
        id: Some(id),
        owner_id,
        name: Some(name.into()),
        patient_type: None,
        gender: None,
        birth_date: None,
        registration_date: None,
        note: None,
    }
}

fn todo(id: i64, owner_id: Option<i64>, patient_id: Option<i64>, date: &str) -> Todo {
    Todo {
        id: Some(id),
        owner_id,
        patient_id,
        name: Some(format!("Synthetic reminder {id}")),
        date: Some(date.into()),
        note: Some(format!("Synthetic reminder note {id}")),
    }
}

/// Owners 1-3, patients 1-3 (3 is an orphan), to-dos 1-3 (3 is an orphan).
fn sample() -> LegacySpec {
    let mut alpha = owner(1, "Synthetic Owner Alpha", Value::Integer(5321234567));
    alpha.address = Some("Synthetic Address 1".into());
    let beta = owner(
        2,
        "  Synthetic Owner Beta  ",
        Value::Text("+90 532 765 43 21".into()),
    );
    let mut gamma = owner(3, "", Value::Text("call after noon".into()));
    gamma.name = None;

    let mut one = patient(1, Some(1), "Synthetic Patient One");
    one.patient_type = Some("Kedi".into());
    one.gender = Some("Erkek".into());
    one.birth_date = Some("4 Kasım 2026 Çarşamba".into());
    one.registration_date = Some("04-11-2026".into());
    one.note = Some("Synthetic private note".into());
    let mut two = patient(2, Some(2), "Synthetic Patient Two");
    two.patient_type = Some("SyntheticFreeSpecies".into());
    two.gender = Some("SyntheticGender".into());
    let mut three = patient(3, Some(99), "Synthetic Orphan Patient");
    three.patient_type = Some("Köpek".into());

    LegacySpec {
        owners: vec![alpha, beta, gamma],
        patients: vec![one, two, three],
        todos: vec![
            todo(1, Some(1), Some(1), "2020-01-01"),
            todo(2, None, Some(2), "2099-01-01"),
            todo(3, None, None, "2099-02-02"),
        ],
    }
}

struct Env {
    dir: TempDir,
}

impl Env {
    fn new() -> Self {
        Self {
            dir: TempDir::new(),
        }
    }

    fn legacy(&self, spec: &LegacySpec) -> std::path::PathBuf {
        let folder = self.dir.path().join("legacy");
        fs::create_dir_all(&folder).unwrap();
        legacy_db(&folder, spec)
    }

    fn backups(&self) -> std::path::PathBuf {
        self.dir.path().join("backups")
    }

    fn work(&self) -> std::path::PathBuf {
        self.dir.path().join("work")
    }

    fn target(&self) -> Database {
        Database::open(&self.dir.path().join("baytar.db"), &self.backups()).unwrap()
    }

    fn run(&self, source: &Path, target: &mut Database) -> ImportReport {
        let prepared = import::prepare(source, &self.work()).unwrap();
        import::execute(&prepared, target, &self.backups()).unwrap()
    }
}

fn count(db: &Database, table: &str) -> i64 {
    db.conn()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn full_import_maps_every_table() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    let report = env.run(&source, &mut target);

    assert_eq!(
        (
            report.inserted.owners,
            report.inserted.patients,
            report.inserted.todos
        ),
        (3, 3, 3)
    );
    assert_eq!(report.skipped.total(), 0);
    // 3 owners + 1 generated + 3 patients + 3 to-dos.
    assert_eq!(count(&target, "owners"), 4);
    assert_eq!(count(&target, "patients"), 3);
    assert_eq!(count(&target, "appointments"), 3);
    assert_eq!(count(&target, "legacy_map"), 3 + 3 + 3 + 1);

    let conn = target.conn();
    let (name, phone, phone_raw, address, search): (String, String, String, String, String) = conn
        .query_row(
            "SELECT full_name, phone, phone_raw, address, search_text FROM owners WHERE full_name = 'Synthetic Owner Alpha'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();
    assert_eq!(name, "Synthetic Owner Alpha");
    assert_eq!(
        phone, "+905321234567",
        "lost leading zero restored, stored as E.164"
    );
    assert_eq!(phone_raw, "5321234567", "INTEGER rendered without decimals");
    assert_eq!(address, "Synthetic Address 1");
    assert_eq!(search, "synthetic owner alpha 905321234567 05321234567");

    let beta_phone: String = conn
        .query_row(
            "SELECT phone FROM owners WHERE full_name = 'Synthetic Owner Beta'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        beta_phone, "+905327654321",
        "name trimmed, already international"
    );

    let (blank_name, blank_phone, blank_raw): (String, String, String) = conn
        .query_row(
            "SELECT full_name, phone, phone_raw FROM owners WHERE phone_raw = 'call after noon'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(blank_name, "İsimsiz sahip");
    assert_eq!(
        blank_phone, "call after noon",
        "unparsed phone kept as typed"
    );
    assert_eq!(blank_raw, "call after noon");

    let (species, note, sex, birth, notes, created, search): (
        String,
        Option<String>,
        String,
        String,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT species, species_note, sex, birth_date, notes, created_at, search_text FROM patients WHERE name = 'Synthetic Patient One'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .unwrap();
    assert_eq!(
        (species.as_str(), note, sex.as_str()),
        ("cat", None, "male")
    );
    assert_eq!(birth, "2026-11-04", "Turkish long date parsed");
    assert_eq!(notes, "Synthetic private note");
    assert_eq!(
        created, "2026-11-04T00:00:00.000Z",
        "dd-MM-yyyy registration date"
    );
    assert_eq!(search, "synthetic patient one");

    let (species, note, sex, notes): (String, String, String, String) = conn
        .query_row(
            "SELECT species, species_note, sex, notes FROM patients WHERE name = 'Synthetic Patient Two'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!((species.as_str(), sex.as_str()), ("other", "unknown"));
    assert_eq!(note, "SyntheticFreeSpecies", "free-text species kept");
    assert_eq!(notes, "Eski kayıttaki cinsiyet: SyntheticGender");

    let status = |title: &str| -> (String, String, String) {
        conn.query_row(
            "SELECT status, kind, starts_on FROM appointments WHERE title = ?1",
            [title],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    };
    assert_eq!(
        status("Synthetic reminder 1"),
        ("done".into(), "reminder".into(), "2020-01-01".into())
    );
    assert_eq!(status("Synthetic reminder 2").0, "planned");
    assert_eq!(status("Synthetic reminder 2").2, "2099-01-01");

    // To-do 2 has a patient but no owner: it follows the patient's owner.
    let (todo_owner, patient_owner): (i64, i64) = conn
        .query_row(
            "SELECT a.owner_id, p.owner_id FROM appointments a JOIN patients p ON p.id = a.patient_id \
             WHERE a.title = 'Synthetic reminder 2'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(todo_owner, patient_owner);

    // The run is recorded with its report.
    let (status, json): (String, String) = conn
        .query_row(
            "SELECT status, report_json FROM import_runs WHERE id = ?1",
            [report.run_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "completed");
    let stored: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(stored["runId"], report.run_id);
    assert_eq!(stored["inserted"]["owners"], 3);
    assert!(backup::list(&env.backups())
        .unwrap()
        .iter()
        .any(|b| b.kind == BackupKind::PreImport));
}

#[test]
fn preview_writes_nothing_and_matches_the_report() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    let prepared = import::prepare(&source, &env.work()).unwrap();

    let preview = import::preview(&prepared, &target).unwrap();
    for table in [
        "owners",
        "patients",
        "appointments",
        "import_runs",
        "legacy_map",
    ] {
        assert_eq!(count(&target, table), 0, "{table}");
    }
    assert_eq!(
        (
            preview.found.owners,
            preview.found.patients,
            preview.found.todos
        ),
        (3, 3, 3)
    );
    assert_eq!(preview.to_import, preview.found);
    assert_eq!(preview.phones_adjusted, 1);
    assert_eq!(preview.phones_unparsed, 1);
    assert_eq!((preview.orphan_patients, preview.orphan_todos), (1, 1));
    assert!(preview.orphan_owner_needed);
    assert_eq!((preview.todos_done, preview.todos_planned), (1, 2));
    assert_eq!(preview.species_mapped_to_other, 1);
    assert_eq!(preview.sex_mapped_to_unknown, 1);
    assert_eq!(prepared.source_sha256().len(), 64);

    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(report.preview, preview);
}

#[test]
fn reimporting_the_same_file_skips_everything() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    let first = env.run(&source, &mut target);
    let before = (
        count(&target, "owners"),
        count(&target, "patients"),
        count(&target, "appointments"),
        count(&target, "legacy_map"),
    );

    let prepared = import::prepare(&source, &env.work()).unwrap();
    let preview = import::preview(&prepared, &target).unwrap();
    assert_eq!(preview.already_imported, first.inserted);
    assert_eq!(preview.to_import.total(), 0);

    // Same backups folder on purpose: the second pre-import backup must not collide.
    let second = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(second.inserted.total(), 0);
    assert_eq!(second.skipped, first.inserted);
    assert!(!second.orphan_owner_created);
    assert_ne!(second.run_id, first.run_id);
    let after = (
        count(&target, "owners"),
        count(&target, "patients"),
        count(&target, "appointments"),
        count(&target, "legacy_map"),
    );
    assert_eq!(before, after);
    assert_eq!(count(&target, "import_runs"), 2);
}

#[test]
fn missing_columns_and_tables_still_import_the_rest() {
    let env = Env::new();
    let source = env.legacy(&sample());
    {
        let connection = Connection::open(&source).unwrap();
        connection
            .execute_batch(
                "ALTER TABLE PatientOwner DROP COLUMN OwnerAdress;\n\
                 DROP TABLE ToDo;\n\
                 ALTER TABLE Patient ADD COLUMN SyntheticExtra TEXT;\n\
                 CREATE TABLE SyntheticLog(Id INTEGER);",
            )
            .unwrap();
    }
    let mut target = env.target();
    let prepared = import::prepare(&source, &env.work()).unwrap();
    let schema = prepared.schema();
    assert_eq!(schema.missing_tables, ["ToDo"]);
    assert!(schema
        .missing_columns
        .contains(&"PatientOwner.OwnerAdress".to_owned()));
    assert_eq!(schema.unknown_tables, ["SyntheticLog"]);
    assert_eq!(schema.unknown_columns, ["Patient.SyntheticExtra"]);

    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(
        (
            report.inserted.owners,
            report.inserted.patients,
            report.inserted.todos
        ),
        (3, 3, 0)
    );
    let addresses: i64 = target
        .conn()
        .query_row("SELECT COUNT(address) FROM owners", [], |r| r.get(0))
        .unwrap();
    assert_eq!(addresses, 0);
    assert_eq!(count(&target, "appointments"), 0);
}

#[test]
fn column_and_table_names_are_case_insensitive() {
    let env = Env::new();
    let folder = env.dir.path().join("legacy");
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join("lower.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE patientowner(id INTEGER PRIMARY KEY, ownername TEXT, ownerphone INTEGER);\n\
             INSERT INTO patientowner VALUES (7, 'Synthetic Lower', 5320000000);",
        )
        .unwrap();
    drop(connection);
    let mut target = env.target();
    let report = env.run(&path, &mut target);
    assert_eq!(report.inserted.owners, 1);
    let phone: String = target
        .conn()
        .query_row("SELECT phone FROM owners", [], |r| r.get(0))
        .unwrap();
    assert_eq!(phone, "+905320000000");
}

#[test]
fn orphans_share_one_generated_owner_per_file() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    let first = env.run(&source, &mut target);
    assert!(first.orphan_owner_created);
    let second = env.run(&source, &mut target);
    assert!(!second.orphan_owner_created);

    let generated: i64 = target
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM owners WHERE full_name = ?1",
            [UNKNOWN_OWNER_NAME],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(generated, 1);
    let conn = target.conn();
    let patient_owner: String = conn
        .query_row(
            "SELECT o.full_name FROM patients p JOIN owners o ON o.id = p.owner_id WHERE p.name = 'Synthetic Orphan Patient'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(patient_owner, UNKNOWN_OWNER_NAME);
    let (todo_owner, notes): (String, Option<String>) = conn
        .query_row(
            "SELECT o.full_name, a.notes FROM appointments a JOIN owners o ON o.id = a.owner_id WHERE a.title = 'Synthetic reminder 3'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(todo_owner, UNKNOWN_OWNER_NAME);
    assert_eq!(notes.as_deref(), Some("Synthetic reminder note 3"));
    // The dangling owner number of the orphan patient is not lost.
    let patient_notes: String = conn
        .query_row(
            "SELECT notes FROM patients WHERE name = 'Synthetic Orphan Patient'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(patient_notes, "Eski kayıttaki sahip no: 99");

    let mapped: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM legacy_map WHERE source_table = 'orphan_owner' AND source_id = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(mapped, 1);
}

#[test]
fn unparsable_values_are_preserved_and_listed() {
    let env = Env::new();
    let mut spec = LegacySpec {
        owners: vec![owner(1, "Synthetic Owner", Value::Integer(5320000001))],
        ..LegacySpec::default()
    };
    let mut odd = patient(1, Some(1), "Synthetic Odd Dates");
    odd.birth_date = Some("someday soon".into());
    odd.registration_date = Some("31-31-2020".into());
    odd.note = Some("Synthetic note".into());
    spec.patients.push(odd);
    spec.todos.push(todo(1, Some(1), None, "next week"));
    let mut blank = todo(2, Some(1), None, "");
    blank.date = None;
    spec.todos.push(blank);

    let source = env.legacy(&spec);
    let mut target = env.target();
    let report = env.run(&source, &mut target);

    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let conn = target.conn();
    let (birth, created_default, notes): (Option<String>, String, String) = conn
        .query_row(
            "SELECT birth_date, created_at, notes FROM patients",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(birth, None);
    assert!(
        !created_default.starts_with("31-"),
        "no invented registration time"
    );
    assert_eq!(
        notes,
        "Synthetic note\nEski kayıttaki doğum tarihi: someday soon\nEski kayıttaki kayıt tarihi: 31-31-2020"
    );

    let (starts_on, status, todo_notes): (String, String, String) = conn
        .query_row(
            "SELECT starts_on, status, notes FROM appointments WHERE title = 'Synthetic reminder 1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (starts_on.as_str(), status.as_str()),
        (today.as_str(), "planned")
    );
    assert_eq!(
        todo_notes,
        "Synthetic reminder note 1\nEski kayıttaki tarih: next week"
    );
    let blank_notes: String = conn
        .query_row(
            "SELECT notes FROM appointments WHERE title = 'Synthetic reminder 2'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(blank_notes.contains("Eski kayıtta tarih yoktu."));

    let pv = &report.preview;
    assert_eq!(pv.dates_unparsed["Patient.BirthDate"], 1);
    assert_eq!(pv.dates_unparsed["Patient.RegistrationDate"], 1);
    assert_eq!(pv.dates_unparsed["ToDo.ToDoDate"], 2);
    let raw_of = |table: &str, field: &str| {
        pv.issues
            .iter()
            .find(|i| {
                i.table == table && i.field == field && i.kind == ImportIssueKind::DateUnparsed
            })
            .map(|i| i.raw.clone())
    };
    assert_eq!(
        raw_of("Patient", "BirthDate"),
        Some(Some("someday soon".into()))
    );
    assert_eq!(
        raw_of("Patient", "RegistrationDate"),
        Some(Some("31-31-2020".into()))
    );
    assert!(raw_of("ToDo", "ToDoDate").is_some());
}

#[test]
fn a_failure_mid_transaction_leaves_the_target_untouched() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    // Owners and patients are inserted first; the first to-do then fails.
    target
        .conn()
        .execute_batch(
            "CREATE TRIGGER synthetic_fail BEFORE INSERT ON appointments \
             BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
        )
        .unwrap();

    let prepared = import::prepare(&source, &env.work()).unwrap();
    let error = import::execute(&prepared, &mut target, &env.backups()).unwrap_err();
    assert!(matches!(error, LegacyError::Database(_)), "{error}");

    for table in [
        "owners",
        "patients",
        "appointments",
        "import_runs",
        "legacy_map",
    ] {
        assert_eq!(count(&target, table), 0, "{table} must be unchanged");
    }
    let backups = backup::list(&env.backups()).unwrap();
    assert!(
        backups.iter().any(|b| b.kind == BackupKind::PreImport),
        "the backup stays"
    );

    // Without the failure the same prepared import succeeds.
    target
        .conn()
        .execute_batch("DROP TRIGGER synthetic_fail")
        .unwrap();
    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(report.inserted.todos, 3);
}

#[test]
fn the_source_file_is_never_changed() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let folder = source.parent().unwrap().to_owned();
    let listing = || {
        let mut names: Vec<_> = fs::read_dir(&folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    };
    let bytes = fs::read(&source).unwrap();
    let modified = fs::metadata(&source).unwrap().modified().unwrap();
    let files = listing();

    let mut target = env.target();
    let prepared = import::prepare(&source, &env.work()).unwrap();
    import::preview(&prepared, &target).unwrap();
    import::execute(&prepared, &mut target, &env.backups()).unwrap();
    drop(prepared);

    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(fs::metadata(&source).unwrap().modified().unwrap(), modified);
    assert_eq!(
        listing(),
        files,
        "no -wal/-shm or other files next to the source"
    );
}

#[test]
fn pending_wal_content_is_part_of_the_copy() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let connection = Connection::open(&source).unwrap();
    let mode: String = connection
        .query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
    connection
        .execute(
            "INSERT INTO PatientOwner(Id, OwnerName, OwnerPhone) VALUES (?1, ?2, ?3)",
            params![50, "Synthetic Wal Owner", 5329998877i64],
        )
        .unwrap();
    // The new row lives only in the -wal file while this connection stays open.
    let wal = std::path::PathBuf::from(format!("{}-wal", source.display()));
    assert!(fs::metadata(&wal).unwrap().len() > 0);

    let prepared = import::prepare(&source, &env.work()).unwrap();
    let mut target = env.target();
    let preview = import::preview(&prepared, &target).unwrap();
    assert_eq!(preview.found.owners, 4);
    drop(connection);
    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(report.inserted.owners, 4);
}

#[test]
fn duplicate_looking_owners_are_reported_not_skipped() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let mut target = env.target();
    target
        .conn()
        .execute(
            "INSERT INTO owners (full_name, phone) VALUES ('SYNTHETIC OWNER ALPHA', '+905321234567')",
            [],
        )
        .unwrap();
    let prepared = import::prepare(&source, &env.work()).unwrap();
    let preview = import::preview(&prepared, &target).unwrap();
    assert_eq!(preview.duplicate_owners, 1);
    assert!(preview
        .issues
        .iter()
        .any(|i| i.kind == ImportIssueKind::DuplicateOwner));
    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();
    assert_eq!(
        report.inserted.owners, 3,
        "possible duplicates are still imported"
    );
}

fn forbidden_tokens() -> [&'static str; 9] {
    [
        "Synthetic Owner Alpha",
        "Synthetic Patient",
        "Synthetic Address",
        "Synthetic private note",
        "Synthetic reminder",
        "SyntheticFreeSpecies",
        "5321234567",
        "532 765",
        "call after noon",
    ]
}

#[test]
fn default_output_has_aggregates_only() {
    let env = Env::new();
    let mut spec = sample();
    spec.patients[0].birth_date = Some("someday soon".into());
    let source = env.legacy(&spec);
    let mut target = env.target();
    let prepared = import::prepare(&source, &env.work()).unwrap();
    let preview = import::preview(&prepared, &target).unwrap();
    let report = import::execute(&prepared, &mut target, &env.backups()).unwrap();

    let outputs = vec![
        format_preview(&preview, false),
        format_report(&report, false),
        serde_json::to_string_pretty(&preview.without_values()).unwrap(),
        serde_json::to_string_pretty(&report.without_values()).unwrap(),
    ];
    for output in &outputs {
        for token in forbidden_tokens()
            .into_iter()
            .chain(["someday soon", "synthetic-legacy"])
        {
            assert!(!output.contains(token), "aggregate output leaked {token:?}");
        }
    }
    assert!(outputs[0].contains("Found: 3 owners, 3 patients, 3 to-dos"));
    assert!(outputs[0].contains("dateUnparsed: 1"));

    // The opt-in output does show the raw values.
    let shown = format_preview(&preview, true);
    assert!(shown.contains("someday soon"));
    assert!(shown.contains("synthetic-legacy.db"));
}

#[test]
fn non_legacy_files_are_rejected() {
    let env = Env::new();
    let folder = env.dir.path().join("files");
    fs::create_dir_all(&folder).unwrap();

    let text = folder.join("notes.txt");
    fs::write(&text, "this is not a database, not even close to one").unwrap();
    let empty = folder.join("empty.db");
    fs::write(&empty, b"").unwrap();
    let garbage = folder.join("garbage.db");
    let mut bytes = b"SQLite format 3\0".to_vec();
    bytes.extend(std::iter::repeat_n(0xAB, 4096));
    fs::write(&garbage, bytes).unwrap();

    let other = folder.join("other.db");
    let connection = Connection::open(&other).unwrap();
    connection
        .execute_batch("CREATE TABLE Recipes(Id INTEGER); INSERT INTO Recipes VALUES (1);")
        .unwrap();
    drop(connection);
    let baytar = folder.join("baytar.db");
    drop(Database::open(&baytar, &env.backups()).unwrap());

    for path in [&text, &empty, &garbage, &other, &baytar] {
        let error = import::prepare(path, &env.work()).unwrap_err();
        assert!(
            matches!(error, LegacyError::NotLegacyDatabase),
            "{}: {error}",
            path.display()
        );
        assert!(error.to_string().contains("not a PatientsCare database"));
    }
    let missing = import::prepare(&folder.join("missing.db"), &env.work()).unwrap_err();
    assert!(matches!(missing, LegacyError::Io(_)), "{missing}");
    // Rejected files are never copied.
    assert!(fs::read_dir(env.work())
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true));
}

#[test]
fn the_working_copy_is_removed_after_use() {
    let env = Env::new();
    let source = env.legacy(&sample());
    let prepared = import::prepare(&source, &env.work()).unwrap();
    assert_eq!(fs::read_dir(env.work()).unwrap().count(), 1);
    drop(prepared);
    assert_eq!(fs::read_dir(env.work()).unwrap().count(), 0);
}
