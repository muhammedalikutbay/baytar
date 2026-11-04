use baytar_core::{
    db::Database,
    repo::{
        owners::{self, NewOwner, OwnerPatch},
        patients::{self, NewPatient, PatientPatch},
        search::search,
        ListQuery, RepoError,
    },
};
use rusqlite::Connection;
use serde_json::json;
fn owner(conn: &Connection, name: &str) -> baytar_core::model::Owner {
    owners::create(
        conn,
        NewOwner {
            full_name: name.into(),
            ..Default::default()
        },
    )
    .unwrap()
}
fn pet(conn: &Connection, id: i64, name: &str) -> baytar_core::model::Patient {
    patients::create(
        conn,
        NewPatient {
            owner_id: id,
            name: name.into(),
            species: "cat".into(),
            ..Default::default()
        },
    )
    .unwrap()
}
#[test]
fn crud_and_nullable_patches_preserve_raw_phone() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owners::create(
        c,
        NewOwner {
            full_name: "Çağrı Şimşek".into(),
            phone: Some("+90 (532) 123-45-67".into()),
            phone_alt: Some("0544 765 43 21".into()),
            email: Some("synthetic@example.test".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(o.phone.as_deref(), Some("+905321234567"));
    assert_eq!(o.phone_raw.as_deref(), Some("+90 (532) 123-45-67"));
    assert_eq!(o.phone_alt.as_deref(), Some("+905447654321"));
    assert_eq!(
        o.search_text,
        "cagri simsek 905321234567 05321234567 905447654321 05447654321"
    );
    let patch: OwnerPatch =
        serde_json::from_value(json!({"fullName":"İpek Işık", "email":null})).unwrap();
    let o2 = owners::update(c, o.id, patch).unwrap();
    assert_eq!(o2.email, None);
    assert_eq!(o2.phone_raw, o.phone_raw);
    assert_ne!(o2.updated_at, o.updated_at);
    assert!(o2.search_text.starts_with("ipek isik"));
    let o3 = owners::update(
        c,
        o.id,
        serde_json::from_value(json!({"phone":null})).unwrap(),
    )
    .unwrap();
    assert_eq!(o3.phone, None);
    assert_eq!(o3.phone_raw, None);
    let p = pet(c, o.id, "Pamuk");
    let updated = patients::update(c, p.id, serde_json::from_value(json!({"breed":"İran", "birthDate":"2024-02-29", "neutered":true, "speciesNote":"Uzun tüylü"})).unwrap()).unwrap();
    assert_eq!(updated.search_text, "pamuk iran uzun tuylu");
    assert_eq!(updated.neutered, Some(true));
    let cleared = patients::update(
        c,
        p.id,
        serde_json::from_value(json!({"breed":null, "neutered":null})).unwrap(),
    )
    .unwrap();
    assert_eq!(cleared.breed, None);
    assert_eq!(cleared.neutered, None);
    assert_eq!(cleared.birth_date, updated.birth_date);
    assert_eq!(owners::get(c, o.id).unwrap(), o3);
    assert_eq!(patients::get(c, p.id).unwrap(), cleared);
    let detail = owners::owner_detail(c, o.id).unwrap();
    assert_eq!(detail.patients.len(), 1);
    assert_eq!(owners::list(c, ListQuery::default()).unwrap().len(), 1);
}
#[test]
fn cascade_restore_recovers_exactly_the_patients_deleted_with_owner() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "Ayşe Işık");
    let old = pet(c, o.id, "Önceden");
    patients::soft_delete(c, old.id).unwrap();
    let a = pet(c, o.id, "Pamuk");
    let b = pet(c, o.id, "Boncuk");
    let ids = owners::soft_delete(c, o.id).unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&a.id) && ids.contains(&b.id));
    let deleted = owners::get(c, o.id).unwrap();
    assert_eq!(
        patients::get(c, a.id).unwrap().deleted_at,
        deleted.deleted_at
    );
    assert!(owners::list(c, ListQuery::default()).unwrap().is_empty());
    assert_eq!(
        owners::list(
            c,
            ListQuery {
                include_deleted: true,
                ..Default::default()
            }
        )
        .unwrap()
        .len(),
        1
    );
    assert!(patients::list_by_owner(c, o.id).unwrap().is_empty());
    assert!(search(c, "Pamuk", 50).unwrap().is_empty());
    assert!(matches!(
        patients::restore(c, a.id),
        Err(RepoError::NotFound)
    ));
    assert!(matches!(
        owners::soft_delete(c, o.id),
        Err(RepoError::Conflict)
    ));
    owners::restore(c, o.id).unwrap();
    assert_eq!(patients::list_by_owner(c, o.id).unwrap().len(), 2);
    assert!(patients::get(c, old.id).unwrap().deleted_at.is_some());
    patients::restore(c, old.id).unwrap();
    assert_eq!(patients::list_by_owner(c, o.id).unwrap().len(), 3);
    owners::soft_delete(c, o.id).unwrap();
    owners::restore(c, o.id).unwrap();
    assert_eq!(patients::list_by_owner(c, o.id).unwrap().len(), 3);
}
#[test]
fn search_folds_turkish_groups_pets_and_limits_owner_groups() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "Şimşek ıiİI");
    for name in ["Şimşek", "ŞİMŞEK", "simsek"] {
        pet(c, o.id, name);
    }
    for query in ["ŞİMŞEK", "Şimşek", "simsek", "  SİMSEK  "] {
        let hits = search(c, query, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].patients.len(), 3);
    }
    for query in ["ıiİI", "IIII", "iiii", "İİİİ"] {
        assert_eq!(search(c, query, 1).unwrap().len(), 1);
    }
    let only_pet = owner(c, "Zeynep");
    pet(c, only_pet.id, "Şimşek");
    let hits = search(c, "simsek", 1).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].owner.id, o.id);
    assert_eq!(search(c, "simsek", 10).unwrap().len(), 2);
    assert!(search(c, " ", 50).unwrap().is_empty());
    assert!(search(c, "simsek", 0).unwrap().is_empty());
    assert!(search(c, "%", 50).unwrap().is_empty());
    assert!(search(c, "_", 50).unwrap().is_empty());
}
#[test]
fn phone_substrings_and_prefix_ranking() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let a = owners::create(
        c,
        NewOwner {
            full_name: "Ayla".into(),
            phone: Some("0532 123 45 67".into()),
            phone_alt: Some("0544 987 65 43".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(search(c, "12345", 50).unwrap()[0].owner.id, a.id);
    assert_eq!(search(c, "98765", 50).unwrap()[0].owner.id, a.id);
    owner(c, "Aylin Şimşek");
    let prefix = owner(c, "Şimşek Zeynep");
    assert_eq!(search(c, "simsek", 50).unwrap()[0].owner.id, prefix.id);
    owner(c, "Cem");
    let listed = owners::list(
        c,
        ListQuery {
            limit: 1,
            offset: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(listed[0].full_name, "Aylin Şimşek");
}
#[test]
fn deleted_patient_excluded_from_search_and_detail() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "İpek");
    let p = pet(c, o.id, "Fındık");
    patients::soft_delete(c, p.id).unwrap();
    assert!(search(c, "findik", 10).unwrap().is_empty());
    assert!(owners::owner_detail(c, o.id).unwrap().patients.is_empty());
    assert!(matches!(
        patients::update(c, p.id, PatientPatch::default()),
        Err(RepoError::NotFound)
    ));
    patients::restore(c, p.id).unwrap();
    assert_eq!(search(c, "findik", 10).unwrap().len(), 1);
    assert!(matches!(
        patients::restore(c, p.id),
        Err(RepoError::Conflict)
    ));
}
#[test]
fn validation_and_missing_records_are_typed_and_atomic() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    assert!(
        matches!(owners::create(c, NewOwner { full_name: " ".into(), ..Default::default() }), Err(RepoError::Invalid { field, .. }) if field == "fullName")
    );
    let o = owner(c, "Deniz");
    let p = pet(c, o.id, "Mavi");
    for (patch, field) in [
        (json!({"name":" "}), "name"),
        (json!({"species":"dragon"}), "species"),
        (json!({"sex":"invalid"}), "sex"),
        (json!({"birthDate":"2023-02-29"}), "birthDate"),
        (json!({"deceasedOn":"2024-2-1"}), "deceasedOn"),
    ] {
        assert!(
            matches!(patients::update(c, p.id, serde_json::from_value(patch).unwrap()), Err(RepoError::Invalid {field: f, ..}) if f == field)
        );
        assert_eq!(patients::get(c, p.id).unwrap(), p);
    }
    assert!(matches!(
        patients::create(
            c,
            NewPatient {
                owner_id: 999,
                name: "Pet".into(),
                species: "cat".into(),
                ..Default::default()
            }
        ),
        Err(RepoError::NotFound)
    ));
    assert!(matches!(
        patients::update(
            c,
            p.id,
            PatientPatch {
                owner_id: Some(999),
                ..Default::default()
            }
        ),
        Err(RepoError::NotFound)
    ));
    assert!(matches!(owners::get(c, 999), Err(RepoError::NotFound)));
    assert!(matches!(patients::get(c, 999), Err(RepoError::NotFound)));
    owners::soft_delete(c, o.id).unwrap();
    assert!(matches!(
        patients::create(
            c,
            NewPatient {
                owner_id: o.id,
                name: "Pet".into(),
                species: "cat".into(),
                ..Default::default()
            }
        ),
        Err(RepoError::NotFound)
    ));
    assert!(matches!(
        owners::update(c, o.id, OwnerPatch::default()),
        Err(RepoError::NotFound)
    ));
}
#[test]
fn failed_cascade_rolls_back_owner_and_patients() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "Ece");
    let p = pet(c, o.id, "Tekir");
    c.execute_batch("CREATE TRIGGER reject_owner_delete BEFORE UPDATE OF deleted_at ON owners BEGIN SELECT RAISE(ABORT, 'synthetic failure'); END;").unwrap();
    assert!(owners::soft_delete(c, o.id).is_err());
    assert!(owners::get(c, o.id).unwrap().deleted_at.is_none());
    assert!(patients::get(c, p.id).unwrap().deleted_at.is_none());
    c.execute_batch("DROP TRIGGER reject_owner_delete").unwrap();
    owners::soft_delete(c, o.id).unwrap();
    c.execute_batch("CREATE TRIGGER reject_owner_restore BEFORE UPDATE OF deleted_at ON owners BEGIN SELECT RAISE(ABORT, 'synthetic failure'); END;").unwrap();
    assert!(owners::restore(c, o.id).is_err());
    assert!(owners::get(c, o.id).unwrap().deleted_at.is_some());
    assert!(patients::get(c, p.id).unwrap().deleted_at.is_some());
}
#[test]
fn nullable_patch_serde_roundtrip_keeps_omission_and_explicit_null() {
    let patch: OwnerPatch = serde_json::from_value(json!({"phone":null})).unwrap();
    assert_eq!(patch.phone, Some(None));
    assert_eq!(patch.email, None);
    assert_eq!(serde_json::to_value(patch).unwrap(), json!({"phone":null}));
    let input: NewPatient =
        serde_json::from_value(json!({"ownerId":1,"name":"Pet","species":"cat"})).unwrap();
    assert_eq!(input.sex, "unknown");
}

