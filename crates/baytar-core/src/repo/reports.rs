//! Read-only practice reports: monthly activity and money, species mix, top diagnoses and
//! medicines, vaccine coverage. Soft-deleted rows never count. Nothing here writes.
use super::{date, range, Result};
use crate::text::fold;
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A report range covers at most this many calendar months.
const MAX_MONTHS: usize = 120;
const TOP_LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyRow {
    /// `YYYY-MM`.
    pub month: String,
    pub visits: i64,
    pub new_patients: i64,
    pub new_owners: i64,
    pub vaccinations: i64,
    pub revenue_minor: i64,
    pub collections_minor: i64,
    /// Sum of positive owner balances at the end of the month (or of the range, when it ends sooner).
    pub outstanding_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesCount {
    pub species: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccineCoverage {
    /// Active (alive, not deleted) dogs and cats.
    pub patients: i64,
    pub rabies_covered: i64,
    pub core_covered: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportOverview {
    pub from: String,
    pub to: String,
    pub months: Vec<MonthlyRow>,
    pub species: Vec<SpeciesCount>,
    pub diagnoses: Vec<NamedCount>,
    pub medicines: Vec<NamedCount>,
    pub coverage: VaccineCoverage,
}

pub fn overview(conn: &Connection, from: &str, to: &str, today: &str) -> Result<ReportOverview> {
    Ok(ReportOverview {
        from: from.into(),
        to: to.into(),
        months: monthly(conn, from, to)?,
        species: species_mix(conn)?,
        diagnoses: top_diagnoses(conn, from, to)?,
        medicines: top_medicines(conn, from, to)?,
        coverage: vaccine_coverage(conn, today)?,
    })
}

fn month_start(d: NaiveDate) -> NaiveDate {
    d.with_day(1).unwrap_or(d)
}

fn next_month(d: NaiveDate) -> NaiveDate {
    let (year, month) = if d.month() == 12 {
        (d.year() + 1, 1)
    } else {
        (d.year(), d.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(d)
}

fn count(conn: &Connection, sql: &str, from: &str, to: &str) -> Result<i64> {
    Ok(conn.query_row(sql, params![from, to], |r| r.get(0))?)
}

pub fn monthly(conn: &Connection, from: &str, to: &str) -> Result<Vec<MonthlyRow>> {
    range(from, to)?;
    let (start, end) = (date("from", from)?, date("to", to)?);
    let mut rows = Vec::new();
    let mut cursor = month_start(start);
    while cursor <= end {
        if rows.len() >= MAX_MONTHS {
            return Err(super::invalid("to", "range is too long"));
        }
        let following = next_month(cursor);
        let first = cursor.max(start).format("%Y-%m-%d").to_string();
        let last = (following.pred_opt().unwrap_or(following))
            .min(end)
            .format("%Y-%m-%d")
            .to_string();
        rows.push(MonthlyRow {
            month: cursor.format("%Y-%m").to_string(),
            visits: count(
                conn,
                "SELECT COUNT(*) FROM visits v JOIN patients p ON p.id = v.patient_id \
                 WHERE v.deleted_at IS NULL AND p.deleted_at IS NULL AND v.visit_on BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            new_patients: count(
                conn,
                "SELECT COUNT(*) FROM patients WHERE deleted_at IS NULL \
                 AND substr(created_at, 1, 10) BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            new_owners: count(
                conn,
                "SELECT COUNT(*) FROM owners WHERE deleted_at IS NULL \
                 AND substr(created_at, 1, 10) BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            vaccinations: count(
                conn,
                "SELECT COUNT(*) FROM vaccinations x JOIN patients p ON p.id = x.patient_id \
                 WHERE x.deleted_at IS NULL AND p.deleted_at IS NULL AND x.given_on BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            revenue_minor: count(
                conn,
                "SELECT COALESCE(SUM(total_minor), 0) FROM charges \
                 WHERE deleted_at IS NULL AND charged_on BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            collections_minor: count(
                conn,
                "SELECT COALESCE(SUM(amount_minor), 0) FROM payments \
                 WHERE deleted_at IS NULL AND paid_on BETWEEN ?1 AND ?2",
                &first,
                &last,
            )?,
            outstanding_minor: outstanding_at(conn, &last)?,
        });
        cursor = following;
    }
    Ok(rows)
}

/// Sum of positive per-owner balances (charges minus payments) up to and including `day`.
/// Credit balances do not offset other owners' debts, matching the receivables list.
fn outstanding_at(conn: &Connection, day: &str) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(balance), 0) FROM ( \
             SELECT owner_id, SUM(amount) AS balance FROM ( \
                 SELECT owner_id, total_minor AS amount FROM charges \
                     WHERE deleted_at IS NULL AND charged_on <= ?1 \
                 UNION ALL \
                 SELECT owner_id, -amount_minor FROM payments \
                     WHERE deleted_at IS NULL AND paid_on <= ?1 \
             ) GROUP BY owner_id HAVING SUM(amount) > 0)",
        [day],
        |r| r.get(0),
    )?)
}

/// Active patients (not deleted, not deceased) per species, largest group first.
pub fn species_mix(conn: &Connection) -> Result<Vec<SpeciesCount>> {
    let mut stmt = conn.prepare(
        "SELECT species, COUNT(*) AS n FROM patients \
         WHERE deleted_at IS NULL AND deceased_on IS NULL \
         GROUP BY species ORDER BY n DESC, species",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SpeciesCount {
                species: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}

/// Groups texts by their folded form; the first spelling seen is the display name.
fn group_top(texts: impl Iterator<Item = String>) -> Vec<NamedCount> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, NamedCount> = HashMap::new();
    for text in texts {
        let key = fold(&text);
        if key.is_empty() {
            continue;
        }
        let entry = groups.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            NamedCount {
                name: text.split_whitespace().collect::<Vec<_>>().join(" "),
                count: 0,
            }
        });
        entry.count += 1;
    }
    let mut result: Vec<NamedCount> = order
        .into_iter()
        .filter_map(|key| groups.remove(&key))
        .collect();
    result.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    result.truncate(TOP_LIMIT);
    result
}

