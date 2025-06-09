mod api;
mod db;

use crate::api::handlers::*;
use crate::db::init::init_db;
use tauri_helper::tauri_collect_commands;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db("test.db").await?; // todo: пока что хардкод, поменять под конфиг

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri_collect_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
