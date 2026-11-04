use crate::{AppDatabase, CommandError};
use baytar_core::repo::reminders::{
    self, NewReminderContact, OverviewQuery, ReminderContact, RemindersOverview,
};
use tauri::State;

/// Everything the reminders centre lists, derived on demand; `query.today` is the local day.
#[tauri::command]
pub fn reminders_overview(
    state: State<'_, AppDatabase>,
    query: OverviewQuery,
) -> Result<RemindersOverview, CommandError> {
    state.with(|conn| reminders::overview(conn, query))
}

/// Marks a reminder as contacted (WhatsApp opened, phone call, or another channel).
#[tauri::command]
pub fn reminders_record_contact(
    state: State<'_, AppDatabase>,
    input: NewReminderContact,
) -> Result<ReminderContact, CommandError> {
    state.with(|conn| reminders::record_contact(conn, input))
}
