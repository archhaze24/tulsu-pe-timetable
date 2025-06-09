mod api;

use tauri_helper::tauri_collect_commands;
use crate::api::handlers::*;
 
// мы не компилируемся для мобилок, но на всякий случай
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri_collect_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
