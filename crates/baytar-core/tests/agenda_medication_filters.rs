use baytar_core::{
    db::Database,
    repo::{
        appointments::{self, NewAppointment},
        medications::{self, NewMedication},
    },
};

#[test]
fn calendar_series_anchor_month_end_and_leap_day() {
    let db = Database::open_in_memory().unwrap();
    for (start, months, expected) in [
        ("2028-01-31", 1, ["2028-01-31", "2028-02-29", "2028-03-31"]),
        ("2028-02-29", 12, ["2028-02-29", "2029-02-28", "2030-02-28"]),
    ] {
        let rows = appointments::create_series_with_months(
            db.conn(),
            NewAppointment {
                title: "Synthetic clinic task".into(),
                starts_on: start.into(),
                ..Default::default()
            },
            3,
            1,
            Some(months),
        )
        .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| row.starts_on.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(rows[0].series_id.is_some());
        assert!(rows.iter().all(|row| row.series_id == rows[0].series_id));
    }
}

#[test]
fn calendar_series_overflow_rolls_back_every_occurrence() {
    let db = Database::open_in_memory().unwrap();
    let base = NewAppointment {
        title: "Synthetic clinic task".into(),
        starts_on: "9999-12-31".into(),
        ..Default::default()
    };
    assert!(
        appointments::create_series_with_months(db.conn(), base.clone(), 2, 1, Some(1)).is_err()
    );
    assert!(appointments::between(db.conn(), "9999-01-01", "9999-12-31")
        .unwrap()
        .is_empty());
    assert!(appointments::create_series_with_months(db.conn(), base, 2, 1, Some(0)).is_err());
}

#[test]
fn inactive_medications_are_opt_in_and_restorable() {
    let db = Database::open_in_memory().unwrap();
    let c = db.conn();
    let row = medications::create(
        c,
        NewMedication {
            name: "Synthetic medicine".into(),
            ..Default::default()
        },
    )
    .unwrap();
    medications::soft_delete(c, row.id).unwrap();
    assert!(!medications::list(c).unwrap().iter().any(|m| m.id == row.id));
    assert!(medications::list_with_deleted(c, true)
        .unwrap()
        .iter()
        .any(|m| m.id == row.id && m.deleted_at.is_some()));
    medications::restore(c, row.id).unwrap();
    assert!(medications::list(c).unwrap().iter().any(|m| m.id == row.id));
}
