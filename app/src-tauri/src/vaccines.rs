use crate::{AppDatabase, CommandError};
use baytar_core::{
    model::{Appointment, Vaccination},
    repo::vaccinations::{
        self, GiveVaccination, PlanVaccination, VaccineCalendarEvent, VaccineDue, VaccineDueQuery,
    },
};
use tauri::State;

#[tauri::command]
pub fn vaccines_due_list(
    state: State<'_, AppDatabase>,
    query: VaccineDueQuery,
) -> Result<Vec<VaccineDue>, CommandError> {
    state.with(|conn| vaccinations::merged_due(conn, query))
}
#[tauri::command]
pub fn vaccines_give(
    state: State<'_, AppDatabase>,
    input: GiveVaccination,
) -> Result<Vaccination, CommandError> {
    state.with(|conn| vaccinations::give(conn, input))
}
#[tauri::command]
pub fn vaccines_plan(
    state: State<'_, AppDatabase>,
    input: PlanVaccination,
) -> Result<Appointment, CommandError> {
    state.with(|conn| vaccinations::plan(conn, input))
}
#[tauri::command]
pub fn vaccines_suggest(
    state: State<'_, AppDatabase>,
    patient_id: i64,
    type_id: i64,
    given_on: String,
) -> Result<Option<String>, CommandError> {
    state.with(|conn| vaccinations::suggest_for_patient(conn, patient_id, type_id, &given_on))
}
#[tauri::command]
pub fn vaccines_calendar(
    state: State<'_, AppDatabase>,
    from: String,
    to: String,
) -> Result<Vec<VaccineCalendarEvent>, CommandError> {
    state.with(|conn| vaccinations::calendar(conn, &from, &to))
}
#[tauri::command]
pub fn vaccines_postpone(
    state: State<'_, AppDatabase>,
    appointment_id: Option<i64>,
    vaccination_id: Option<i64>,
    on: String,
) -> Result<Appointment, CommandError> {
    state.with(|conn| vaccinations::postpone(conn, appointment_id, vaccination_id, &on))
}