#[test]
fn unrelated_owner_edit_preserves_legacy_normalized_phone_and_raw_value() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "Synthetic Owner");
    c.execute(
        "UPDATE owners SET phone = '05321234567', phone_raw = '5321234567' WHERE id = ?1",
        [o.id],
    )
    .unwrap();
    let updated = owners::update(
        c,
        o.id,
        OwnerPatch {
            full_name: Some("Updated Owner".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(updated.phone.as_deref(), Some("05321234567"));
    assert_eq!(updated.phone_raw.as_deref(), Some("5321234567"));
    assert_eq!(updated.search_text, "updated owner 05321234567");
}

#[test]
fn owner_v2_fields_phone_forms_and_city_search() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owners::create(
        c,
        NewOwner {
            full_name: "Synthetic Ada".into(),
            phone: Some("0532 123 45 67".into()),
            phone_alt: Some("+49 151 1234 5678".into()),
            national_id: Some(" 12345678901 ".into()),
            city: Some("  Eskişehir ".into()),
            district: Some("   ".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(o.phone.as_deref(), Some("+905321234567"));
    assert_eq!(o.phone_raw.as_deref(), Some("0532 123 45 67"));
    assert_eq!(o.phone_alt.as_deref(), Some("+4915112345678"));
    assert_eq!(o.national_id.as_deref(), Some("12345678901"));
    assert_eq!(o.city.as_deref(), Some("Eskişehir"));
    assert_eq!(o.district, None, "blank free text is stored as NULL");
    assert!(o.search_text.contains("eskisehir"));

    // Both the local and the international digit forms find the owner, as does the city.
    for query in ["0532", "532123", "90532", "eskisehir", "4915112"] {
        let hits = search(c, query, 10).unwrap();
        assert_eq!(hits.len(), 1, "{query}");
        assert_eq!(hits[0].owner.id, o.id);
    }

    let patch: OwnerPatch = serde_json::from_value(
        json!({"city": "Ankara", "district": "Çankaya", "nationalId": null}),
    )
    .unwrap();
    let updated = owners::update(c, o.id, patch).unwrap();
    assert_eq!(updated.city.as_deref(), Some("Ankara"));
    assert_eq!(updated.district.as_deref(), Some("Çankaya"));
    assert_eq!(updated.national_id, None);
    assert_eq!(updated.phone, o.phone, "unrelated edit keeps the phone");
    assert!(updated.search_text.contains("ankara"));
    assert!(!updated.search_text.contains("eskisehir"));

    // Soft delete and restore rebuild the same key, including the city.
    owners::soft_delete(c, o.id).unwrap();
    let deleted = owners::get(c, o.id).unwrap();
    assert!(deleted.search_text.contains("ankara"));
    assert!(deleted.search_text.contains("0532"));
    let restored = owners::restore(c, o.id).unwrap();
    assert_eq!(restored.search_text, updated.search_text);
}

#[test]
fn patient_alerts_round_trip() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let o = owner(c, "Synthetic Owner");
    let p = patients::create(
        c,
        NewPatient {
            owner_id: o.id,
            name: "Synthetic Tortoise".into(),
            species: "other".into(),
            alerts: Some("  bites  ".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.alerts.as_deref(), Some("bites"));
    assert_eq!(p.species, baytar_core::model::Species::Other);
    let p = patients::update(
        c,
        p.id,
        serde_json::from_value(json!({"alerts": "allergic to penicillin", "species": "reptile"}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(p.alerts.as_deref(), Some("allergic to penicillin"));
    assert_eq!(p.species, baytar_core::model::Species::Reptile);
    let p = patients::update(
        c,
        p.id,
        serde_json::from_value(json!({"alerts": null})).unwrap(),
    )
    .unwrap();
    assert_eq!(p.alerts, None);
    assert!(patients::create(
        c,
        NewPatient {
            owner_id: o.id,
            name: "X".into(),
            species: "dragon".into(),
            ..Default::default()
        }
    )
    .is_err());
}
