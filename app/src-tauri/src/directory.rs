use crate::{AppDatabase, CommandError};
use baytar_core::repo::search::{self, Directory};
use tauri::State;

#[tauri::command]
pub fn directory_list(
    state: State<'_, AppDatabase>,
    query: String,
    limit: u32,
) -> Result<Directory, CommandError> {
    state.with(|conn| search::directory(conn, &query, limit.min(1000)))
}
