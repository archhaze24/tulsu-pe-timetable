mod archive;
mod commands;
mod config;
mod error;
mod export;
mod logger;
mod models;
mod schedule;
mod startup;
mod storage;

use std::sync::Mutex;

use rusqlite::Connection;

/// Общее состояние приложения: соединение с БД и текущая конфигурация.
/// Регистрируется только после успешного запуска (см. `startup`).
pub struct AppState {
    pub conn: Mutex<Connection>,
    pub config: Mutex<config::Config>,
}

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            startup::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_direction,
            commands::get_directions,
            commands::get_directions_by_archived,
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
            commands::preview_unbind_teacher,
            commands::unbind_teacher_from_semester,
            commands::get_schedule,
            commands::toggle_teacher_in_slot,
            commands::set_slot_faculties,
            commands::add_slot,
            commands::set_slot_teacher_count,
            commands::clear_slot_assignments,
            commands::delete_slot,
            commands::get_export_file_name,
            commands::export_schedule_xlsx,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        logger::error("app", format!("ошибка работы приложения: {e}"));
        std::process::exit(1);
    }
}
