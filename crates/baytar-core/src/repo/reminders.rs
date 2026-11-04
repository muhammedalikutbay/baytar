//! Reminders centre: a contact log plus the derived lists of people to remind.
//!
//! Reminder rows are never stored. They are derived on demand from vaccinations and planned
//! vaccine appointments (the merged due list), appointments, receivables and follow-up
//! appointments. The only stored state is `reminder_contacts`, an append-only log that remembers
//! who was contacted about which reminder, so the UI can avoid repeating it the same day.
use super::{accounts, invalid, now, owners, vaccinations, RepoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const KINDS: [&str; 4] = ["vaccine", "appointment", "debt", "followup"];
pub const CHANNELS: [&str; 3] = ["whatsapp", "phone", "other"];
const MAX_DAYS: i64 = 365;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReminderContact {
    pub kind: String,
    pub ref_key: String,
    pub owner_id: i64,
    pub channel: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderContact {
    pub id: i64,
    pub kind: String,
    pub ref_key: String,
    pub owner_id: i64,
    pub channel: String,
    pub contacted_at: String,
    pub note: Option<String>,
}

const COLUMNS: &str = "id, kind, ref_key, owner_id, channel, contacted_at, note";
fn contact_row(r: &Row<'_>) -> rusqlite::Result<ReminderContact> {
    Ok(ReminderContact {
        id: r.get(0)?,
        kind: r.get(1)?,
        ref_key: r.get(2)?,
        owner_id: r.get(3)?,
        channel: r.get(4)?,
        contacted_at: r.get(5)?,
        note: r.get(6)?,
    })
}

/// Appends a contact to the log. The reminder key must belong to the stated kind
/// (`<kind>:...`), the channel must be known and the owner must exist (archived owners are
/// allowed: an outstanding debt outlives an archived contact).
pub fn record_contact(conn: &Connection, input: NewReminderContact) -> Result<ReminderContact> {
    if !KINDS.contains(&input.kind.as_str()) {
        return Err(invalid("kind", "unknown reminder kind"));
    }
    if !CHANNELS.contains(&input.channel.as_str()) {
        return Err(invalid("channel", "unknown contact channel"));
    }
    let key = input.ref_key.trim();
    if !key
        .strip_prefix(input.kind.as_str())
        .is_some_and(|rest| rest.len() > 1 && rest.starts_with(':'))
    {
        return Err(invalid(
            "refKey",
            "must start with the reminder kind and ':'",
        ));
    }
    owners::get(conn, input.owner_id)?;
    let note = super::clean(input.note);
    conn.execute(
        "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel, contacted_at, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![input.kind, key, input.owner_id, input.channel, now(), note],
    )?;
    get(conn, conn.last_insert_rowid())
}

fn get(conn: &Connection, id: i64) -> Result<ReminderContact> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM reminder_contacts WHERE id = ?1"),
        [id],
        contact_row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}

/// Most recent contact of one reminder, if any.
pub fn last_contact(conn: &Connection, ref_key: &str) -> Result<Option<ReminderContact>> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM reminder_contacts WHERE ref_key = ?1 ORDER BY contacted_at DESC, id DESC LIMIT 1"),
            [ref_key],
            contact_row,
        )
        .optional()?)
}

/// Most recent contact for each of the given reminder keys; keys never contacted are absent.
pub fn last_contacts(
    conn: &Connection,
    ref_keys: &[String],
) -> Result<HashMap<String, ReminderContact>> {
    let mut found = HashMap::new();
    for key in ref_keys {
        if found.contains_key(key) {
            continue;
        }
        if let Some(contact) = last_contact(conn, key)? {
            found.insert(key.clone(), contact);
        }
    }
    Ok(found)
}

