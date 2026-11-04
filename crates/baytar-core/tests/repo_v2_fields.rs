//! Repositories read and write the schema v2 columns. All data is synthetic.

use baytar_core::{
    db::Database,
    model::MedicationCategory,
    repo::{
        appointments::{self, AppointmentPatch, NewAppointment},
        medications::{self, MedicationPatch, NewMedication},
        owners::{self, NewOwner},
        patients::{self, NewPatient},
        vaccinations::{self, NewVaccination},
        vaccine_types::{self, NewVaccineType, VaccineTypePatch},
        visits::{self, NewVisit, VisitPatch},
        RepoError,
    },
};
use serde_json::json;

fn patient(conn: &rusqlite::Connection, name: &str) -> (i64, i64) {
    let o = owners::create(
        conn,
        NewOwner {
            full_name: format!("Owner of {name}"),
            ..Default::default()
        },
    )
    .unwrap();
    let p = patients::create(
        conn,
        NewPatient {
            owner_id: o.id,
            name: name.into(),
            species: "other".into(),
            ..Default::default()
        },
    )
    .unwrap();
    (o.id, p.id)
}

#[test]
fn visit_vitals_round_trip_and_validation() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let (_, p) = patient(c, "Synthetic Patient");
    let v = visits::create(
        c,
        NewVisit {
            patient_id: p,
            visit_on: "2026-11-04".into(),
            heart_rate_bpm: Some(120),
            resp_rate_bpm: Some(30),
            body_condition: Some(5),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (v.heart_rate_bpm, v.resp_rate_bpm, v.body_condition),
        (Some(120), Some(30), Some(5))
    );
    let v = visits::update(
        c,
        v.id,
        serde_json::from_value(json!({"heartRateBpm": null, "bodyCondition": 9})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (v.heart_rate_bpm, v.resp_rate_bpm, v.body_condition),
        (None, Some(30), Some(9))
    );
    for bad in [
        json!({"bodyCondition": 0}),
        json!({"bodyCondition": 10}),
        json!({"heartRateBpm": 0}),
        json!({"respRateBpm": -4}),
    ] {
        let patch: VisitPatch = serde_json::from_value(bad).unwrap();
        assert!(matches!(
            visits::update(c, v.id, patch),
            Err(RepoError::Invalid { .. })
        ));
    }
    assert_eq!(visits::get(c, v.id).unwrap().body_condition, Some(9));
}

#[test]
fn vaccine_type_catalog_fields_round_trip_and_validation() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let t = vaccine_types::create(
        c,
        NewVaccineType {
            name: "Synthetic Reptile Vaccine".into(),
            species: Some("reptile".into()),
            manufacturer: Some("Synthetic Lab".into()),
            diseases: Some("A, B".into()),
            primary_doses: Some(2),
            primary_interval_days: Some(21),
            min_age_weeks: Some(0),
            default_interval_days: Some(180),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!t.is_builtin);
    assert_eq!(t.species, Some(baytar_core::model::Species::Reptile));
    assert_eq!(t.manufacturer.as_deref(), Some("Synthetic Lab"));
    assert_eq!(
        (t.primary_doses, t.primary_interval_days, t.min_age_weeks),
        (Some(2), Some(21), Some(0))
    );
    let t = vaccine_types::update(
        c,
        t.id,
        serde_json::from_value(json!({"primaryDoses": null, "minAgeWeeks": 8})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (t.primary_doses, t.primary_interval_days, t.min_age_weeks),
        (None, Some(21), Some(8))
    );
    assert_eq!(
        t.default_interval_days,
        Some(180),
        "booster interval untouched"
    );
    // is_builtin is owned by the catalog seed and is preserved by edits.
    c.execute("UPDATE vaccine_types SET is_builtin = 1", [])
        .unwrap();
    let t = vaccine_types::update(
        c,
        t.id,
        VaccineTypePatch {
            notes: Some(Some("edited".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(t.is_builtin);
    for bad in [
        json!({"primaryDoses": 0}),
        json!({"primaryIntervalDays": 0}),
        json!({"minAgeWeeks": -1}),
        json!({"species": "dragon"}),
    ] {
        let patch: VaccineTypePatch = serde_json::from_value(bad).unwrap();
        assert!(matches!(
            vaccine_types::update(c, t.id, patch),
            Err(RepoError::Invalid { .. })
        ));
    }
}

#[test]
fn medication_catalog_fields_round_trip_and_validation() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let m = medications::create(
        c,
        NewMedication {
            name: "Synthetic Drug".into(),
            category: Some("antibiotic".into()),
            manufacturer: Some("Synthetic Pharma".into()),
            target_species: Some("Köpek, Kedi".into()),
            prescription_required: Some(true),
            pack_unit: Some("50 ml flacon".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!m.is_builtin);
    assert_eq!(m.category, Some(MedicationCategory::Antibiotic));
    assert_eq!(m.prescription_required, Some(true));
    assert_eq!(m.target_species.as_deref(), Some("Köpek, Kedi"));
    let m = medications::update(
        c,
        m.id,
        MedicationPatch {
            category: Some(None),
            prescription_required: Some(Some(false)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(m.category, None);
    assert_eq!(m.prescription_required, Some(false));
    assert_eq!(m.pack_unit.as_deref(), Some("50 ml flacon"));
    let bad: MedicationPatch = serde_json::from_value(json!({"category": "poison"})).unwrap();
    assert!(matches!(
        medications::update(c, m.id, bad),
        Err(RepoError::Invalid { .. })
    ));
    let unset = medications::create(
        c,
        NewMedication {
            name: "No metadata".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(unset.prescription_required, None, "unknown stays NULL");
}

#[test]
fn appointment_vaccine_links_round_trip_and_validation() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let (owner, p) = patient(c, "Synthetic Patient");
    let (_, other_patient) = patient(c, "Other Patient");
    let t = vaccine_types::create(
        c,
        NewVaccineType {
            name: "Synthetic Vaccine".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let a = appointments::create(
        c,
        NewAppointment {
            patient_id: Some(p),
            owner_id: Some(owner),
            title: "Booster".into(),
            starts_on: "2026-12-01".into(),
            kind: "vaccine".into(),
            vaccine_type_id: Some(t.id),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((a.vaccine_type_id, a.vaccination_id), (Some(t.id), None));
    let given = vaccinations::create(
        c,
        serde_json::from_value::<NewVaccination>(json!({
            "patientId": p,
            "vaccineTypeId": t.id,
            "vaccineName": "Synthetic Vaccine",
            "givenOn": "2026-12-01"
        }))
        .unwrap(),
    )
    .unwrap();
    let done = appointments::update(
        c,
        a.id,
        AppointmentPatch {
            status: Some("done".into()),
            vaccination_id: Some(Some(given.id)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(done.vaccination_id, Some(given.id));
    assert_eq!(done.vaccine_type_id, Some(t.id), "unrelated link kept");

    // The calendar query still pairs names with the right columns.
    let day = appointments::between(c, "2026-12-01", "2026-12-01").unwrap();
    assert_eq!(day[0].patient_name.as_deref(), Some("Synthetic Patient"));
    assert_eq!(
        day[0].owner_name.as_deref(),
        Some("Owner of Synthetic Patient")
    );
    assert_eq!(day[0].appointment.vaccination_id, Some(given.id));

    let cleared = appointments::update(
        c,
        a.id,
        serde_json::from_value(json!({"vaccineTypeId": null})).unwrap(),
    )
    .unwrap();
    assert_eq!(cleared.vaccine_type_id, None);

    // A vaccination of another patient, or a missing vaccine type, is refused.
    let wrong = appointments::update(
        c,
        a.id,
        AppointmentPatch {
            patient_id: Some(Some(other_patient)),
            owner_id: Some(None),
            ..Default::default()
        },
    );
    assert!(matches!(wrong, Err(RepoError::Invalid { .. })), "{wrong:?}");
    let missing = appointments::update(
        c,
        a.id,
        AppointmentPatch {
            vaccine_type_id: Some(Some(9999)),
            ..Default::default()
        },
    );
    assert!(matches!(missing, Err(RepoError::NotFound)), "{missing:?}");
}
