use baytar_core::{
    db::Database,
    model::AppointmentStatus,
    repo::{
        appointments::{self, AppointmentPatch, NewAppointment},
        medications::{self, MedicationPatch, NewMedication},
        owners::{self, NewOwner},
        patients::{self, NewPatient, PatientPatch},
        prescriptions::{self, NewPrescription, PrescriptionPatch},
        vaccinations::{self, NewVaccination, VaccinationPatch},
        vaccine_types::{self, NewVaccineType, VaccineTypePatch},
        visits::{self, NewVisit, VisitPatch},
        RepoError,
    },
};
use rusqlite::Connection;
use serde_json::json;

fn patient(c: &Connection) -> i64 {
    let owner = owners::create(
        c,
        NewOwner {
            full_name: "Synthetic Owner".into(),
            ..Default::default()
        },
    )
    .unwrap();
    patients::create(
        c,
        NewPatient {
            owner_id: owner.id,
            name: "Synthetic Cat".into(),
            species: "cat".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id
}
fn visit(
    c: &Connection,
    patient_id: i64,
    date: &str,
    weight: Option<f64>,
) -> baytar_core::model::Visit {
    visits::create(
        c,
        NewVisit {
            patient_id,
            visit_on: date.into(),
            weight_kg: weight,
            ..Default::default()
        },
    )
    .unwrap()
}
fn vaccine(
    c: &Connection,
    patient_id: i64,
    name: &str,
    given: &str,
    due: &str,
) -> baytar_core::model::Vaccination {
    vaccinations::create(
        c,
        NewVaccination {
            patient_id,
            vaccine_name: name.into(),
            given_on: given.into(),
            next_due_on: Some(due.into()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn appointment(patient_id: Option<i64>) -> NewAppointment {
    NewAppointment {
        patient_id,
        title: "Synthetic appointment".into(),
        starts_on: "2026-12-30".into(),
        starts_time: Some("09:15".into()),
        ..Default::default()
    }
}
fn invalid<T: std::fmt::Debug>(result: Result<T, RepoError>, expected: &str) {
    assert!(
        matches!(result, Err(RepoError::Invalid { ref field, .. }) if field == expected),
        "{result:?}"
    );
}

#[test]
fn visits_crud_nullable_patch_and_latest_weight() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    assert!(visits::latest_weight(c, p).unwrap().is_none());
    let first = visit(c, p, "2026-01-01", Some(4.0));
    let second = visit(c, p, "2026-02-01", Some(4.5));
    let newest = visit(c, p, "2026-03-01", None);
    assert_eq!(
        visits::list_by_patient(c, p)
            .unwrap()
            .iter()
            .map(|v| v.id)
            .collect::<Vec<_>>(),
        vec![newest.id, second.id, first.id]
    );
    let latest = visits::latest_weight(c, p).unwrap().unwrap();
    assert_eq!(latest.weight_kg, 4.5);
    assert_eq!(latest.visit_on, "2026-02-01");
    let changed = visits::update(
        c,
        second.id,
        VisitPatch {
            reason: Some(Some("Follow-up".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(changed.weight_kg, Some(4.5));
    assert_ne!(changed.updated_at, second.updated_at);
    let patch: VisitPatch = serde_json::from_value(json!({"weightKg":null})).unwrap();
    assert_eq!(
        serde_json::to_value(&patch).unwrap(),
        json!({"weightKg":null})
    );
    visits::update(c, second.id, patch).unwrap();
    assert_eq!(visits::latest_weight(c, p).unwrap().unwrap().weight_kg, 4.0);
    visits::soft_delete(c, first.id).unwrap();
    assert!(visits::get(c, first.id).unwrap().deleted_at.is_some());
    assert!(visits::latest_weight(c, p).unwrap().is_none());
    assert!(matches!(
        visits::soft_delete(c, first.id),
        Err(RepoError::Conflict)
    ));
    assert!(matches!(
        visits::update(c, first.id, VisitPatch::default()),
        Err(RepoError::NotFound)
    ));
    visits::restore(c, first.id).unwrap();
    assert_eq!(visits::latest_weight(c, p).unwrap().unwrap().weight_kg, 4.0);
    assert!(matches!(
        visits::restore(c, first.id),
        Err(RepoError::Conflict)
    ));
    invalid(
        visits::update(
            c,
            first.id,
            VisitPatch {
                weight_kg: Some(Some(0.0)),
                ..Default::default()
            },
        ),
        "weightKg",
    );
    invalid(
        visits::update(
            c,
            first.id,
            VisitPatch {
                weight_kg: Some(Some(f64::NAN)),
                ..Default::default()
            },
        ),
        "weightKg",
    );
    invalid(
        visits::update(
            c,
            first.id,
            VisitPatch {
                temperature_c: Some(Some(f64::INFINITY)),
                ..Default::default()
            },
        ),
        "temperatureC",
    );
}

#[test]
fn dates_and_times_require_exact_valid_calendar_values() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    for date in [
        "2026-02-29",
        "2026-2-01",
        "2026-04-31",
        "2026-01-01T12:00:00",
        "bad",
    ] {
        invalid(
            visits::create(
                c,
                NewVisit {
                    patient_id: p,
                    visit_on: date.into(),
                    ..Default::default()
                },
            ),
            "visitOn",
        );
        invalid(
            appointments::create(
                c,
                NewAppointment {
                    starts_on: date.into(),
                    ..appointment(None)
                },
            ),
            "startsOn",
        );
    }
    for time in ["24:00", "12:60", "9:00", "12:00:00", "", "-1:00"] {
        invalid(
            visits::create(
                c,
                NewVisit {
                    patient_id: p,
                    visit_on: "2024-02-29".into(),
                    visit_time: Some(time.into()),
                    ..Default::default()
                },
            ),
            "visitTime",
        );
        invalid(
            appointments::create(
                c,
                NewAppointment {
                    starts_time: Some(time.into()),
                    ..appointment(None)
                },
            ),
            "startsTime",
        );
    }
    for time in [None, Some("00:00"), Some("23:59")] {
        appointments::create(
            c,
            NewAppointment {
                starts_time: time.map(str::to_string),
                ..appointment(None)
            },
        )
        .unwrap();
    }
    visit(c, p, "2024-02-29", None);
}

#[test]
fn vaccine_catalog_seeds_once_and_supports_crud() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let backups = dir.path().join("backups");
    let db = Database::open(&path, &backups).unwrap();
    let c = db.conn();
    vaccine_types::seed(c).unwrap();
    let seeded = vaccine_types::list(c).unwrap();
    assert_eq!(seeded.len(), 6);
    for (name, days, species) in [
        ("Karma aşı", 365, Some("dog")),
        ("Kuduz aşısı", 365, None),
        ("İç parazit", 90, None),
        ("Dış parazit", 30, None),
        ("Kedi karma aşısı", 365, Some("cat")),
        ("Lösemi aşısı (FeLV)", 365, Some("cat")),
    ] {
        let row = seeded.iter().find(|v| v.name == name).unwrap();
        assert_eq!(row.default_interval_days, Some(days));
        assert_eq!(row.species.map(|s| s.as_str()), species);
    }
    let v = vaccine_types::create(
        c,
        NewVaccineType {
            name: "Synthetic vaccine".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let changed = vaccine_types::update(
        c,
        v.id,
        VaccineTypePatch {
            is_active: Some(false),
            notes: Some(Some("Synthetic notes".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!changed.is_active);
    assert_ne!(changed.updated_at, v.updated_at);
    assert_eq!(vaccine_types::list(c).unwrap().last().unwrap().id, v.id);
    vaccine_types::soft_delete(c, v.id).unwrap();
    assert_eq!(vaccine_types::get(c, v.id).unwrap().name, v.name);
    let replacement = vaccine_types::create(
        c,
        NewVaccineType {
            name: v.name.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vaccine_types::restore(c, v.id),
        Err(RepoError::Conflict)
    ));
    vaccine_types::soft_delete(c, replacement.id).unwrap();
    vaccine_types::restore(c, v.id).unwrap();
    for row in vaccine_types::list(c).unwrap() {
        vaccine_types::soft_delete(c, row.id).unwrap();
    }
    drop(db);
    let db = Database::open(&path, &backups).unwrap();
    assert!(vaccine_types::list(db.conn()).unwrap().is_empty());
    // Even an empty table must not cause re-seeding after the marker is recorded.
    db.conn().execute("DELETE FROM vaccine_types", []).unwrap();
    vaccine_types::seed(db.conn()).unwrap();
    assert!(vaccine_types::list(db.conn()).unwrap().is_empty());
}

#[test]
fn preexisting_catalog_is_preserved_and_marked() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    vaccine_types::create(
        c,
        NewVaccineType {
            name: "Existing".into(),
            ..Default::default()
        },
    )
    .unwrap();
    vaccine_types::seed(c).unwrap();
    assert_eq!(vaccine_types::list(c).unwrap().len(), 1);
    assert_eq!(
        c.query_row(
            "SELECT value FROM settings WHERE key = 'seeded_vaccine_types'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "1"
    );
}

#[test]
fn next_due_is_only_a_suggestion_and_handles_leap_years() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let v = vaccine_types::create(
        c,
        NewVaccineType {
            name: "Synthetic".into(),
            default_interval_days: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        vaccinations::suggest_next_due(c, v.id, "2024-02-28")
            .unwrap()
            .as_deref(),
        Some("2024-02-29")
    );
    assert_eq!(
        vaccinations::suggest_next_due(c, v.id, "2026-12-31")
            .unwrap()
            .as_deref(),
        Some("2027-01-01")
    );
    let row = vaccinations::create(
        c,
        NewVaccination {
            patient_id: p,
            vaccine_type_id: Some(v.id),
            vaccine_name: v.name,
            given_on: "2024-02-28".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(row.next_due_on.is_none());
    let updated = vaccinations::update(
        c,
        row.id,
        VaccinationPatch {
            next_due_on: Some(Some("2024-02-29".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(updated.next_due_on.as_deref(), Some("2024-02-29"));
    assert_ne!(updated.updated_at, row.updated_at);
    assert_eq!(vaccinations::list_by_patient(c, p).unwrap().len(), 1);
    vaccinations::soft_delete(c, row.id).unwrap();
    assert!(vaccinations::list_by_patient(c, p).unwrap().is_empty());
    assert!(vaccinations::get(c, row.id).unwrap().deleted_at.is_some());
    vaccinations::restore(c, row.id).unwrap();
    vaccine_types::update(
        c,
        v.id,
        VaccineTypePatch {
            default_interval_days: Some(None),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(vaccinations::suggest_next_due(c, v.id, "2024-02-28")
        .unwrap()
        .is_none());
    vaccine_types::update(
        c,
        v.id,
        VaccineTypePatch {
            default_interval_days: Some(Some(i64::MAX)),
            ..Default::default()
        },
    )
    .unwrap();
    invalid(
        vaccinations::suggest_next_due(c, v.id, "2024-02-28"),
        "defaultIntervalDays",
    );
    invalid(
        vaccinations::suggest_next_due(c, v.id, "2026-02-29"),
        "givenOn",
    );
}

#[test]
fn due_boundaries_supersession_and_patient_lifecycle() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let yesterday = vaccine(c, p, "Yesterday", "2025-01-01", "2026-11-03");
    let today = vaccine(c, p, "Today", "2025-01-01", "2026-11-04");
    let tomorrow = vaccine(c, p, "Tomorrow", "2025-01-01", "2026-11-05");
    let rows = vaccinations::due_between(c, "2026-11-03", "2026-11-05").unwrap();
    assert_eq!(
        rows.iter().map(|r| r.vaccination.id).collect::<Vec<_>>(),
        vec![yesterday.id, today.id, tomorrow.id]
    );
    assert_eq!(rows[0].patient_name, "Synthetic Cat");
    assert_eq!(rows[0].owner_name, "Synthetic Owner");
    assert_eq!(
        vaccinations::due_between(c, "2026-11-04", "2026-11-04").unwrap()[0]
            .vaccination
            .id,
        today.id
    );
    assert_eq!(
        vaccinations::overdue(c, "2026-11-04")
            .unwrap()
            .iter()
            .map(|r| r.vaccination.id)
            .collect::<Vec<_>>(),
        vec![yesterday.id]
    );
    let later = vaccine(c, p, "Yesterday", "2026-11-02", "2027-11-02");
    assert!(vaccinations::overdue(c, "2026-11-04").unwrap().is_empty());
    vaccinations::soft_delete(c, later.id).unwrap();
    assert_eq!(vaccinations::overdue(c, "2026-11-04").unwrap().len(), 1);
    vaccinations::restore(c, later.id).unwrap();
    patients::update(
        c,
        p,
        PatientPatch {
            deceased_on: Some(Some("2026-11-04".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(vaccinations::due_between(c, "2026-01-01", "2028-01-01")
        .unwrap()
        .is_empty());
    assert!(vaccinations::overdue(c, "2028-01-01").unwrap().is_empty());
    patients::update(
        c,
        p,
        PatientPatch {
            deceased_on: Some(None),
            ..Default::default()
        },
    )
    .unwrap();
    patients::soft_delete(c, p).unwrap();
    assert!(vaccinations::due_between(c, "2026-01-01", "2028-01-01")
        .unwrap()
        .is_empty());
    assert!(vaccinations::overdue(c, "2028-01-01").unwrap().is_empty());
    patients::restore(c, p).unwrap();
    assert_eq!(
        vaccinations::due_between(c, "2026-01-01", "2028-01-01")
            .unwrap()
            .len(),
        3
    );
    invalid(
        vaccinations::due_between(c, "2026-11-05", "2026-11-04"),
        "to",
    );
    invalid(vaccinations::overdue(c, "2026-02-29"), "today");
}

#[test]
fn supersession_uses_type_even_when_snapshot_name_changes() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let t = vaccine_types::create(
        c,
        NewVaccineType {
            name: "Original".into(),
            ..Default::default()
        },
    )
    .unwrap();
    for (name, given, due) in [
        ("Original", "2025-01-01", "2026-01-01"),
        ("Renamed", "2026-01-01", "2027-01-01"),
    ] {
        vaccinations::create(
            c,
            NewVaccination {
                patient_id: p,
                vaccine_type_id: Some(t.id),
                vaccine_name: name.into(),
                given_on: given.into(),
                next_due_on: Some(due.into()),
                ..Default::default()
            },
        )
        .unwrap();
    }
    assert!(vaccinations::overdue(c, "2026-11-04").unwrap().is_empty());
    assert!(vaccinations::due_between(c, "2026-01-01", "2026-11-04")
        .unwrap()
        .is_empty());
}

#[test]
fn medication_crud_folded_search_and_dose_validation() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let m = medications::create(
        c,
        NewMedication {
            name: "İlaç".into(),
            active_ingredient: Some("Çinko".into()),
            form: Some("tablet".into()),
            dose_mg_per_kg_min: Some(1.0),
            dose_mg_per_kg_max: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    let other = medications::create(
        c,
        NewMedication {
            name: "A synthetic item".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(medications::list(c).unwrap()[0].id, other.id);
    for query in ["ILAC", "İLAÇ", "cinko", "ÇİNKO"] {
        assert_eq!(medications::search(c, query).unwrap()[0].id, m.id);
    }
    let changed = medications::update(
        c,
        m.id,
        MedicationPatch {
            notes: Some(Some("Instructions".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_ne!(changed.updated_at, m.updated_at);
    let patch: MedicationPatch = serde_json::from_value(json!({"activeIngredient":null})).unwrap();
    medications::update(c, m.id, patch).unwrap();
    assert!(medications::search(c, "cinko").unwrap().is_empty());
    invalid(
        medications::update(
            c,
            m.id,
            MedicationPatch {
                dose_mg_per_kg_max: Some(Some(0.5)),
                ..Default::default()
            },
        ),
        "doseMgPerKgMax",
    );
    invalid(
        medications::update(
            c,
            m.id,
            MedicationPatch {
                form: Some(Some("invalid".into())),
                ..Default::default()
            },
        ),
        "form",
    );
    medications::soft_delete(c, m.id).unwrap();
    assert_eq!(medications::list(c).unwrap().len(), 1);
    assert!(medications::get(c, m.id).unwrap().deleted_at.is_some());
    medications::restore(c, m.id).unwrap();
    assert_eq!(medications::list(c).unwrap().len(), 2);
}

#[test]
fn prescription_crud_preserves_medication_snapshot() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let v = visit(c, p, "2026-11-04", None);
    let m = medications::create(
        c,
        NewMedication {
            name: "Original name".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let input: NewPrescription =
        serde_json::from_value(json!({"visitId":v.id,"medicationId":m.id})).unwrap();
    let item = prescriptions::create(c, input).unwrap();
    assert_eq!(item.medication_name, "Original name");
    medications::update(
        c,
        m.id,
        MedicationPatch {
            name: Some("Renamed".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let changed = prescriptions::update(
        c,
        item.id,
        PrescriptionPatch {
            dose_text: Some(Some("Synthetic instructions".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(changed.medication_name, "Original name");
    assert_ne!(changed.updated_at, item.updated_at);
    assert_eq!(prescriptions::list_by_visit(c, v.id).unwrap().len(), 1);
    prescriptions::soft_delete(c, item.id).unwrap();
    assert!(prescriptions::list_by_visit(c, v.id).unwrap().is_empty());
    assert!(prescriptions::get(c, item.id).unwrap().deleted_at.is_some());
    prescriptions::restore(c, item.id).unwrap();
    let second = medications::create(
        c,
        NewMedication {
            name: "Second medicine".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let changed = prescriptions::update(
        c,
        item.id,
        PrescriptionPatch {
            medication_id: Some(Some(second.id)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(changed.medication_name, "Second medicine");
    invalid(
        prescriptions::update(
            c,
            item.id,
            PrescriptionPatch {
                duration_days: Some(Some(-1)),
                ..Default::default()
            },
        ),
        "durationDays",
    );
}

#[test]
fn appointments_crud_status_series_dates_and_atomicity() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let a = appointments::create(c, appointment(Some(p))).unwrap();
    let changed = appointments::update(
        c,
        a.id,
        AppointmentPatch {
            notes: Some(Some("Synthetic notes".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_ne!(changed.updated_at, a.updated_at);
    assert_eq!(
        appointments::set_status(c, a.id, "done").unwrap().status,
        AppointmentStatus::Done
    );
    invalid(appointments::set_status(c, a.id, "unknown"), "status");
    let joined = appointments::today(c, "2026-12-30").unwrap();
    assert_eq!(joined[0].patient_name.as_deref(), Some("Synthetic Cat"));
    assert_eq!(joined[0].owner_name.as_deref(), Some("Synthetic Owner"));
    appointments::soft_delete(c, a.id).unwrap();
    assert!(appointments::today(c, "2026-12-30").unwrap().is_empty());
    assert!(appointments::get(c, a.id).unwrap().deleted_at.is_some());
    appointments::restore(c, a.id).unwrap();
    let rows = appointments::create_series(c, appointment(Some(p)), 3, 2).unwrap();
    assert_eq!(
        rows.iter()
            .map(|a| a.starts_on.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-12-30", "2027-01-01", "2027-01-03"]
    );
    assert!(rows[0].series_id.is_some());
    assert!(rows.iter().all(|a| a.series_id == rows[0].series_id));
    assert_eq!(
        appointments::between(c, "2026-12-31", "2027-01-03")
            .unwrap()
            .len(),
        2
    );
    for count in [0, 53] {
        invalid(
            appointments::create_series(c, appointment(None), count, 1),
            "count",
        );
    }
    invalid(
        appointments::create_series(c, appointment(None), 2, 0),
        "everyDays",
    );
    invalid(
        appointments::create_series(c, appointment(None), 3, i64::MAX),
        "everyDays",
    );
    invalid(
        appointments::create_series(
            c,
            NewAppointment {
                starts_on: "9999-12-31".into(),
                ..appointment(None)
            },
            2,
            1,
        ),
        "startsOn",
    );
    assert_eq!(
        appointments::between(c, "0001-01-01", "9999-12-31")
            .unwrap()
            .len(),
        4
    );
    let standalone = appointments::create(c, appointment(None)).unwrap();
    let rows = appointments::today(c, &standalone.starts_on).unwrap();
    let row = rows
        .iter()
        .find(|a| a.appointment.id == standalone.id)
        .unwrap();
    assert!(row.patient_name.is_none());
    assert!(row.owner_name.is_none());
}

#[test]
fn linked_records_must_agree() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let p = patient(c);
    let other = patient(c);
    let v = visit(c, p, "2026-11-04", None);
    invalid(
        vaccinations::create(
            c,
            NewVaccination {
                patient_id: other,
                visit_id: Some(v.id),
                vaccine_name: "Synthetic".into(),
                given_on: "2026-11-04".into(),
                ..Default::default()
            },
        ),
        "visitId",
    );
    invalid(
        appointments::create(
            c,
            NewAppointment {
                owner_id: Some(patients::get(c, other).unwrap().owner_id),
                ..appointment(Some(p))
            },
        ),
        "ownerId",
    );
    vaccinations::create(
        c,
        NewVaccination {
            patient_id: p,
            visit_id: Some(v.id),
            vaccine_name: "Synthetic".into(),
            given_on: "2026-11-04".into(),
            ..Default::default()
        },
    )
    .unwrap();
    invalid(
        visits::update(
            c,
            v.id,
            VisitPatch {
                patient_id: Some(other),
                ..Default::default()
            },
        ),
        "patientId",
    );
    patients::soft_delete(c, p).unwrap();
    assert!(matches!(
        appointments::create(c, appointment(Some(p))),
        Err(RepoError::NotFound)
    ));
}
