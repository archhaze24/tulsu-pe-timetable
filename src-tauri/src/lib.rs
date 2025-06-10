mod api;
mod db;
mod config;

#[macro_use]
mod utils;

use crate::api::handlers::*;
use tauri_helper::tauri_collect_commands;
use crate::config::create::get_or_create_config;
use crate::db::Db;

pub async fn run() -> anyhow::Result<()> {
    let config = get_or_create_config()?;
    let db = Db::new(config.db_path.as_str()).await.expect("DB init failed");

    tauri::Builder::default()
        .manage(db)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri_collect_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
