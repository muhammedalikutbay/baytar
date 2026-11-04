//! Thin inventory commands; all validation and transactions live in baytar-core.
use crate::{AppDatabase, CommandError};
use baytar_core::repo::inventory::{
    self, AdjustInput, ConsumeInput, ExpiringLot, InventoryAlerts, InventoryItem, InventoryLot,
    ItemInput, LotInput, ReceiveInput, StockMovement,
};
use tauri::State;

#[tauri::command]
pub fn inventory_items(
    state: State<'_, AppDatabase>,
    include_deleted: bool,
) -> Result<Vec<InventoryItem>, CommandError> {
    state.with(|c| inventory::list(c, include_deleted))
}
#[tauri::command]
pub fn inventory_item_get(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<InventoryItem, CommandError> {
    state.with(|c| inventory::get(c, id))
}
#[tauri::command]
pub fn inventory_item_create(
    state: State<'_, AppDatabase>,
    input: ItemInput,
) -> Result<InventoryItem, CommandError> {
    state.with(|c| inventory::create(c, input))
}
#[tauri::command]
pub fn inventory_item_update(
    state: State<'_, AppDatabase>,
    id: i64,
    input: ItemInput,
) -> Result<InventoryItem, CommandError> {
    state.with(|c| inventory::update(c, id, input))
}
#[tauri::command]
pub fn inventory_item_set_deleted(
    state: State<'_, AppDatabase>,
    id: i64,
    deleted: bool,
) -> Result<InventoryItem, CommandError> {
    state.with(|c| inventory::set_deleted(c, id, deleted))
}
#[tauri::command]
pub fn inventory_from_catalog(
    state: State<'_, AppDatabase>,
    kind: String,
    catalog_id: i64,
) -> Result<InventoryItem, CommandError> {
    state.with(|c| inventory::create_from_catalog(c, &kind, catalog_id))
}
#[tauri::command]
pub fn inventory_lots(
    state: State<'_, AppDatabase>,
    item_id: i64,
    include_deleted: bool,
) -> Result<Vec<InventoryLot>, CommandError> {
    state.with(|c| inventory::lots(c, item_id, include_deleted))
}
#[tauri::command]
pub fn inventory_lot_create(
    state: State<'_, AppDatabase>,
    input: LotInput,
    moved_on: String,
) -> Result<InventoryLot, CommandError> {
    state.with(|c| inventory::lot_create(c, input, &moved_on))
}
#[tauri::command]
pub fn inventory_lot_update(
    state: State<'_, AppDatabase>,
    id: i64,
    input: LotInput,
    moved_on: String,
    reason: String,
) -> Result<InventoryLot, CommandError> {
    state.with(|c| inventory::lot_update(c, id, input, &moved_on, &reason))
}
#[tauri::command]
pub fn inventory_lot_set_deleted(
    state: State<'_, AppDatabase>,
    id: i64,
    deleted: bool,
) -> Result<InventoryLot, CommandError> {
    state.with(|c| inventory::lot_set_deleted(c, id, deleted))
}
#[tauri::command]
pub fn inventory_receive(
    state: State<'_, AppDatabase>,
    input: ReceiveInput,
) -> Result<InventoryLot, CommandError> {
    state.with(|c| inventory::receive(c, input))
}
#[tauri::command]
pub fn inventory_adjust(
    state: State<'_, AppDatabase>,
    input: AdjustInput,
) -> Result<StockMovement, CommandError> {
    state.with(|c| inventory::adjust(c, input))
}
#[tauri::command]
pub fn inventory_write_off_expired(
    state: State<'_, AppDatabase>,
    item_id: i64,
    today: String,
) -> Result<Vec<StockMovement>, CommandError> {
    state.with(|c| inventory::write_off_expired(c, item_id, &today))
}
#[tauri::command]
pub fn inventory_consume(
    state: State<'_, AppDatabase>,
    input: ConsumeInput,
) -> Result<Vec<StockMovement>, CommandError> {
    state.with(|c| inventory::consume(c, input))
}
#[tauri::command]
pub fn inventory_history(
    state: State<'_, AppDatabase>,
    item_id: i64,
) -> Result<Vec<StockMovement>, CommandError> {
    state.with(|c| inventory::history(c, item_id))
}
#[tauri::command]
pub fn inventory_low_stock(
    state: State<'_, AppDatabase>,
) -> Result<Vec<InventoryItem>, CommandError> {
    state.with(inventory::low_stock)
}
#[tauri::command]
pub fn inventory_expiring(
    state: State<'_, AppDatabase>,
    today: String,
    days: u32,
) -> Result<Vec<ExpiringLot>, CommandError> {
    state.with(|c| inventory::expiring_within(c, &today, days))
}
#[tauri::command]
pub fn inventory_expired(
    state: State<'_, AppDatabase>,
    today: String,
) -> Result<Vec<ExpiringLot>, CommandError> {
    state.with(|c| inventory::expired(c, &today))
}
#[tauri::command]
pub fn inventory_alerts(
    state: State<'_, AppDatabase>,
    today: Option<String>,
) -> Result<InventoryAlerts, CommandError> {
    let today = today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    state.with(|c| inventory::alerts(c, &today))
}