/// All contacts of one owner, newest first.
pub fn owner_contacts(conn: &Connection, owner_id: i64) -> Result<Vec<ReminderContact>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM reminder_contacts WHERE owner_id = ?1 ORDER BY contacted_at DESC, id DESC"
    ))?;
    let rows = stmt
        .query_map([owner_id], contact_row)?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OverviewQuery {
    /// The clinic's local calendar day, `YYYY-MM-DD`.
    pub today: String,
    /// Vaccines overdue or due within this many days.
    pub vaccine_days: i64,
    /// Receivables whose oldest unpaid charge is at least this many days old.
    pub debt_days: i64,
    /// Planned follow-up visits within this many days.
    pub followup_days: i64,
}
impl Default for OverviewQuery {
    fn default() -> Self {
        Self {
            today: String::new(),
            vaccine_days: 7,
            debt_days: 30,
            followup_days: 7,
        }
    }
}

/// One person to remind. The same shape serves every kind; unused fields are `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderItem {
    pub kind: String,
    pub ref_key: String,
    pub owner_id: i64,
    pub owner_name: String,
    pub phone: Option<String>,
    pub patient_id: Option<i64>,
    pub patient_name: Option<String>,
    pub species: Option<String>,
    /// Vaccine name, appointment title, or follow-up title; empty for debts.
    pub reason: String,
    /// Due date, appointment day, or day of the oldest unpaid charge.
    pub date: String,
    pub time: Option<String>,
    pub amount_minor: Option<i64>,
    pub last_visit_on: Option<String>,
    pub last_contact: Option<ReminderContact>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemindersOverview {
    pub vaccines: Vec<ReminderItem>,
    pub appointments: Vec<ReminderItem>,
    pub debts: Vec<ReminderItem>,
    pub followups: Vec<ReminderItem>,
}

