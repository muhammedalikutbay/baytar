use crate::{AppDatabase, CommandError};
use baytar_core::{
    model::{Charge, Payment, Service},
    repo::{
        accounts::{self, ChargeSource, DailySummary, Receivable, StatementEntry},
        charges::{self, NewCharge},
        payments::{self, NewPayment},
        services::{self, NewService},
    },
};
use tauri::State;
#[tauri::command]
pub fn service_get(state: State<'_, AppDatabase>, id: i64) -> Result<Service, CommandError> {
    state.with(|conn| services::get(conn, id))
}
#[tauri::command]
pub fn service_create(
    state: State<'_, AppDatabase>,
    input: NewService,
) -> Result<Service, CommandError> {
    state.with(|conn| services::create(conn, input))
}
#[tauri::command]
pub fn service_update(
    state: State<'_, AppDatabase>,
    id: i64,
    input: NewService,
) -> Result<Service, CommandError> {
    state.with(|conn| services::update(conn, id, input))
}
#[tauri::command]
pub fn service_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Service, CommandError> {
    state.with(|conn| services::soft_delete(conn, id))
}
#[tauri::command]
pub fn service_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Service, CommandError> {
    state.with(|conn| services::restore(conn, id))
}
#[tauri::command]
pub fn services_list(
    state: State<'_, AppDatabase>,
    include_deleted: bool,
) -> Result<Vec<Service>, CommandError> {
    state.with(|conn| services::list(conn, include_deleted))
}
#[tauri::command]
pub fn charge_get(state: State<'_, AppDatabase>, id: i64) -> Result<Charge, CommandError> {
    state.with(|conn| charges::get(conn, id))
}
#[tauri::command]
pub fn charge_create(
    state: State<'_, AppDatabase>,
    input: NewCharge,
) -> Result<Charge, CommandError> {
    state.with(|conn| charges::create(conn, input))
}
#[tauri::command]
pub fn charge_update(
    state: State<'_, AppDatabase>,
    id: i64,
    input: NewCharge,
) -> Result<Charge, CommandError> {
    state.with(|conn| charges::update(conn, id, input))
}
#[tauri::command]
pub fn charge_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Charge, CommandError> {
    state.with(|conn| charges::soft_delete(conn, id))
}
#[tauri::command]
pub fn charge_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Charge, CommandError> {
    state.with(|conn| charges::restore(conn, id))
}
#[tauri::command]
pub fn charges_list(
    state: State<'_, AppDatabase>,
    include_deleted: bool,
) -> Result<Vec<Charge>, CommandError> {
    state.with(|conn| charges::list(conn, include_deleted))
}
#[tauri::command]
pub fn payment_get(state: State<'_, AppDatabase>, id: i64) -> Result<Payment, CommandError> {
    state.with(|conn| payments::get(conn, id))
}
#[tauri::command]
pub fn payment_create(
    state: State<'_, AppDatabase>,
    input: NewPayment,
) -> Result<Payment, CommandError> {
    state.with(|conn| payments::create(conn, input))
}
#[tauri::command]
pub fn payment_update(
    state: State<'_, AppDatabase>,
    id: i64,
    input: NewPayment,
) -> Result<Payment, CommandError> {
    state.with(|conn| payments::update(conn, id, input))
}
#[tauri::command]
pub fn payment_delete(state: State<'_, AppDatabase>, id: i64) -> Result<Payment, CommandError> {
    state.with(|conn| payments::soft_delete(conn, id))
}
#[tauri::command]
pub fn payment_restore(state: State<'_, AppDatabase>, id: i64) -> Result<Payment, CommandError> {
    state.with(|conn| payments::restore(conn, id))
}
#[tauri::command]
pub fn payments_list(
    state: State<'_, AppDatabase>,
    include_deleted: bool,
) -> Result<Vec<Payment>, CommandError> {
    state.with(|conn| payments::list(conn, include_deleted))
}
#[tauri::command]
pub fn charges_by_visit(
    state: State<'_, AppDatabase>,
    visit_id: i64,
    include_deleted: bool,
) -> Result<Vec<Charge>, CommandError> {
    state.with(|conn| charges::by_visit(conn, visit_id, include_deleted))
}
#[tauri::command]
pub fn account_balance(state: State<'_, AppDatabase>, owner_id: i64) -> Result<i64, CommandError> {
    state.with(|conn| accounts::balance(conn, owner_id))
}
#[tauri::command]
pub fn account_statement(
    state: State<'_, AppDatabase>,
    owner_id: i64,
    include_deleted: bool,
) -> Result<Vec<StatementEntry>, CommandError> {
    state.with(|conn| accounts::statement(conn, owner_id, include_deleted))
}
#[tauri::command]
pub fn accounts_receivables(
    state: State<'_, AppDatabase>,
) -> Result<Vec<Receivable>, CommandError> {
    state.with(accounts::receivables)
}
#[tauri::command]
pub fn billing_daily_summary(
    state: State<'_, AppDatabase>,
    from: String,
    to: String,
) -> Result<DailySummary, CommandError> {
    state.with(|conn| accounts::daily_summary(conn, &from, &to))
}
#[tauri::command]
pub fn billing_charge_sources(
    state: State<'_, AppDatabase>,
) -> Result<Vec<ChargeSource>, CommandError> {
    state.with(accounts::charge_sources)
}
