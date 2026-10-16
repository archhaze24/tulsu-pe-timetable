mod commands;
mod config;
mod error;
mod export;
mod locales;
mod logger;
mod models;
mod schedule;
mod storage;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

/// Общее состояние приложения: соединение с БД и текущая конфигурация.
pub struct AppState {
    pub conn: Mutex<Connection>,
    pub config: Mutex<config::Config>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config = config::load().map_err(|e| e.to_string())?;
            let db_path = std::path::Path::new(&config.db_path);
            logger::init(db_path);
            let conn = storage::open(db_path).map_err(|e| e.to_string())?;
            app.manage(AppState { conn: Mutex::new(conn), config: Mutex::new(config) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![

            commands::get_config,
            commands::update_config,

            commands::create_direction,
            commands::get_directions,
            commands::get_directions_by_archived,
            commands::update_direction,
            commands::delete_direction,
            commands::restore_direction,
            commands::create_faculty,
            commands::get_faculties,
            commands::get_faculties_by_archived,
            commands::update_faculty,
            commands::delete_faculty,
            commands::restore_faculty,
            commands::create_teacher,
            commands::get_teachers,
            commands::get_teachers_by_archived,
            commands::update_teacher,
            commands::delete_teacher,
            commands::restore_teacher,

            commands::get_semesters,
            commands::get_semesters_by_archived,
            commands::create_semester,
            commands::update_semester,

            commands::delete_semester,
            commands::restore_semester,

            commands::get_all_teachers_for_semester,
            commands::bind_teacher_to_semester,
            commands::unbind_teacher_from_semester,
            commands::get_schedule,
            commands::toggle_teacher_in_slot,
            commands::set_slot_faculties,
            commands::add_slot,
            commands::delete_slot,
            commands::get_export_file_name,
            commands::export_schedule_xlsx,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка запуска приложения");
}