fn after(today: chrono::NaiveDate, days: i64) -> Result<chrono::NaiveDate> {
    today
        .checked_add_signed(chrono::Duration::days(days))
        .ok_or_else(|| invalid("today", "date out of range"))
}
fn days_in_range(field: &str, value: i64) -> Result<()> {
    if !(1..=MAX_DAYS).contains(&value) {
        return Err(invalid(field, "must be between 1 and 365"));
    }
    Ok(())
}
fn fmt(day: chrono::NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

pub fn overview(conn: &Connection, query: OverviewQuery) -> Result<RemindersOverview> {
    let today = super::date("today", &query.today)?;
    days_in_range("vaccineDays", query.vaccine_days)?;
    days_in_range("debtDays", query.debt_days)?;
    days_in_range("followupDays", query.followup_days)?;
    let mut result = RemindersOverview {
        vaccines: vaccine_items(conn, &fmt(after(today, query.vaccine_days)?))?,
        appointments: appointment_items(conn, &query.today, &fmt(after(today, 1)?))?,
        debts: debt_items(conn, &fmt(after(today, -query.debt_days)?))?,
        followups: followup_items(
            conn,
            &query.today,
            &fmt(after(today, 2)?),
            &fmt(after(today, query.followup_days)?),
        )?,
    };
    let keys: Vec<String> = [
        &result.vaccines,
        &result.appointments,
        &result.debts,
        &result.followups,
    ]
    .into_iter()
    .flatten()
    .map(|item| item.ref_key.clone())
    .collect();
    let contacts = last_contacts(conn, &keys)?;
    for item in result
        .vaccines
        .iter_mut()
        .chain(&mut result.appointments)
        .chain(&mut result.debts)
        .chain(&mut result.followups)
    {
        item.last_contact = contacts.get(&item.ref_key).cloned();
    }
    Ok(result)
}

/// Overdue and upcoming doses from the merged due list; deceased patients never appear there.
fn vaccine_items(conn: &Connection, until: &str) -> Result<Vec<ReminderItem>> {
    let due = vaccinations::merged_due(
        conn,
        vaccinations::VaccineDueQuery {
            to: Some(until.into()),
            ..Default::default()
        },
    )?;
    let mut items = Vec::new();
    for row in due {
        let (owner_id, owner_name): (i64, String) = conn.query_row(
            "SELECT o.id, o.full_name FROM patients p JOIN owners o ON o.id = p.owner_id WHERE p.id = ?1",
            [row.patient_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let type_part = row.vaccine_type_id.map_or_else(
            || format!("n-{}", row.vaccine_name.trim().to_lowercase()),
            |id| id.to_string(),
        );
        items.push(ReminderItem {
            kind: "vaccine".into(),
            ref_key: format!("vaccine:{}:{type_part}:{}", row.patient_id, row.due_on),
            owner_id,
            owner_name,
            phone: row.phone,
            patient_id: Some(row.patient_id),
            patient_name: Some(row.patient_name),
            species: Some(row.species.as_str().into()),
            reason: row.vaccine_name,
            date: row.due_on,
            time: None,
            amount_minor: None,
            last_visit_on: None,
            last_contact: None,
        });
    }
    Ok(items)
}

type Mapped = (ReminderItem, Option<String>);
fn appointment_query(
    conn: &Connection,
    sql: &str,
    args: &[&dyn rusqlite::ToSql],
    kind: &str,
) -> Result<Vec<Mapped>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(args, |r| {
            let id: i64 = r.get(0)?;
            Ok((
                ReminderItem {
                    kind: kind.into(),
                    ref_key: format!("{kind}:{id}"),
                    owner_id: r.get(7)?,
                    owner_name: r.get(8)?,
                    phone: r.get(9)?,
                    patient_id: r.get(4)?,
                    patient_name: r.get(5)?,
                    species: r.get(6)?,
                    reason: r.get(1)?,
                    date: r.get(2)?,
                    time: r.get(3)?,
                    amount_minor: None,
                    last_visit_on: None,
                    last_contact: None,
                },
                r.get::<_, Option<String>>(10)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Planned visit/other appointments for today and tomorrow. Vaccine appointments are covered by
/// the vaccine section, clinic-internal reminders have no owner to contact.
fn appointment_items(conn: &Connection, today: &str, tomorrow: &str) -> Result<Vec<ReminderItem>> {
    let rows = appointment_query(
        conn,
        "SELECT a.id, a.title, a.starts_on, a.starts_time, p.id, p.name, p.species, o.id, o.full_name, coalesce(o.phone, o.phone_raw), NULL
         FROM appointments a
         LEFT JOIN patients p ON p.id = a.patient_id
         JOIN owners o ON o.id = coalesce(a.owner_id, p.owner_id)
         WHERE a.deleted_at IS NULL AND a.status = 'planned' AND a.kind IN ('visit', 'other')
           AND a.starts_on BETWEEN ?1 AND ?2
           AND (p.id IS NULL OR (p.deleted_at IS NULL AND p.deceased_on IS NULL))
           AND o.deleted_at IS NULL
         ORDER BY a.starts_on, a.starts_time, a.id",
        params![today, tomorrow],
        "appointment",
    )?;
    Ok(rows.into_iter().map(|(item, _)| item).collect())
}

/// Planned `visit` appointments after tomorrow for patients who already had a visit. Today and
/// tomorrow belong to the appointments section, so a booking is never listed twice.
fn followup_items(
    conn: &Connection,
    today: &str,
    from: &str,
    to: &str,
) -> Result<Vec<ReminderItem>> {
    let rows = appointment_query(
        conn,
        "SELECT a.id, a.title, a.starts_on, a.starts_time, p.id, p.name, p.species, o.id, o.full_name, coalesce(o.phone, o.phone_raw),
           (SELECT max(v.visit_on) FROM visits v WHERE v.patient_id = p.id AND v.deleted_at IS NULL AND v.visit_on <= ?1)
         FROM appointments a
         JOIN patients p ON p.id = a.patient_id
         JOIN owners o ON o.id = p.owner_id
         WHERE a.deleted_at IS NULL AND a.status = 'planned' AND a.kind = 'visit'
           AND a.starts_on BETWEEN ?2 AND ?3
           AND p.deleted_at IS NULL AND p.deceased_on IS NULL AND o.deleted_at IS NULL
         ORDER BY a.starts_on, a.starts_time, a.id",
        params![today, from, to],
        "followup",
    )?;
    Ok(rows
        .into_iter()
        .filter_map(|(mut item, last_visit)| {
            item.last_visit_on = Some(last_visit?);
            Some(item)
        })
        .collect())
}

/// Owners with a positive balance whose oldest unpaid charge is on or before `cutoff`.
fn debt_items(conn: &Connection, cutoff: &str) -> Result<Vec<ReminderItem>> {
    let mut items = Vec::new();
    for debt in accounts::receivables(conn)? {
        if debt.oldest_unpaid_on.as_str() > cutoff {
            continue;
        }
        let phone: Option<String> = conn.query_row(
            "SELECT coalesce(phone, phone_raw) FROM owners WHERE id = ?1",
            [debt.owner_id],
            |r| r.get(0),
        )?;
        items.push(ReminderItem {
            kind: "debt".into(),
            ref_key: format!("debt:{}:{}", debt.owner_id, debt.oldest_unpaid_on),
            owner_id: debt.owner_id,
            owner_name: debt.owner_name,
            phone,
            patient_id: None,
            patient_name: None,
            species: None,
            reason: String::new(),
            date: debt.oldest_unpaid_on,
            time: None,
            amount_minor: Some(debt.balance_minor),
            last_visit_on: None,
            last_contact: None,
        });
    }
    items.sort_by(|a, b| a.date.cmp(&b.date).then(a.owner_id.cmp(&b.owner_id)));
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn contact(owner_id: i64, ref_key: &str, channel: &str) -> NewReminderContact {
        NewReminderContact {
            kind: ref_key.split(':').next().unwrap().into(),
            ref_key: ref_key.into(),
            owner_id,
            channel: channel.into(),
            note: None,
        }
    }

    fn fixture() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute_batch(
                "INSERT INTO owners(id, full_name, phone) VALUES (1, 'Synthetic Owner', '+905551112233'), (2, 'Debtor Owner', NULL), (3, 'Archived Owner', '+905550000000');
                 INSERT INTO patients(id, owner_id, name, species) VALUES (1, 1, 'Pet One', 'cat'), (2, 1, 'Pet Gone', 'dog'), (3, 3, 'Pet Archived', 'cat');
                 UPDATE patients SET deceased_on = '2026-09-01' WHERE id = 2;
                 INSERT INTO vaccine_types(id, name) VALUES (900, 'Synthetic Vaccine');
                 INSERT INTO visits(patient_id, visit_on, reason) VALUES (1, '2026-10-01', 'checkup'), (2, '2026-10-01', 'checkup');",
            )
            .unwrap();
        db
    }

    #[test]
    fn contact_log_records_and_returns_the_latest_per_key() {
        let db = fixture();
        let c = db.conn();
        assert!(last_contact(c, "appointment:5").unwrap().is_none());
        let first = record_contact(c, contact(1, "appointment:5", "whatsapp")).unwrap();
        assert_eq!(first.channel, "whatsapp");
        assert!(first.contacted_at.ends_with('Z'));
        let second = record_contact(
            c,
            NewReminderContact {
                note: Some("  Left a message  ".into()),
                ..contact(1, "appointment:5", "phone")
            },
        )
        .unwrap();
        assert_eq!(second.note.as_deref(), Some("Left a message"));
        record_contact(c, contact(1, "appointment:6", "other")).unwrap();
        assert_eq!(
            last_contact(c, "appointment:5").unwrap().unwrap().id,
            second.id
        );
        let keys = vec![
            "appointment:5".to_string(),
            "appointment:6".into(),
            "appointment:7".into(),
        ];
        let map = last_contacts(c, &keys).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["appointment:5"].channel, "phone");
        assert_eq!(owner_contacts(c, 1).unwrap().len(), 3);
        assert!(owner_contacts(c, 2).unwrap().is_empty());
    }

    #[test]
    fn contact_log_validates_input() {
        let db = fixture();
        let c = db.conn();
        for bad in [
            NewReminderContact {
                kind: "sms".into(),
                ..contact(1, "sms:1", "phone")
            },
            contact(1, "appointment:1", "carrier-pigeon"),
            contact(1, "vaccine:1:2:2026-11-04", "phone").tap_kind("debt"),
            contact(1, "appointment:", "phone"),
            contact(1, "appointmentx:1", "phone").tap_kind("appointment"),
            contact(1, "  ", "phone").tap_kind("debt"),
        ] {
            assert!(
                matches!(record_contact(c, bad), Err(RepoError::Invalid { .. })),
                "should be invalid"
            );
        }
        assert!(matches!(
            record_contact(c, contact(99, "appointment:1", "phone")),
            Err(RepoError::NotFound)
        ));
        // Archived owners can still be recorded: debts outlive archived contacts.
        c.execute(
            "UPDATE owners SET deleted_at = '2026-10-01T00:00:00Z' WHERE id = 3",
            [],
        )
        .unwrap();
        record_contact(c, contact(3, "debt:3:2026-01-01", "phone")).unwrap();
        // The table itself rejects what the repo would reject.
        assert!(c
            .execute(
                "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('bogus', 'x:1', 1, 'phone')",
                []
            )
            .is_err());
        assert!(c
            .execute(
                "INSERT INTO reminder_contacts (kind, ref_key, owner_id, channel) VALUES ('debt', 'debt:99', 99, 'phone')",
                []
            )
            .is_err());
    }

    trait TapKind {
        fn tap_kind(self, kind: &str) -> Self;
    }
    impl TapKind for NewReminderContact {
        fn tap_kind(mut self, kind: &str) -> Self {
            self.kind = kind.into();
            self
        }
    }

    fn query(today: &str) -> OverviewQuery {
        OverviewQuery {
            today: today.into(),
            ..Default::default()
        }
    }

    #[test]
    fn overview_derives_every_section_and_skips_deceased_patients() {
        let db = fixture();
        let c = db.conn();
        c.execute_batch(
            "INSERT INTO vaccinations(id, patient_id, vaccine_type_id, vaccine_name, given_on, next_due_on) VALUES
               (1, 1, 900, 'Synthetic Vaccine', '2025-11-01', '2026-11-01'),
               (2, 2, 900, 'Synthetic Vaccine', '2025-11-01', '2026-11-02'),
               (3, 1, NULL, 'Far Away Vaccine', '2026-01-01', '2027-06-01'),
               (4, 3, 900, 'Synthetic Vaccine', '2025-10-20', '2026-10-20');
             INSERT INTO appointments(id, patient_id, owner_id, title, starts_on, starts_time, kind, status) VALUES
               (10, 1, 1, 'Tomorrow exam', '2026-11-05', '09:30', 'visit', 'planned'),
               (11, 1, 1, 'Today exam', '2026-11-04', NULL, 'visit', 'planned'),
               (12, 1, 1, 'Done today', '2026-11-04', NULL, 'visit', 'done'),
               (13, 1, 1, 'Clinic reminder', '2026-11-05', NULL, 'reminder', 'planned'),
               (14, 2, 1, 'Deceased exam', '2026-11-05', NULL, 'visit', 'planned'),
               (15, 1, 1, 'Control in six days', '2026-11-10', '11:00', 'visit', 'planned'),
               (16, 1, 1, 'Control in two months', '2027-01-10', NULL, 'visit', 'planned'),
               (17, 3, 3, 'New patient exam', '2026-11-08', NULL, 'visit', 'planned'),
               (18, 1, 1, 'Cancelled control', '2026-11-09', NULL, 'visit', 'cancelled');
             INSERT INTO appointments(id, patient_id, owner_id, title, starts_on, kind, status, vaccine_type_id) VALUES
               (19, 1, 1, 'Vaccine tomorrow', '2026-11-05', 'vaccine', 'planned', 900);
             INSERT INTO charges(owner_id, description, quantity, unit_price_minor, total_minor, charged_on) VALUES
               (2, 'Old charge', 1, 25000, 25000, '2026-09-01'),
               (1, 'Recent charge', 1, 9000, 9000, '2026-10-30');",
        )
        .unwrap();
        record_contact(c, contact(1, "appointment:10", "whatsapp")).unwrap();

        let o = overview(c, query("2026-11-04")).unwrap();

        // An overdue dose and the planned vaccine appointment (which supersedes the older dose) are listed;
        // the deceased patient and the dose due next year are not.
        let keys: Vec<&str> = o.vaccines.iter().map(|i| i.ref_key.as_str()).collect();
        assert_eq!(
            keys,
            vec!["vaccine:3:900:2026-10-20", "vaccine:1:900:2026-11-05"]
        );
        assert_eq!(o.vaccines[0].owner_id, 3);
        assert_eq!(o.vaccines[1].owner_id, 1);
        assert_eq!(o.vaccines[1].patient_name.as_deref(), Some("Pet One"));
        assert_eq!(o.vaccines[1].phone.as_deref(), Some("+905551112233"));
        assert_eq!(o.vaccines[1].reason, "Synthetic Vaccine");

        // Planned visit appointments of today and tomorrow, not done/cancelled/vaccine/internal
        // or deceased ones.
        let keys: Vec<&str> = o.appointments.iter().map(|i| i.ref_key.as_str()).collect();
        assert_eq!(keys, vec!["appointment:11", "appointment:10"]);
        assert_eq!(o.appointments[1].time.as_deref(), Some("09:30"));
        assert_eq!(
            o.appointments[1].last_contact.as_ref().unwrap().channel,
            "whatsapp"
        );
        assert!(o.appointments[0].last_contact.is_none());

        // Only the debt older than 30 days.
        assert_eq!(o.debts.len(), 1);
        assert_eq!(o.debts[0].ref_key, "debt:2:2026-09-01");
        assert_eq!(o.debts[0].amount_minor, Some(25000));
        assert_eq!(o.debts[0].phone, None);

        // Follow-ups: after tomorrow, patient already had a visit, planned only.
        let keys: Vec<&str> = o.followups.iter().map(|i| i.ref_key.as_str()).collect();
        assert_eq!(keys, vec!["followup:15"]);
        assert_eq!(o.followups[0].last_visit_on.as_deref(), Some("2026-10-01"));

        // Wider windows pull in more rows.
        let wide = overview(
            c,
            OverviewQuery {
                vaccine_days: 365,
                debt_days: 1,
                followup_days: 365,
                ..query("2026-11-04")
            },
        )
        .unwrap();
        assert_eq!(wide.vaccines.len(), 3);
        assert_eq!(wide.debts.len(), 2);
        assert_eq!(wide.followups.len(), 2);
    }

    #[test]
    fn overview_validates_its_query() {
        let db = fixture();
        let c = db.conn();
        assert!(overview(c, query("")).is_err());
        assert!(overview(c, query("2026-02-30")).is_err());
        for bad in [0, -1, 366] {
            for q in [
                OverviewQuery {
                    vaccine_days: bad,
                    ..query("2026-11-04")
                },
                OverviewQuery {
                    debt_days: bad,
                    ..query("2026-11-04")
                },
                OverviewQuery {
                    followup_days: bad,
                    ..query("2026-11-04")
                },
            ] {
                assert!(overview(c, q).is_err());
            }
        }
        let empty = overview(c, query("2026-11-04")).unwrap();
        assert!(empty.vaccines.is_empty() && empty.debts.is_empty());
    }
}
