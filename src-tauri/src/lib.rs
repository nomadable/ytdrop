mod commands;
mod db;
mod model;
mod parse;
mod queue;
mod settings;
mod ytdlp;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            let db = db::open(&app_dir)?;
            app.manage(db);
            app.manage(queue::QueueState {
                notify: Arc::new(Notify::new()),
            });
            queue::spawn_worker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_download,
            commands::retry_download,
            commands::remove_from_queue,
            commands::clear_history,
            commands::list_downloads,
            commands::get_settings,
            commands::set_download_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
