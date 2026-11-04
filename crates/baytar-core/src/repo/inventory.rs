//! Lot stock and its immutable movement ledger are updated in one immediate transaction.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    const TODAY: &str = "2026-11-04";
    fn item(c: &Connection) -> InventoryItem {
        create(
            c,
            ItemInput {
                name: "Synthetic stock".into(),
                kind: "supply".into(),
                medication_id: None,
                vaccine_type_id: None,
                unit: "adet".into(),
                sale_price_minor: Some(1250),
                reorder_level: Some(5.0),
                is_active: true,
                notes: None,
            },
        )
        .unwrap()
    }
    fn lot(c: &Connection, id: i64, qty: f64, expires: Option<&str>) -> InventoryLot {
        receive(
            c,
            ReceiveInput {
                lot_id: None,
                lot: LotInput {
                    item_id: id,
                    lot_no: Some("TEST".into()),
                    expires_on: expires.map(str::to_owned),
                    quantity: qty,
                    cost_minor: Some(500),
                    received_on: Some(TODAY.into()),
                    notes: None,
                },
                moved_on: TODAY.into(),
            },
        )
        .unwrap()
    }
    fn usage(id: i64, quantity: f64) -> ConsumeInput {
        ConsumeInput {
            item_id: id,
            quantity,
            reason: "use".into(),
            moved_on: TODAY.into(),
            allow_negative: false,
            ref_table: None,
            ref_id: None,
            notes: None,
        }
    }
    #[test]
    fn fefo_splits_skips_expired_and_uses_undated_last() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        let undated = lot(c, i.id, 7.0, None);
        let late = lot(c, i.id, 4.0, Some("2027-01-01"));
        let early = lot(c, i.id, 3.0, Some(TODAY));
        let expired = lot(c, i.id, 9.0, Some("2026-11-03"));
        let used = consume(c, usage(i.id, 8.0)).unwrap();
        assert_eq!(
            used.iter().map(|m| (m.lot_id, m.delta)).collect::<Vec<_>>(),
            vec![
                (Some(early.id), -3.0),
                (Some(late.id), -4.0),
                (Some(undated.id), -1.0)
            ]
        );
        assert_eq!(lot_get(c, expired.id).unwrap().data.quantity, 9.0);
        assert_eq!(get(c, i.id).unwrap().stock, 15.0);
        let ledger: f64 = history(c, i.id).unwrap().iter().map(|m| m.delta).sum();
        assert_eq!(ledger, 15.0);
    }
    #[test]
    fn negative_guard_is_atomic_and_override_is_lotless() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        let l = lot(c, i.id, 2.0, None);
        assert!(consume(c, usage(i.id, 3.0)).is_err());
        assert_eq!(lot_get(c, l.id).unwrap().data.quantity, 2.0);
        assert_eq!(history(c, i.id).unwrap().len(), 1);
        let mut input = usage(i.id, 3.0);
        input.allow_negative = true;
        let used = consume(c, input).unwrap();
        assert_eq!(used.len(), 2);
        assert_eq!(used[1].lot_id, None);
        assert_eq!(used[1].delta, -1.0);
        let i = get(c, i.id).unwrap();
        assert_eq!(i.stock, 0.0);
        assert_eq!(i.unallocated_delta, -1.0);
        assert!(set_deleted(c, i.id, true).is_err());
    }
    #[test]
    fn failed_ledger_insert_rolls_back_receive_adjust_consume_and_writeoff() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        let a = lot(c, i.id, 2.0, Some("2026-12-01"));
        let b = lot(c, i.id, 2.0, Some("2026-12-02"));
        c.execute_batch(&format!("CREATE TRIGGER fail_second BEFORE INSERT ON stock_movements WHEN NEW.lot_id={} BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;",b.id)).unwrap();
        assert!(consume(c, usage(i.id, 3.0)).is_err());
        assert_eq!(lot_get(c, a.id).unwrap().data.quantity, 2.0);
        assert_eq!(history(c, i.id).unwrap().len(), 2);
        assert!(write_off_expired(c, i.id, "2027-01-01").is_err());
        assert_eq!(lot_get(c, a.id).unwrap().data.quantity, 2.0);
        assert!(adjust(
            c,
            AdjustInput {
                lot_id: b.id,
                delta: 1.0,
                moved_on: TODAY.into(),
                notes: "count".into()
            }
        )
        .is_err());
        assert_eq!(lot_get(c, b.id).unwrap().data.quantity, 2.0);
        c.execute_batch("DROP TRIGGER fail_second; CREATE TRIGGER fail_all BEFORE INSERT ON stock_movements BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        let input = ReceiveInput {
            lot_id: None,
            lot: a.data.clone(),
            moved_on: TODAY.into(),
        };
        assert!(receive(c, input).is_err());
        assert_eq!(lots(c, i.id, false).unwrap().len(), 2);
    }
    #[test]
    fn expiry_boundaries_low_stock_and_alert_counts_are_per_item() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        lot(c, i.id, 1.0, Some("2026-11-03"));
        lot(c, i.id, 1.0, Some(TODAY));
        lot(c, i.id, 1.0, Some("2027-01-03"));
        lot(c, i.id, 1.0, Some("2027-01-04"));
        lot(c, i.id, 0.5, None);
        let zero = lot(c, i.id, 1.0, Some("2026-11-01"));
        adjust(
            c,
            AdjustInput {
                lot_id: zero.id,
                delta: -1.0,
                moved_on: TODAY.into(),
                notes: "count".into(),
            },
        )
        .unwrap();
        assert_eq!(expiring_within(c, TODAY, 60).unwrap().len(), 2);
        assert_eq!(expired(c, TODAY).unwrap().len(), 1);
        assert_eq!(low_stock(c).unwrap().len(), 1);
        let alerts = alerts(c, TODAY).unwrap();
        assert_eq!(
            (
                alerts.item_count,
                alerts.low_stock_count,
                alerts.expiring_count,
                alerts.expired_count
            ),
            (1, 1, 1, 1)
        );
        let writes = write_off_expired(c, i.id, TODAY).unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].reason, "expired");
        assert!(write_off_expired(c, i.id, TODAY).unwrap().is_empty());
        let mut input = get(c, i.id).unwrap().data;
        input.is_active = false;
        update(c, i.id, input).unwrap();
        assert!(expired(c, TODAY).unwrap().is_empty());
        assert!(low_stock(c).unwrap().is_empty());
        assert!(expiring_within(c, "2026-02-30", 60).is_err());
    }
    #[test]
    fn lot_crud_tracks_quantity_edits_and_archives_only_empty_lots() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        let l = lot_create(
            c,
            LotInput {
                item_id: i.id,
                lot_no: None,
                expires_on: None,
                quantity: 2.0,
                cost_minor: None,
                received_on: None,
                notes: None,
            },
            TODAY,
        )
        .unwrap();
        let mut input = l.data.clone();
        input.quantity = 3.0;
        input.lot_no = Some("edited".into());
        assert!(lot_update(c, l.id, input.clone(), TODAY, "").is_err());
        lot_update(c, l.id, input, TODAY, "recount").unwrap();
        assert_eq!(get(c, i.id).unwrap().stock, 3.0);
        assert_eq!(history(c, i.id).unwrap().len(), 2);
        assert!(lot_set_deleted(c, l.id, true).is_err());
        assert!(set_deleted(c, i.id, true).is_err());
        adjust(
            c,
            AdjustInput {
                lot_id: l.id,
                delta: -3.0,
                moved_on: TODAY.into(),
                notes: "recount".into(),
            },
        )
        .unwrap();
        lot_set_deleted(c, l.id, true).unwrap();
        assert!(lots(c, i.id, false).unwrap().is_empty());
        lot_set_deleted(c, l.id, false).unwrap();
        set_deleted(c, i.id, true).unwrap();
        assert!(list(c, false).unwrap().is_empty());
        assert_eq!(list(c, true).unwrap().len(), 1);
        set_deleted(c, i.id, false).unwrap();
    }
    #[test]
    fn receive_existing_lot_and_validation_do_not_change_ledger_on_error() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        let l = lot(c, i.id, 2.0, None);
        receive(
            c,
            ReceiveInput {
                lot_id: Some(l.id),
                lot: l.data.clone(),
                moved_on: TODAY.into(),
            },
        )
        .unwrap();
        assert_eq!(get(c, i.id).unwrap().stock, 4.0);
        for qty in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(consume(c, usage(i.id, qty)).is_err());
        }
        assert!(adjust(
            c,
            AdjustInput {
                lot_id: l.id,
                delta: -5.0,
                moved_on: TODAY.into(),
                notes: "count".into()
            }
        )
        .is_err());
        let other = item(c);
        let mut wrong = l.data.clone();
        wrong.item_id = other.id;
        assert!(receive(
            c,
            ReceiveInput {
                lot_id: Some(l.id),
                lot: wrong,
                moved_on: TODAY.into()
            }
        )
        .is_err());
        assert_eq!(history(c, i.id).unwrap().len(), 2);
    }
    #[test]
    fn catalog_helper_fills_links_and_references_cannot_be_deducted_twice() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let med = super::super::medications::create(
            c,
            super::super::medications::NewMedication {
                name: "Synthetic medicine".into(),
                pack_unit: Some("ml".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let i = create_from_catalog(c, "medication", med.id).unwrap();
        assert_eq!(i.data.unit, "ml");
        assert_eq!(
            i.id,
            create_from_catalog(c, "medication", med.id).unwrap().id
        );
        assert!(create(c, i.data.clone()).is_err());
        c.execute_batch(&format!("INSERT INTO owners(id,full_name,search_text) VALUES(1,'Synthetic owner','synthetic'); INSERT INTO patients(id,owner_id,name,species,search_text) VALUES(1,1,'Synthetic patient','cat','synthetic'); INSERT INTO visits(id,patient_id,visit_on) VALUES(1,1,'2026-11-04'); INSERT INTO prescription_items(id,visit_id,medication_id,medication_name) VALUES(1,1,{},'Synthetic medicine');",med.id)).unwrap();
        lot(c, i.id, 2.0, None);
        let mut input = usage(i.id, 1.0);
        input.ref_table = Some("prescription_items".into());
        input.ref_id = Some(1);
        let moved = consume(c, input.clone()).unwrap();
        assert_eq!(moved[0].ref_id, Some(1));
        assert!(consume(c, input).is_err());
        assert_eq!(get(c, i.id).unwrap().stock, 1.0);
        let mut invalid = usage(i.id, 1.0);
        invalid.ref_table = Some("owners".into());
        invalid.ref_id = Some(1);
        assert!(consume(c, invalid).is_err());
    }
    #[test]
    fn fractional_stock_does_not_create_spurious_negative_movement() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let i = item(c);
        lot(c, i.id, 0.1, None);
        lot(c, i.id, 0.2, None);
        let rows = consume(c, usage(i.id, 0.1 + 0.2)).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|m| m.lot_id.is_some()));
        assert_eq!(get(c, i.id).unwrap().stock, 0.0);
    }
}
use super::{clean, date, invalid, now, required, transaction, RepoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    pub name: String,
    pub kind: String,
    pub medication_id: Option<i64>,
    pub vaccine_type_id: Option<i64>,
    pub unit: String,
    pub sale_price_minor: Option<i64>,
    pub reorder_level: Option<f64>,
    #[serde(default = "yes")]
    pub is_active: bool,
    pub notes: Option<String>,
}
fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: i64,
    #[serde(flatten)]
    pub data: ItemInput,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    /// Physical stock is the sum of non-deleted lots, including expired stock.
    pub stock: f64,
    pub nearest_expiry: Option<String>,
    /// Negative consumption without a lot remains visible separately from physical stock.
    pub unallocated_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LotInput {
    pub item_id: i64,
    pub lot_no: Option<String>,
    pub expires_on: Option<String>,
    #[serde(default)]
    pub quantity: f64,
    pub cost_minor: Option<i64>,
    pub received_on: Option<String>,
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryLot {
    pub id: i64,
    #[serde(flatten)]
    pub data: LotInput,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovement {
    pub id: i64,
    pub item_id: i64,
    pub lot_id: Option<i64>,
    pub delta: f64,
    pub reason: String,
    pub ref_table: Option<String>,
    pub ref_id: Option<i64>,
    pub moved_on: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveInput {
    /// An existing lot may be selected explicitly; otherwise a new lot is created.
    pub lot_id: Option<i64>,
    pub lot: LotInput,
    pub moved_on: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustInput {
    pub lot_id: i64,
    pub delta: f64,
    pub moved_on: String,
    pub notes: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeInput {
    pub item_id: i64,
    pub quantity: f64,
    pub reason: String,
    pub moved_on: String,
    #[serde(default)]
    pub allow_negative: bool,
    pub ref_table: Option<String>,
    pub ref_id: Option<i64>,
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiringLot {
    pub item: InventoryItem,
    pub lot: InventoryLot,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryAlerts {
    pub item_count: usize,
    pub low_stock_count: usize,
    pub expiring_count: usize,
    pub expired_count: usize,
    pub low_stock: Vec<InventoryItem>,
    pub expiring: Vec<ExpiringLot>,
    pub expired: Vec<ExpiringLot>,
}

const ITEM_COLUMNS: &str = "i.id,i.name,i.kind,i.medication_id,i.vaccine_type_id,i.unit,i.sale_price_minor,i.reorder_level,i.is_active,i.notes,i.created_at,i.updated_at,i.deleted_at,
 (SELECT COALESCE(SUM(quantity),0.0) FROM inventory_lots WHERE item_id=i.id AND deleted_at IS NULL),
 (SELECT MIN(expires_on) FROM inventory_lots WHERE item_id=i.id AND deleted_at IS NULL AND quantity>0),
 (SELECT COALESCE(SUM(delta),0.0) FROM stock_movements WHERE item_id=i.id AND lot_id IS NULL AND deleted_at IS NULL)";
const LOT_COLUMNS: &str = "id,item_id,lot_no,expires_on,quantity,cost_minor,received_on,notes,created_at,updated_at,deleted_at";
fn item_row(r: &Row<'_>) -> rusqlite::Result<InventoryItem> {
    Ok(InventoryItem {
        id: r.get(0)?,
        data: ItemInput {
            name: r.get(1)?,
            kind: r.get(2)?,
            medication_id: r.get(3)?,
            vaccine_type_id: r.get(4)?,
            unit: r.get(5)?,
            sale_price_minor: r.get(6)?,
            reorder_level: r.get(7)?,
            is_active: r.get(8)?,
            notes: r.get(9)?,
        },
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
        deleted_at: r.get(12)?,
        stock: r.get(13)?,
        nearest_expiry: r.get(14)?,
        unallocated_delta: r.get(15)?,
    })
}
fn lot_row(r: &Row<'_>) -> rusqlite::Result<InventoryLot> {
    Ok(InventoryLot {
        id: r.get(0)?,
        data: LotInput {
            item_id: r.get(1)?,
            lot_no: r.get(2)?,
            expires_on: r.get(3)?,
            quantity: r.get(4)?,
            cost_minor: r.get(5)?,
            received_on: r.get(6)?,
            notes: r.get(7)?,
        },
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
        deleted_at: r.get(10)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<InventoryItem> {
    conn.query_row(
        &format!("SELECT {ITEM_COLUMNS} FROM inventory_items i WHERE i.id=?1"),
        [id],
        item_row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
fn active_item(conn: &Connection, id: i64) -> Result<InventoryItem> {
    let item = get(conn, id)?;
    if item.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    Ok(item)
}
fn usable_item(conn: &Connection, id: i64) -> Result<InventoryItem> {
    let item = active_item(conn, id)?;
    if !item.data.is_active {
        return Err(RepoError::Conflict);
    }
    Ok(item)
}
pub fn list(conn: &Connection, include_deleted: bool) -> Result<Vec<InventoryItem>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ITEM_COLUMNS} FROM inventory_items i WHERE (?1 OR i.deleted_at IS NULL)"
    ))?;
    let mut rows = stmt
        .query_map([include_deleted], item_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    rows.sort_by_key(|i| (crate::text::fold(&i.data.name), i.id));
    Ok(rows)
}
fn nonnegative(field: &str, value: f64) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(invalid(field, "must be finite and nonnegative"));
    }
    Ok(())
}
fn money(field: &str, value: Option<i64>) -> Result<()> {
    if value.is_some_and(|v| v < 0) {
        return Err(invalid(field, "must be nonnegative"));
    }
    Ok(())
}
fn prepare_item(conn: &Connection, input: &mut ItemInput, id: Option<i64>) -> Result<()> {
    required("name", &input.name)?;
    required("unit", &input.unit)?;
    if ![
        "medication",
        "vaccine",
        "supply",
        "food",
        "product",
        "other",
    ]
    .contains(&input.kind.as_str())
    {
        return Err(invalid("kind", "unknown inventory kind"));
    }
    money("salePriceMinor", input.sale_price_minor)?;
    if let Some(v) = input.reorder_level {
        nonnegative("reorderLevel", v)?;
    }
    if input.medication_id.is_some() && input.vaccine_type_id.is_some() {
        return Err(invalid("medicationId", "only one catalog link is allowed"));
    }
    if let Some(link) = input.medication_id {
        if input.kind != "medication" {
            return Err(invalid("kind", "catalog link requires medication kind"));
        }
        if super::medications::get(conn, link)?.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    if let Some(link) = input.vaccine_type_id {
        if input.kind != "vaccine" {
            return Err(invalid("kind", "catalog link requires vaccine kind"));
        }
        let v = super::vaccine_types::get(conn, link)?;
        if v.deleted_at.is_some() || !v.is_active {
            return Err(RepoError::NotFound);
        }
    }
    if conn.query_row("SELECT EXISTS(SELECT 1 FROM inventory_items WHERE deleted_at IS NULL AND (?1 IS NULL OR id<>?1) AND ((?2 IS NOT NULL AND medication_id=?2) OR (?3 IS NOT NULL AND vaccine_type_id=?3)))",params![id,input.medication_id,input.vaccine_type_id],|r|r.get::<_,bool>(0))? { return Err(RepoError::Conflict); }
    input.name = input.name.trim().into();
    input.unit = input.unit.trim().into();
    input.notes = clean(input.notes.take());
    Ok(())
}
fn insert_item(conn: &Connection, input: &ItemInput) -> Result<InventoryItem> {
    let stamp = now();
    conn.execute("INSERT INTO inventory_items(name,kind,medication_id,vaccine_type_id,unit,sale_price_minor,reorder_level,is_active,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",params![input.name,input.kind,input.medication_id,input.vaccine_type_id,input.unit,input.sale_price_minor,input.reorder_level,input.is_active,input.notes,stamp])?;
    get(conn, conn.last_insert_rowid())
}
pub fn create(conn: &Connection, mut input: ItemInput) -> Result<InventoryItem> {
    let tx = transaction(conn)?;
    prepare_item(&tx, &mut input, None)?;
    let result = insert_item(&tx, &input)?;
    tx.commit()?;
    Ok(result)
}
/// Catalog creation is idempotent: the active link already in stock is returned.
pub fn create_from_catalog(
    conn: &Connection,
    kind: &str,
    catalog_id: i64,
) -> Result<InventoryItem> {
    let tx = transaction(conn)?;
    let (name, unit, medication_id, vaccine_type_id) = match kind {
        "medication" => {
            let m = super::medications::get(&tx, catalog_id)?;
            if m.deleted_at.is_some() {
                return Err(RepoError::NotFound);
            }
            (
                m.name,
                m.pack_unit.unwrap_or_else(|| "adet".into()),
                Some(catalog_id),
                None,
            )
        }
        "vaccine" => {
            let v = super::vaccine_types::get(&tx, catalog_id)?;
            if v.deleted_at.is_some() || !v.is_active {
                return Err(RepoError::NotFound);
            }
            (v.name, "doz".into(), None, Some(catalog_id))
        }
        _ => return Err(invalid("kind", "expected medication or vaccine")),
    };
    let existing=tx.query_row("SELECT id FROM inventory_items WHERE deleted_at IS NULL AND ((?1 IS NOT NULL AND medication_id=?1) OR (?2 IS NOT NULL AND vaccine_type_id=?2))",params![medication_id,vaccine_type_id],|r|r.get::<_,i64>(0)).optional()?;
    let result = if let Some(id) = existing {
        get(&tx, id)?
    } else {
        let mut input = ItemInput {
            name,
            kind: kind.into(),
            medication_id,
            vaccine_type_id,
            unit,
            sale_price_minor: None,
            reorder_level: None,
            is_active: true,
            notes: None,
        };
        prepare_item(&tx, &mut input, None)?;
        insert_item(&tx, &input)?
    };
    tx.commit()?;
    Ok(result)
}
/// Updates all editable fields; quantities can only change through lot operations.
pub fn update(conn: &Connection, id: i64, mut input: ItemInput) -> Result<InventoryItem> {
    let tx = transaction(conn)?;
    active_item(&tx, id)?;
    prepare_item(&tx, &mut input, Some(id))?;
    tx.execute("UPDATE inventory_items SET name=?1,kind=?2,medication_id=?3,vaccine_type_id=?4,unit=?5,sale_price_minor=?6,reorder_level=?7,is_active=?8,notes=?9,updated_at=?10 WHERE id=?11",params![input.name,input.kind,input.medication_id,input.vaccine_type_id,input.unit,input.sale_price_minor,input.reorder_level,input.is_active,input.notes,now(),id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<InventoryItem> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() == deleted
        || (deleted && (old.stock != 0.0 || old.unallocated_delta != 0.0))
    {
        return Err(RepoError::Conflict);
    }
    if !deleted {
        let mut data = old.data;
        prepare_item(&tx, &mut data, Some(id))?;
    }
    let stamp = now();
    tx.execute(
        "UPDATE inventory_items SET deleted_at=?1,updated_at=?2 WHERE id=?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn lot_get(conn: &Connection, id: i64) -> Result<InventoryLot> {
    conn.query_row(
        &format!("SELECT {LOT_COLUMNS} FROM inventory_lots WHERE id=?1"),
        [id],
        lot_row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
fn active_lot(conn: &Connection, id: i64) -> Result<InventoryLot> {
    let lot = lot_get(conn, id)?;
    active_item(conn, lot.data.item_id)?;
    if lot.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    Ok(lot)
}
pub fn lots(conn: &Connection, item_id: i64, include_deleted: bool) -> Result<Vec<InventoryLot>> {
    get(conn, item_id)?;
    let mut stmt=conn.prepare(&format!("SELECT {LOT_COLUMNS} FROM inventory_lots WHERE item_id=?1 AND (?2 OR deleted_at IS NULL) ORDER BY expires_on IS NULL,expires_on,received_on,id"))?;
    let rows = stmt
        .query_map(params![item_id, include_deleted], lot_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
fn prepare_lot(input: &mut LotInput) -> Result<()> {
    nonnegative("quantity", input.quantity)?;
    money("costMinor", input.cost_minor)?;
    if let Some(v) = &input.expires_on {
        date("expiresOn", v)?;
    }
    if let Some(v) = &input.received_on {
        date("receivedOn", v)?;
    }
    input.lot_no = clean(input.lot_no.take());
    input.notes = clean(input.notes.take());
    Ok(())
}
fn insert_lot(conn: &Connection, input: &LotInput) -> Result<i64> {
    let stamp = now();
    conn.execute("INSERT INTO inventory_lots(item_id,lot_no,expires_on,cost_minor,received_on,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",params![input.item_id,input.lot_no,input.expires_on,input.cost_minor,input.received_on,input.notes,stamp])?;
    Ok(conn.last_insert_rowid())
}
struct Movement<'a> {
    item_id: i64,
    lot_id: Option<i64>,
    delta: f64,
    reason: &'a str,
    moved_on: &'a str,
    notes: Option<&'a str>,
    reference: Option<(&'a str, i64)>,
}
fn movement(conn: &Connection, m: Movement<'_>) -> Result<StockMovement> {
    if !m.delta.is_finite() || m.delta == 0.0 {
        return Err(invalid("delta", "must be finite and nonzero"));
    }
    let stamp = now();
    if let Some(id) = m.lot_id {
        let old = active_lot(conn, id)?;
        if old.data.item_id != m.item_id {
            return Err(RepoError::Conflict);
        }
        let next = old.data.quantity + m.delta;
        nonnegative("quantity", next)?;
        if next == old.data.quantity {
            return Err(invalid(
                "delta",
                "quantity change is below storage precision",
            ));
        }
        conn.execute(
            "UPDATE inventory_lots SET quantity=?1,updated_at=?2 WHERE id=?3",
            params![next, stamp, id],
        )?;
    }
    let (ref_table, ref_id) = m
        .reference
        .map(|(t, i)| (Some(t), Some(i)))
        .unwrap_or((None, None));
    conn.execute("INSERT INTO stock_movements(item_id,lot_id,delta,reason,ref_table,ref_id,moved_on,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)",params![m.item_id,m.lot_id,m.delta,m.reason,ref_table,ref_id,m.moved_on,m.notes,stamp])?;
    conn.execute(
        "UPDATE inventory_items SET updated_at=?1 WHERE id=?2",
        params![stamp, m.item_id],
    )?;
    Ok(StockMovement {
        id: conn.last_insert_rowid(),
        item_id: m.item_id,
        lot_id: m.lot_id,
        delta: m.delta,
        reason: m.reason.into(),
        ref_table: ref_table.map(str::to_owned),
        ref_id,
        moved_on: m.moved_on.into(),
        notes: m.notes.map(str::to_owned),
        created_at: stamp.clone(),
        updated_at: stamp,
        deleted_at: None,
    })
}
pub fn lot_create(conn: &Connection, mut input: LotInput, moved_on: &str) -> Result<InventoryLot> {
    date("movedOn", moved_on)?;
    prepare_lot(&mut input)?;
    let tx = transaction(conn)?;
    usable_item(&tx, input.item_id)?;
    let id = insert_lot(&tx, &input)?;
    if input.quantity != 0.0 {
        movement(
            &tx,
            Movement {
                item_id: input.item_id,
                lot_id: Some(id),
                delta: input.quantity,
                reason: "adjust",
                moved_on,
                notes: input.notes.as_deref(),
                reference: None,
            },
        )?;
    }
    let result = lot_get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
/// Quantity edits are adjustments and require an explanation. Item identity cannot change.
pub fn lot_update(
    conn: &Connection,
    id: i64,
    mut input: LotInput,
    moved_on: &str,
    reason: &str,
) -> Result<InventoryLot> {
    date("movedOn", moved_on)?;
    prepare_lot(&mut input)?;
    let tx = transaction(conn)?;
    let old = active_lot(&tx, id)?;
    if old.data.item_id != input.item_id {
        return Err(RepoError::Conflict);
    }
    let delta = input.quantity - old.data.quantity;
    if delta != 0.0 {
        required("notes", reason)?;
        movement(
            &tx,
            Movement {
                item_id: input.item_id,
                lot_id: Some(id),
                delta,
                reason: "adjust",
                moved_on,
                notes: Some(reason),
                reference: None,
            },
        )?;
    }
    tx.execute("UPDATE inventory_lots SET lot_no=?1,expires_on=?2,cost_minor=?3,received_on=?4,notes=?5,updated_at=?6 WHERE id=?7",params![input.lot_no,input.expires_on,input.cost_minor,input.received_on,input.notes,now(),id])?;
    let result = lot_get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
/// Only empty lots can be archived; stocked lots must first be adjusted or written off.
pub fn lot_set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<InventoryLot> {
    let tx = transaction(conn)?;
    let old = lot_get(&tx, id)?;
    active_item(&tx, old.data.item_id)?;
    if old.deleted_at.is_some() == deleted || old.data.quantity != 0.0 {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE inventory_lots SET deleted_at=?1,updated_at=?2 WHERE id=?3",
        params![if deleted { Some(&stamp) } else { None }, stamp, id],
    )?;
    let result = lot_get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn receive(conn: &Connection, mut input: ReceiveInput) -> Result<InventoryLot> {
    date("movedOn", &input.moved_on)?;
    prepare_lot(&mut input.lot)?;
    super::positive("quantity", Some(input.lot.quantity))?;
    let tx = transaction(conn)?;
    usable_item(&tx, input.lot.item_id)?;
    let id = if let Some(id) = input.lot_id {
        let old = active_lot(&tx, id)?;
        if old.data.item_id != input.lot.item_id {
            return Err(RepoError::Conflict);
        }
        id
    } else {
        insert_lot(&tx, &input.lot)?
    };
    movement(
        &tx,
        Movement {
            item_id: input.lot.item_id,
            lot_id: Some(id),
            delta: input.lot.quantity,
            reason: "purchase",
            moved_on: &input.moved_on,
            notes: input.lot.notes.as_deref(),
            reference: None,
        },
    )?;
    let result = lot_get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn adjust(conn: &Connection, input: AdjustInput) -> Result<StockMovement> {
    date("movedOn", &input.moved_on)?;
    required("notes", &input.notes)?;
    let tx = transaction(conn)?;
    let lot = active_lot(&tx, input.lot_id)?;
    let result = movement(
        &tx,
        Movement {
            item_id: lot.data.item_id,
            lot_id: Some(input.lot_id),
            delta: input.delta,
            reason: "adjust",
            moved_on: &input.moved_on,
            notes: Some(input.notes.trim()),
            reference: None,
        },
    )?;
    tx.commit()?;
    Ok(result)
}
pub fn write_off_expired(
    conn: &Connection,
    item_id: i64,
    today: &str,
) -> Result<Vec<StockMovement>> {
    date("today", today)?;
    let tx = transaction(conn)?;
    active_item(&tx, item_id)?;
    let expired: Vec<_> = lots(&tx, item_id, false)?
        .into_iter()
        .filter(|l| {
            l.data.quantity > 0.0 && l.data.expires_on.as_deref().is_some_and(|d| d < today)
        })
        .collect();
    let mut result = Vec::new();
    for lot in expired {
        result.push(movement(
            &tx,
            Movement {
                item_id,
                lot_id: Some(lot.id),
                delta: -lot.data.quantity,
                reason: "expired",
                moved_on: today,
                notes: None,
                reference: None,
            },
        )?);
    }
    tx.commit()?;
    Ok(result)
}
/// FEFO ignores expired/deleted/empty lots, with undated lots last and id as final tie-breaker.
pub fn consume(conn: &Connection, input: ConsumeInput) -> Result<Vec<StockMovement>> {
    date("movedOn", &input.moved_on)?;
    super::positive("quantity", Some(input.quantity))?;
    if !["use", "sale"].contains(&input.reason.as_str()) {
        return Err(invalid("reason", "expected use or sale"));
    }
    let tx = transaction(conn)?;
    let item = usable_item(&tx, input.item_id)?;
    let reference = match (input.ref_table.as_deref(), input.ref_id) {
        (None, None) => None,
        (Some(table @ ("vaccinations" | "prescription_items")), Some(id)) => {
            let column = if table == "vaccinations" {
                "vaccine_type_id"
            } else {
                "medication_id"
            };
            let linked = tx
                .query_row(
                    &format!("SELECT {column} FROM {table} WHERE id=?1 AND deleted_at IS NULL"),
                    [id],
                    |r| r.get::<_, Option<i64>>(0),
                )
                .optional()?
                .ok_or(RepoError::NotFound)?;
            let expected = if table == "vaccinations" {
                item.data.vaccine_type_id
            } else {
                item.data.medication_id
            };
            if expected.is_none() || linked != expected {
                return Err(invalid("refId", "record must use the linked catalog entry"));
            }
            if tx.query_row("SELECT EXISTS(SELECT 1 FROM stock_movements WHERE item_id=?1 AND ref_table=?2 AND ref_id=?3 AND deleted_at IS NULL)",params![input.item_id,table,id],|r|r.get::<_,bool>(0))?{return Err(RepoError::Conflict);}
            Some((table, id))
        }
        _ => {
            return Err(invalid(
                "refTable",
                "expected a clinical table and matching refId",
            ))
        }
    };
    let eligible: Vec<_> = lots(&tx, input.item_id, false)?
        .into_iter()
        .filter(|l| {
            l.data.quantity > 0.0
                && l.data
                    .expires_on
                    .as_deref()
                    .is_none_or(|d| d >= input.moved_on.as_str())
        })
        .collect();
    let available: f64 = eligible.iter().map(|l| l.data.quantity).sum();
    let tolerance = input.quantity.max(available) * f64::EPSILON * 4.0;
    if !input.allow_negative && available < input.quantity && input.quantity - available > tolerance
    {
        return Err(invalid("quantity", "insufficient unexpired stock"));
    }
    let mut remaining = input.quantity;
    let mut result = Vec::new();
    for lot in eligible {
        if remaining <= 0.0 {
            break;
        }
        let take = remaining.min(lot.data.quantity);
        result.push(movement(
            &tx,
            Movement {
                item_id: input.item_id,
                lot_id: Some(lot.id),
                delta: -take,
                reason: &input.reason,
                moved_on: &input.moved_on,
                notes: input.notes.as_deref(),
                reference,
            },
        )?);
        remaining -= take;
        if remaining <= tolerance {
            remaining = 0.0;
        }
    }
    if remaining > 0.0 {
        result.push(movement(
            &tx,
            Movement {
                item_id: input.item_id,
                lot_id: None,
                delta: -remaining,
                reason: &input.reason,
                moved_on: &input.moved_on,
                notes: input.notes.as_deref(),
                reference,
            },
        )?);
    }
    tx.commit()?;
    Ok(result)
}
pub fn history(conn: &Connection, item_id: i64) -> Result<Vec<StockMovement>> {
    get(conn, item_id)?;
    let mut stmt=conn.prepare("SELECT id,item_id,lot_id,delta,reason,ref_table,ref_id,moved_on,notes,created_at,updated_at,deleted_at FROM stock_movements WHERE item_id=?1 AND deleted_at IS NULL ORDER BY moved_on DESC,id DESC")?;
    let rows = stmt
        .query_map([item_id], |r| {
            Ok(StockMovement {
                id: r.get(0)?,
                item_id: r.get(1)?,
                lot_id: r.get(2)?,
                delta: r.get(3)?,
                reason: r.get(4)?,
                ref_table: r.get(5)?,
                ref_id: r.get(6)?,
                moved_on: r.get(7)?,
                notes: r.get(8)?,
                created_at: r.get(9)?,
                updated_at: r.get(10)?,
                deleted_at: r.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn low_stock(conn: &Connection) -> Result<Vec<InventoryItem>> {
    Ok(list(conn, false)?
        .into_iter()
        .filter(|i| i.data.is_active && i.data.reorder_level.is_some_and(|v| i.stock <= v))
        .collect())
}
pub fn expiring_within(conn: &Connection, today: &str, days: u32) -> Result<Vec<ExpiringLot>> {
    let end = date("today", today)?
        .checked_add_signed(chrono::Duration::days(i64::from(days)))
        .ok_or_else(|| invalid("days", "date overflow"))?
        .format("%Y-%m-%d")
        .to_string();
    expiry_query(conn, today, Some(&end))
}
pub fn expired(conn: &Connection, today: &str) -> Result<Vec<ExpiringLot>> {
    date("today", today)?;
    expiry_query(conn, today, None)
}
fn expiry_query(conn: &Connection, today: &str, end: Option<&str>) -> Result<Vec<ExpiringLot>> {
    let mut stmt=conn.prepare("SELECT l.id,l.item_id FROM inventory_lots l JOIN inventory_items i ON i.id=l.item_id WHERE l.deleted_at IS NULL AND i.deleted_at IS NULL AND i.is_active=1 AND l.quantity>0 AND ((?2 IS NULL AND l.expires_on<?1) OR (?2 IS NOT NULL AND l.expires_on BETWEEN ?1 AND ?2)) ORDER BY l.expires_on,l.item_id,l.id")?;
    let ids = stmt
        .query_map(params![today, end], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|(l, i)| {
            Ok(ExpiringLot {
                item: get(conn, i)?,
                lot: lot_get(conn, l)?,
            })
        })
        .collect()
}
pub fn alerts(conn: &Connection, today: &str) -> Result<InventoryAlerts> {
    // A single read transaction keeps all counts and previews consistent.
    let tx = transaction(conn)?;
    let mut low_stock = low_stock(&tx)?;
    let mut expiring = expiring_within(&tx, today, 60)?;
    let mut expired = expired(&tx, today)?;
    fn distinct(rows: &mut Vec<ExpiringLot>) -> usize {
        let mut seen = std::collections::HashSet::new();
        rows.retain(|r| seen.insert(r.item.id));
        rows.len()
    }
    let result = InventoryAlerts {
        item_count: list(&tx, false)?.len(),
        low_stock_count: low_stock.len(),
        expiring_count: distinct(&mut expiring),
        expired_count: distinct(&mut expired),
        low_stock: {
            low_stock.truncate(5);
            low_stock
        },
        expiring: {
            expiring.truncate(5);
            expiring
        },
        expired: {
            expired.truncate(5);
            expired
        },
    };
    tx.commit()?;
    Ok(result)
}
