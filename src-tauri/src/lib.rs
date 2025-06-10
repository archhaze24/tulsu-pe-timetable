mod api;
mod db;
mod config;

#[macro_use]
mod utils;

use crate::api::handlers::*;
use tauri_helper::tauri_collect_commands;
use crate::db::Db;

pub async fn run() -> anyhow::Result<()> {
    let db = Db::new("test.db").await.expect("DB init failed");

    tauri::Builder::default()
        .manage(db)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri_collect_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
