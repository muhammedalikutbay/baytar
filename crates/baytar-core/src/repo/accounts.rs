//! Derived ledgers. Deleting an owner never erases their financial obligations.
use super::{RepoError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const MAX_MONEY: i64 = 9_007_199_254_740_991;
pub(crate) fn money(field: &str, value: Option<i64>) -> Result<()> {
    if value.is_some_and(|v| !(0..=MAX_MONEY).contains(&v)) {
        return Err(super::invalid(field, "must be a non-negative safe integer"));
    }
    Ok(())
}
pub(crate) fn vat(value: Option<i64>) -> Result<()> {
    if value.is_some_and(|v| !(0..=100).contains(&v)) {
        return Err(super::invalid("vatRate", "must be between 0 and 100"));
    }
    Ok(())
}
pub(crate) fn active_owner(conn: &Connection, id: i64) -> Result<()> {
    if super::owners::get(conn, id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    Ok(())
}
fn add(left: i64, right: i64) -> Result<i64> {
    left.checked_add(right)
        .filter(|v| (-MAX_MONEY..=MAX_MONEY).contains(v))
        .ok_or_else(|| super::invalid("balance", "account total is too large"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementEntry {
    pub id: i64,
    pub kind: String,
    pub owner_id: i64,
    pub owner_name: String,
    pub date: String,
    pub description: String,
    pub amount_minor: i64,
    pub method: Option<String>,
    pub notes: Option<String>,
    pub deleted_at: Option<String>,
    pub running_balance_minor: i64,
}
const LEDGER: &str = "SELECT c.id, 'charge' AS kind, c.owner_id, o.full_name, c.charged_on AS day, c.description, c.total_minor AS amount, NULL AS method, c.notes, c.deleted_at, c.created_at FROM charges c JOIN owners o ON o.id=c.owner_id UNION ALL SELECT p.id, 'payment', p.owner_id, o.full_name, p.paid_on, '', p.amount_minor, p.method, p.notes, p.deleted_at, p.created_at FROM payments p JOIN owners o ON o.id=p.owner_id";
fn entries(
    conn: &Connection,
    owner_id: Option<i64>,
    from: &str,
    to: &str,
    include_deleted: bool,
) -> Result<Vec<StatementEntry>> {
    let mut stmt = conn.prepare(&format!("SELECT * FROM ({LEDGER}) WHERE (?1 IS NULL OR owner_id=?1) AND day BETWEEN ?2 AND ?3 AND (?4 OR deleted_at IS NULL) ORDER BY day, created_at, kind, id"))?;
    let rows = stmt
        .query_map(params![owner_id, from, to, include_deleted], |r| {
            Ok(StatementEntry {
                id: r.get(0)?,
                kind: r.get(1)?,
                owner_id: r.get(2)?,
                owner_name: r.get(3)?,
                date: r.get(4)?,
                description: r.get(5)?,
                amount_minor: r.get(6)?,
                method: r.get(7)?,
                notes: r.get(8)?,
                deleted_at: r.get(9)?,
                running_balance_minor: 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn statement(
    conn: &Connection,
    owner_id: i64,
    include_deleted: bool,
) -> Result<Vec<StatementEntry>> {
    super::owners::get(conn, owner_id)?;
    let mut rows = entries(
        conn,
        Some(owner_id),
        "0001-01-01",
        "9999-12-31",
        include_deleted,
    )?;
    let mut balance = 0;
    for row in &mut rows {
        if row.deleted_at.is_none() {
            balance = add(
                balance,
                if row.kind == "charge" {
                    row.amount_minor
                } else {
                    -row.amount_minor
                },
            )?;
        }
        row.running_balance_minor = balance;
    }
    Ok(rows)
}
pub fn balance(conn: &Connection, owner_id: i64) -> Result<i64> {
    Ok(statement(conn, owner_id, false)?
        .last()
        .map_or(0, |r| r.running_balance_minor))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receivable {
    pub owner_id: i64,
    pub owner_name: String,
    pub phone: Option<String>,
    pub balance_minor: i64,
    pub oldest_unpaid_on: String,
    pub last_payment_on: Option<String>,
}
pub fn receivables(conn: &Connection) -> Result<Vec<Receivable>> {
    // Include archived owners: hiding a contact must not hide outstanding money.
    let mut stmt = conn.prepare("SELECT id, full_name, phone FROM owners WHERE EXISTS(SELECT 1 FROM charges WHERE owner_id=owners.id AND deleted_at IS NULL)")?;
    let owners = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = Vec::new();
    for (id, name, phone) in owners {
        let rows = statement(conn, id, false)?;
        let balance = rows.last().map_or(0, |r| r.running_balance_minor);
        if balance <= 0 {
            continue;
        }
        let mut paid = 0;
        let mut last_payment_on = None;
        for row in rows.iter().filter(|r| r.kind == "payment") {
            paid = add(paid, row.amount_minor)?;
            last_payment_on = Some(row.date.clone());
        }
        for row in rows.iter().filter(|r| r.kind == "charge") {
            if paid >= row.amount_minor {
                paid -= row.amount_minor;
            } else {
                result.push(Receivable {
                    owner_id: id,
                    owner_name: name,
                    phone,
                    balance_minor: balance,
                    oldest_unpaid_on: row.date.clone(),
                    last_payment_on,
                });
                break;
            }
        }
    }
    result.sort_by(|a, b| {
        b.balance_minor
            .cmp(&a.balance_minor)
            .then(a.oldest_unpaid_on.cmp(&b.oldest_unpaid_on))
            .then(a.owner_id.cmp(&b.owner_id))
    });
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySummary {
    pub charges_minor: i64,
    pub collections_minor: i64,
    pub collections_by_method: BTreeMap<String, i64>,
    pub entries: Vec<StatementEntry>,
}
pub fn daily_summary(conn: &Connection, from: &str, to: &str) -> Result<DailySummary> {
    super::range(from, to)?;
    let rows = entries(conn, None, from, to, false)?;
    let mut summary = DailySummary {
        charges_minor: 0,
        collections_minor: 0,
        collections_by_method: BTreeMap::from([
            ("cash".into(), 0),
            ("card".into(), 0),
            ("transfer".into(), 0),
            ("other".into(), 0),
        ]),
        entries: Vec::new(),
    };
    for row in &rows {
        if row.kind == "charge" {
            summary.charges_minor = add(summary.charges_minor, row.amount_minor)?;
        } else {
            summary.collections_minor = add(summary.collections_minor, row.amount_minor)?;
            let amount = summary
                .collections_by_method
                .entry(row.method.clone().unwrap_or_default())
                .or_default();
            *amount = add(*amount, row.amount_minor)?;
        }
    }
    summary.entries = rows;
    Ok(summary)
}

/// Clinical catalog choices, optionally linked to a priced stock item. No stock mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargeSource {
    pub key: String,
    pub name: String,
    pub inventory_item_id: Option<i64>,
    pub price_minor: Option<i64>,
}
pub fn charge_sources(conn: &Connection) -> Result<Vec<ChargeSource>> {
    let mut stmt=conn.prepare("SELECT 'medication-' || m.id || '-' || coalesce(i.id,0), m.name, i.id, i.sale_price_minor FROM medications m LEFT JOIN inventory_items i ON i.medication_id=m.id AND i.deleted_at IS NULL AND i.is_active=1 WHERE m.deleted_at IS NULL UNION ALL SELECT 'vaccine-' || v.id || '-' || coalesce(i.id,0), v.name, i.id, i.sale_price_minor FROM vaccine_types v LEFT JOIN inventory_items i ON i.vaccine_type_id=v.id AND i.deleted_at IS NULL AND i.is_active=1 WHERE v.deleted_at IS NULL AND v.is_active=1 ORDER BY 2,1")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ChargeSource {
                key: r.get(0)?,
                name: r.get(1)?,
                inventory_item_id: r.get(2)?,
                price_minor: r.get(3)?,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::Database,
        model::{PaymentMethod, ServiceCategory},
        repo::{
            charges::{self, NewCharge},
            payments::{self, NewPayment},
            services::{self, NewService},
        },
    };
    fn fixture() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute(
                "INSERT INTO owners(id,full_name) VALUES(1,'Synthetic Owner'),(2,'Other Owner')",
                [],
            )
            .unwrap();
        db
    }
    fn charge(conn: &Connection, date: &str, amount: i64) -> crate::model::Charge {
        charges::create(
            conn,
            NewCharge {
                owner_id: 1,
                patient_id: None,
                visit_id: None,
                service_id: None,
                inventory_item_id: None,
                description: "Synthetic service".into(),
                quantity: 1.0,
                unit_price_minor: amount,
                discount_minor: 0,
                vat_rate: None,
                charged_on: date.into(),
                notes: None,
            },
        )
        .unwrap()
    }
    fn pay(conn: &Connection, date: &str, amount: i64) -> crate::model::Payment {
        payments::create(
            conn,
            NewPayment {
                owner_id: 1,
                paid_on: date.into(),
                amount_minor: amount,
                method: PaymentMethod::Cash,
                notes: None,
            },
        )
        .unwrap()
    }
    #[test]
    fn rounding_discount_and_numeric_bounds() {
        assert_eq!(charges::total_minor(1.5, 101, 1).unwrap(), 151);
        assert_eq!(charges::total_minor(0.5, 1, 0).unwrap(), 1);
        assert_eq!(charges::total_minor(1.0, 100, 100).unwrap(), 0);
        for (q, p, d) in [
            (1.0, 100, 101),
            (0.0, 1, 0),
            (-1.0, 1, 0),
            (f64::NAN, 1, 0),
            (f64::INFINITY, 1, 0),
            (f64::MAX, 2, 0),
            (1.0, -1, 0),
            (1.0, 1, -1),
        ] {
            assert!(charges::total_minor(q, p, d).is_err());
        }
    }
    #[test]
    fn balances_deletions_restores_and_credit() {
        let db = fixture();
        let c = db.conn();
        let bill = charge(c, "2026-01-01", 1000);
        let payment = pay(c, "2026-01-02", 400);
        assert_eq!(balance(c, 1).unwrap(), 600);
        payments::soft_delete(c, payment.id).unwrap();
        assert_eq!(balance(c, 1).unwrap(), 1000);
        payments::restore(c, payment.id).unwrap();
        charges::soft_delete(c, bill.id).unwrap();
        assert_eq!(balance(c, 1).unwrap(), -400);
        assert!(receivables(c).unwrap().is_empty());
        charges::restore(c, bill.id).unwrap();
        assert_eq!(balance(c, 1).unwrap(), 600);
        assert_eq!(balance(c, 2).unwrap(), 0);
        assert!(balance(c, 99).is_err());
    }
    #[test]
    fn fifo_partial_full_prepayment_and_deleted_rows() {
        let db = fixture();
        let c = db.conn();
        let a = charge(c, "2026-01-01", 100);
        charge(c, "2026-02-01", 200);
        let p = pay(c, "2025-12-01", 99);
        assert_eq!(receivables(c).unwrap()[0].oldest_unpaid_on, "2026-01-01");
        pay(c, "2026-02-02", 1);
        let r = receivables(c).unwrap();
        assert_eq!(r[0].oldest_unpaid_on, "2026-02-01");
        assert_eq!(r[0].last_payment_on.as_deref(), Some("2026-02-02"));
        payments::soft_delete(c, p.id).unwrap();
        assert_eq!(receivables(c).unwrap()[0].oldest_unpaid_on, "2026-01-01");
        charges::soft_delete(c, a.id).unwrap();
        assert_eq!(receivables(c).unwrap()[0].oldest_unpaid_on, "2026-02-01");
        pay(c, "2026-03-01", 1000);
        assert!(receivables(c).unwrap().is_empty());
    }
    #[test]
    fn statements_order_ties_and_ignore_deleted_in_running_balance() {
        let db = fixture();
        let c = db.conn();
        let later = charge(c, "2026-02-01", 200);
        charge(c, "2026-01-01", 100);
        pay(c, "2026-01-01", 50);
        c.execute("UPDATE charges SET created_at='2026-01-01T00:00:00Z'", [])
            .unwrap();
        c.execute("UPDATE payments SET created_at='2026-01-01T00:00:00Z'", [])
            .unwrap();
        charges::soft_delete(c, later.id).unwrap();
        let rows = statement(c, 1, true).unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| r.running_balance_minor)
                .collect::<Vec<_>>(),
            vec![100, 50, 50]
        );
        assert_eq!(rows[0].kind, "charge");
        assert_eq!(rows[1].kind, "payment");
    }
    #[test]
    fn summary_dates_methods_and_validation() {
        let db = fixture();
        let c = db.conn();
        charge(c, "2026-01-01", 200);
        charge(c, "2026-02-01", 300);
        pay(c, "2026-01-01", 70);
        let p = payments::create(
            c,
            NewPayment {
                owner_id: 1,
                paid_on: "2026-01-02".into(),
                amount_minor: 30,
                method: PaymentMethod::Card,
                notes: None,
            },
        )
        .unwrap();
        let s = daily_summary(c, "2026-01-01", "2026-01-31").unwrap();
        assert_eq!((s.charges_minor, s.collections_minor), (200, 100));
        assert_eq!(s.collections_by_method["card"], 30);
        payments::soft_delete(c, p.id).unwrap();
        assert_eq!(
            daily_summary(c, "2026-01-01", "2026-01-31")
                .unwrap()
                .collections_minor,
            70
        );
        assert!(daily_summary(c, "2026-02-30", "2026-03-01").is_err());
        assert!(daily_summary(c, "2026-02-01", "2026-01-01").is_err());
    }
    #[test]
    fn service_crud_preserves_builtin_and_checks_vat() {
        let db = fixture();
        let c = db.conn();
        let input = NewService {
            name: "Exam".into(),
            category: ServiceCategory::Exam,
            price_minor: Some(12500),
            vat_rate: Some(20),
            is_active: true,
            notes: None,
        };
        let row = services::create(c, input.clone()).unwrap();
        c.execute("UPDATE services SET is_builtin=1 WHERE id=?1", [row.id])
            .unwrap();
        let mut edited = input;
        edited.price_minor = None;
        edited.is_active = false;
        assert!(
            services::update(c, row.id, edited.clone())
                .unwrap()
                .is_builtin
        );
        services::soft_delete(c, row.id).unwrap();
        assert!(services::list(c, false).unwrap().is_empty());
        assert_eq!(services::list(c, true).unwrap().len(), 1);
        services::restore(c, row.id).unwrap();
        edited.vat_rate = Some(101);
        assert!(services::update(c, row.id, edited).is_err());
    }
    #[test]
    fn charge_links_must_match_owner_and_patient() {
        let db = fixture();
        let c = db.conn();
        c.execute(
            "INSERT INTO patients(id,owner_id,name,species) VALUES(1,2,'Synthetic pet','cat')",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO visits(id,patient_id,visit_on) VALUES(1,1,'2026-01-01')",
            [],
        )
        .unwrap();
        let row = charge(c, "2026-01-01", 100);
        let mut input = NewCharge {
            owner_id: 1,
            patient_id: Some(1),
            visit_id: None,
            service_id: None,
            inventory_item_id: None,
            description: row.description,
            quantity: 1.0,
            unit_price_minor: 100,
            discount_minor: 0,
            vat_rate: None,
            charged_on: row.charged_on,
            notes: None,
        };
        assert!(charges::create(c, input.clone()).is_err());
        input.patient_id = None;
        input.visit_id = Some(1);
        assert!(charges::create(c, input).is_err());
    }
}
