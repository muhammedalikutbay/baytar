#[tauri::command]
fn app_version() -> &'static str {
    baytar_core::version()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!(
        "Starting {} {}",
        baytar_core::APP_NAME,
        baytar_core::version()
    );

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .expect("error while running Baytar");
}