fn text_rows(conn: &Connection, sql: &str, from: &str, to: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![from, to], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn top_diagnoses(conn: &Connection, from: &str, to: &str) -> Result<Vec<NamedCount>> {
    range(from, to)?;
    let rows = text_rows(
        conn,
        "SELECT v.diagnosis FROM visits v JOIN patients p ON p.id = v.patient_id \
         WHERE v.deleted_at IS NULL AND p.deleted_at IS NULL AND v.diagnosis IS NOT NULL \
         AND v.visit_on BETWEEN ?1 AND ?2 ORDER BY v.visit_on, v.id",
        from,
        to,
    )?;
    Ok(group_top(rows.into_iter()))
}

pub fn top_medicines(conn: &Connection, from: &str, to: &str) -> Result<Vec<NamedCount>> {
    range(from, to)?;
    let rows = text_rows(
        conn,
        "SELECT i.medication_name FROM prescription_items i \
         JOIN visits v ON v.id = i.visit_id JOIN patients p ON p.id = v.patient_id \
         WHERE i.deleted_at IS NULL AND v.deleted_at IS NULL AND p.deleted_at IS NULL \
         AND v.visit_on BETWEEN ?1 AND ?2 ORDER BY v.visit_on, i.id",
        from,
        to,
    )?;
    Ok(group_top(rows.into_iter()))
}

fn is_rabies(name: &str) -> bool {
    name.contains("kuduz") || name.contains("rabies")
}

fn is_core(name: &str) -> bool {
    ["karma", "dhpp", "dhlpp", "fvrcp", "parvo", "distemper"]
        .iter()
        .any(|k| name.contains(k))
}

/// Share of active dogs and cats whose latest rabies / core vaccination is not overdue.
/// A class counts as covered when the patient has at least one vaccination of that class and
/// the most recent one has no `next_due_on` or a due date that is not before `today`.
pub fn vaccine_coverage(conn: &Connection, today: &str) -> Result<VaccineCoverage> {
    date("today", today)?;
    let patients: i64 = conn.query_row(
        "SELECT COUNT(*) FROM patients WHERE deleted_at IS NULL AND deceased_on IS NULL \
         AND species IN ('dog', 'cat')",
        [],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT x.patient_id, x.vaccine_name, t.name, x.given_on, x.next_due_on \
         FROM vaccinations x JOIN patients p ON p.id = x.patient_id \
         LEFT JOIN vaccine_types t ON t.id = x.vaccine_type_id \
         WHERE x.deleted_at IS NULL AND p.deleted_at IS NULL AND p.deceased_on IS NULL \
         AND p.species IN ('dog', 'cat') ORDER BY x.given_on, x.id",
    )?;
    // Rows come oldest first, so a later row of the same class replaces the earlier one.
    let mut rabies: HashMap<i64, bool> = HashMap::new();
    let mut core: HashMap<i64, bool> = HashMap::new();
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(4)?,
        ))
    })?;
    for row in rows {
        let (patient, name, type_name, next_due) = row?;
        let names = fold(&format!("{} {}", name, type_name.unwrap_or_default()));
        let current = next_due.is_none_or(|d| d.as_str() >= today);
        if is_rabies(&names) {
            rabies.insert(patient, current);
        }
        if is_core(&names) {
            core.insert(patient, current);
        }
    }
    Ok(VaccineCoverage {
        patients,
        rabies_covered: rabies.values().filter(|c| **c).count() as i64,
        core_covered: core.values().filter(|c| **c).count() as i64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute_batch(
                "INSERT INTO owners(id, full_name, created_at) VALUES
                    (1, 'Owner One', '2026-01-10T09:00:00Z'),
                    (2, 'Owner Two', '2026-02-05T09:00:00Z'),
                    (3, 'Owner Gone', '2026-02-06T09:00:00Z');
                 UPDATE owners SET deleted_at = '2026-03-01T00:00:00Z' WHERE id = 3;
                 INSERT INTO patients(id, owner_id, name, species, created_at) VALUES
                    (1, 1, 'Dog A', 'dog', '2026-01-10T09:00:00Z'),
                    (2, 1, 'Cat B', 'cat', '2026-02-11T09:00:00Z'),
                    (3, 2, 'Bird C', 'bird', '2026-02-12T09:00:00Z'),
                    (4, 2, 'Dog D', 'dog', '2026-02-20T09:00:00Z'),
                    (5, 2, 'Dog Gone', 'dog', '2026-02-21T09:00:00Z');
                 UPDATE patients SET deleted_at = '2026-03-01T00:00:00Z' WHERE id = 5;
                 INSERT INTO patients(id, owner_id, name, species, deceased_on, created_at)
                    VALUES (6, 2, 'Cat Late', 'cat', '2026-01-01', '2026-01-02T09:00:00Z');",
            )
            .unwrap();
        db
    }

    #[test]
    fn monthly_counts_money_and_outstanding() {
        let db = db();
        let c = db.conn();
        c.execute_batch(
            "INSERT INTO visits(id, patient_id, visit_on, diagnosis) VALUES
                (1, 1, '2026-01-15', 'x'), (2, 2, '2026-02-03', 'x'),
                (3, 1, '2026-02-28', 'x'), (4, 5, '2026-02-10', 'x'), (5, 1, '2026-03-01', 'x');
             UPDATE visits SET deleted_at = '2026-03-02T00:00:00Z' WHERE id = 3;
             INSERT INTO vaccinations(patient_id, vaccine_name, given_on)
                VALUES (1, 'Kuduz', '2026-02-04'), (5, 'Kuduz', '2026-02-04');
             INSERT INTO charges(owner_id, description, quantity, unit_price_minor, total_minor, charged_on)
                VALUES (1, 'a', 1, 10000, 10000, '2026-01-15'), (1, 'b', 1, 5000, 5000, '2026-02-03'),
                       (2, 'c', 1, 2000, 2000, '2026-02-10'), (2, 'gone', 1, 900, 900, '2026-02-11');
             UPDATE charges SET deleted_at = '2026-03-02T00:00:00Z' WHERE description = 'gone';
             INSERT INTO payments(owner_id, paid_on, amount_minor) VALUES
                (1, '2026-01-20', 4000), (1, '2026-02-15', 20000), (2, '2026-02-12', 500);",
        )
        .unwrap();
        let rows = monthly(c, "2026-01-01", "2026-02-28").unwrap();
        assert_eq!(rows.len(), 2);
        let jan = &rows[0];
        assert_eq!(jan.month, "2026-01");
        assert_eq!(
            (
                jan.visits,
                jan.new_patients,
                jan.new_owners,
                jan.vaccinations
            ),
            (1, 2, 1, 0)
        );
        assert_eq!((jan.revenue_minor, jan.collections_minor), (10000, 4000));
        assert_eq!(jan.outstanding_minor, 6000);
        let feb = &rows[1];
        // Visit 3 is deleted and visit 4 belongs to a deleted patient.
        assert_eq!(
            (
                feb.visits,
                feb.new_patients,
                feb.new_owners,
                feb.vaccinations
            ),
            (1, 3, 1, 1)
        );
        assert_eq!((feb.revenue_minor, feb.collections_minor), (7000, 20500));
        // Owner 1 paid 24000 against 15000 (credit, ignored); owner 2 owes 2000 - 500.
        assert_eq!(feb.outstanding_minor, 1500);
    }

    #[test]
    fn partial_month_range_clips_both_ends() {
        let db = db();
        db.conn()
            .execute_batch(
                "INSERT INTO visits(patient_id, visit_on) VALUES (1, '2026-02-02'), (1, '2026-02-10'), (1, '2026-03-04'), (1, '2026-03-20');",
            )
            .unwrap();
        let rows = monthly(db.conn(), "2026-02-05", "2026-03-10").unwrap();
        assert_eq!(rows.iter().map(|r| r.visits).collect::<Vec<_>>(), [1, 1]);
        assert!(monthly(db.conn(), "2026-03-10", "2026-02-05").is_err());
        assert!(monthly(db.conn(), "2000-01-01", "2026-02-05").is_err());
        assert!(monthly(db.conn(), "bad", "2026-02-05").is_err());
    }

    #[test]
    fn species_mix_counts_only_active_living_patients() {
        let db = db();
        let mix = species_mix(db.conn()).unwrap();
        assert_eq!(
            mix,
            vec![
                SpeciesCount {
                    species: "dog".into(),
                    count: 2
                },
                SpeciesCount {
                    species: "bird".into(),
                    count: 1
                },
                SpeciesCount {
                    species: "cat".into(),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn diagnoses_group_by_folded_text_and_cap_at_ten() {
        let db = db();
        let c = db.conn();
        c.execute_batch(
            "INSERT INTO visits(patient_id, visit_on, diagnosis) VALUES
                (1, '2026-02-01', 'Otitis  Externa'), (1, '2026-02-02', 'otitis externa'),
                (2, '2026-02-03', 'OTİTİS EXTERNA'), (2, '2026-02-04', 'Gastroenterit'), (2, '2026-02-04', 'GASTROENTERİT'),
                (2, '2026-02-05', '   '), (2, '2026-04-01', 'Out of range'), (5, '2026-02-06', 'Deleted patient');",
        )
        .unwrap();
        for i in 0..12 {
            c.execute(
                "INSERT INTO visits(patient_id, visit_on, diagnosis) VALUES (1, '2026-02-07', ?1)",
                [format!("Dx {i:02}")],
            )
            .unwrap();
        }
        let top = top_diagnoses(c, "2026-02-01", "2026-02-28").unwrap();
        assert_eq!(top.len(), 10);
        assert_eq!(
            top[0],
            NamedCount {
                name: "Otitis Externa".into(),
                count: 3
            }
        );
        assert_eq!(top[1].name, "Gastroenterit");
        assert!(top.iter().all(|d| d.name != "Deleted patient"));
    }

    #[test]
    fn medicines_count_prescriptions_in_range() {
        let db = db();
        let c = db.conn();
        c.execute_batch(
            "INSERT INTO visits(id, patient_id, visit_on) VALUES (1, 1, '2026-02-01'), (2, 2, '2026-02-02'), (3, 1, '2026-05-01');
             INSERT INTO prescription_items(visit_id, medication_name) VALUES
                (1, 'Amoksisilin'), (2, 'AMOKSİSİLİN'), (2, 'Meloksikam'), (3, 'Amoksisilin'), (1, 'Gone');
             UPDATE prescription_items SET deleted_at = '2026-06-01T00:00:00Z' WHERE medication_name = 'Gone';",
        )
        .unwrap();
        let top = top_medicines(c, "2026-02-01", "2026-02-28").unwrap();
        assert_eq!(
            top,
            vec![
                NamedCount {
                    name: "Amoksisilin".into(),
                    count: 2
                },
                NamedCount {
                    name: "Meloksikam".into(),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn vaccine_coverage_uses_latest_vaccination_per_class() {
        let db = db();
        let c = db.conn();
        // Active dogs/cats: patients 1 (dog), 2 (cat), 4 (dog). Patient 6 is deceased.
        c.execute_batch(
            "INSERT INTO vaccinations(patient_id, vaccine_name, given_on, next_due_on) VALUES
                (1, 'Kuduz aşısı', '2025-05-01', '2026-05-01'),
                (1, 'Karma aşı', '2025-06-01', '2026-06-01'),
                (2, 'Kuduz aşısı', '2024-01-01', '2025-01-01'),
                (2, 'Kuduz aşısı', '2026-01-01', '2027-01-01'),
                (2, 'Kedi karma aşısı', '2024-01-01', '2025-01-01'),
                (4, 'Karma aşı', '2026-01-01', NULL),
                (6, 'Kuduz aşısı', '2026-01-01', '2027-01-01'),
                (3, 'Kuduz aşısı', '2026-01-01', '2027-01-01'),
                (4, 'Kuduz aşısı', '2025-01-01', '2026-01-01');
             UPDATE vaccinations SET deleted_at = '2026-06-01T00:00:00Z' WHERE patient_id = 4 AND vaccine_name = 'Kuduz aşısı';",
        )
        .unwrap();
        let coverage = vaccine_coverage(c, "2026-03-01").unwrap();
        assert_eq!(coverage.patients, 3);
        // Rabies: patient 1 current, patient 2 current (latest). Patient 4 has only a deleted one.
        assert_eq!(coverage.rabies_covered, 2);
        // Core: patient 1 current, patient 2 overdue, patient 4 never due.
        assert_eq!(coverage.core_covered, 2);
        assert!(vaccine_coverage(c, "nope").is_err());
        let late = vaccine_coverage(c, "2026-07-01").unwrap();
        assert_eq!((late.rabies_covered, late.core_covered), (1, 1));
    }

    #[test]
    fn empty_database_returns_zero_rows() {
        let db = Database::open_in_memory().unwrap();
        let report = overview(db.conn(), "2026-01-01", "2026-03-31", "2026-03-31").unwrap();
        assert_eq!(report.months.len(), 3);
        assert!(report
            .months
            .iter()
            .all(|m| m.visits == 0 && m.revenue_minor == 0 && m.outstanding_minor == 0));
        assert!(report.species.is_empty());
        assert!(report.diagnoses.is_empty() && report.medicines.is_empty());
        assert_eq!(report.coverage.patients, 0);
    }
}
