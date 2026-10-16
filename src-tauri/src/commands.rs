//! Команды Tauri. Имена и форма ответа повторяют методы Go-версии (`backend/app_services/app.go`):
//! каждая команда возвращает `ApiResponse { data, error }`, ошибки уходят в `error`, а не в reject.

use serde::{Deserialize, Serialize};

use tauri::State;
use ts_rs::TS;

use crate::error::{AppError, AppResult};
use crate::models::{Direction, Faculty, Semester, Teacher};
use crate::storage::repo;
use crate::{config, export, logger, AppState};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: String,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { data: Some(data), error: String::new() }
    }

    fn err(error: AppError) -> Self {
        Self { data: None, error: error.message().to_owned() }
    }
}

/// Преобразует результат операции в `ApiResponse`, пишет ошибку в журнал.
fn respond<T: Serialize>(result: AppResult<T>) -> ApiResponse<T> {
    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => {
            logger::error("api", e.message());
            ApiResponse::err(e)
        }
    }
}

/// Для команд без полезной нагрузки: `data: true` при успехе, как в Go.
fn respond_flag(result: AppResult<()>) -> ApiResponse<bool> {
    match result {
        Ok(()) => ApiResponse::ok(true),
        Err(e) => {
            logger::error("api", e.message());
            ApiResponse::err(e)
        }
    }
}

// ---------- Конфиг ----------

#[tauri::command]
pub fn get_config(state: State<AppState>) -> ApiResponse<config::Config> {
    respond(state.config.lock().map(|c| c.clone()).map_err(|_| AppError::key("errors.config.read_failed")))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    pub db_path: String,
    pub theme: String,
}

#[tauri::command]
pub fn update_config(state: State<AppState>, config: UpdateConfigRequest) -> ApiResponse<bool> {
    let next = config::Config { db_path: config.db_path, theme: config.theme };
    let result = config::save(&next).map(|_| {
        if let Ok(mut current) = state.config.lock() {
            // Соединение с базой открыто при старте, поэтому новый путь применится после перезапуска.
            if current.db_path != next.db_path {
                logger::warn("config", format!("путь к базе изменён на {}: применится после перезапуска", next.db_path));
            }
            logger::info("config", format!("настройки сохранены: тема {}", next.theme));
            *current = next;
        }
    });
    respond_flag(result)
}


// ---------- Направления ----------

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct NameRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct UpdateNameRequest {
    pub id: i64,
    pub name: String,
}

#[tauri::command]
pub fn create_direction(state: State<AppState>, req: NameRequest) -> ApiResponse<Direction> {
    let conn = lock(&state);
    respond(repo::create_direction(&conn, &req.name))
}

#[tauri::command]
pub fn get_directions(state: State<AppState>) -> ApiResponse<Vec<Direction>> {
    get_directions_by_archived(state, false)
}

#[tauri::command]
pub fn get_directions_by_archived(state: State<AppState>, is_archived: bool) -> ApiResponse<Vec<Direction>> {
    let conn = lock(&state);
    respond(repo::list_directions(&conn, is_archived))
}

#[tauri::command]
pub fn update_direction(state: State<AppState>, req: UpdateNameRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::update_direction(&conn, req.id, &req.name))
}

#[tauri::command]
pub fn delete_direction(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::archive_direction(&conn, id))
}

#[tauri::command]
pub fn restore_direction(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::restore_direction(&conn, id))
}

// ---------- Факультеты ----------

#[tauri::command]
pub fn create_faculty(state: State<AppState>, req: NameRequest) -> ApiResponse<Faculty> {
    let conn = lock(&state);
    respond(repo::create_faculty(&conn, &req.name))
}

#[tauri::command]
pub fn get_faculties(state: State<AppState>) -> ApiResponse<Vec<Faculty>> {
    get_faculties_by_archived(state, false)
}

#[tauri::command]
pub fn get_faculties_by_archived(state: State<AppState>, is_archived: bool) -> ApiResponse<Vec<Faculty>> {
    let conn = lock(&state);
    respond(repo::list_faculties(&conn, is_archived))
}

#[tauri::command]
pub fn update_faculty(state: State<AppState>, req: UpdateNameRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::update_faculty(&conn, req.id, &req.name))
}

#[tauri::command]
pub fn delete_faculty(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::archive_faculty(&conn, id))
}

#[tauri::command]
pub fn restore_faculty(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::restore_faculty(&conn, id))
}

// ---------- Преподаватели ----------

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct TeacherRequest {
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub direction_id: i64,
    pub rate: f64,
}

impl TeacherRequest {
    fn into_teacher(self) -> Teacher {
        Teacher {
            id: self.id.unwrap_or(0),
            first_name: self.first_name,
            last_name: self.last_name,
            middle_name: self.middle_name,
            direction_id: self.direction_id,
            rate: self.rate,
            direction_name: String::new(),
            is_archived: false,
            is_guest: false,
            is_bound: false,
        }
    }
}

#[tauri::command]
pub fn create_teacher(state: State<AppState>, req: TeacherRequest) -> ApiResponse<Teacher> {
    let conn = lock(&state);
    respond(repo::create_teacher(&conn, &req.into_teacher()))
}

#[tauri::command]
pub fn get_teachers(state: State<AppState>) -> ApiResponse<Vec<Teacher>> {
    get_teachers_by_archived(state, false)
}

#[tauri::command]
pub fn get_teachers_by_archived(state: State<AppState>, is_archived: bool) -> ApiResponse<Vec<Teacher>> {
    let conn = lock(&state);
    respond(repo::list_teachers(&conn, is_archived))
}

#[tauri::command]
pub fn update_teacher(state: State<AppState>, req: TeacherRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::update_teacher(&conn, &req.into_teacher()))
}

#[tauri::command]
pub fn delete_teacher(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::archive_teacher(&conn, id))
}

#[tauri::command]
pub fn restore_teacher(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::restore_teacher(&conn, id))
}

// ---------- Семестры ----------

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct SemesterRequest {
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<i64>,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

#[tauri::command]
pub fn get_semesters(state: State<AppState>) -> ApiResponse<Vec<Semester>> {
    get_semesters_by_archived(state, false)
}

#[tauri::command]
pub fn get_semesters_by_archived(state: State<AppState>, is_archived: bool) -> ApiResponse<Vec<Semester>> {
    let conn = lock(&state);
    respond(repo::list_semesters(&conn, is_archived))
}

#[tauri::command]
pub fn create_semester(state: State<AppState>, req: SemesterRequest) -> ApiResponse<Semester> {
    let conn = lock(&state);
    respond(repo::create_semester(&conn, &req.name, &req.start_date, &req.end_date))
}

#[tauri::command]
pub fn update_semester(state: State<AppState>, req: SemesterRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    let id = req.id.unwrap_or(0);
    respond_flag(repo::update_semester(&conn, id, &req.name, &req.start_date, &req.end_date))
}


#[tauri::command]
pub fn delete_semester(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::archive_semester(&conn, id))
}

#[tauri::command]
pub fn restore_semester(state: State<AppState>, id: i64) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::restore_semester(&conn, id))
}

#[tauri::command]
pub fn get_all_teachers_for_semester(state: State<AppState>, semester_id: i64) -> ApiResponse<Vec<Teacher>> {
    let conn = lock(&state);
    respond(repo::teachers_for_semester(&conn, semester_id))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct BindRequest {
    pub semester_id: i64,
    pub teacher_id: i64,
}

#[tauri::command]
pub fn bind_teacher_to_semester(state: State<AppState>, req: BindRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::bind_teacher(&conn, req.semester_id, req.teacher_id))
}

#[tauri::command]
pub fn unbind_teacher_from_semester(state: State<AppState>, req: BindRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(repo::unbind_teacher(&conn, req.semester_id, req.teacher_id))
}


// ---------- Расписание ----------

#[tauri::command]
pub fn get_schedule(state: State<AppState>, semester_id: i64) -> ApiResponse<crate::schedule::ScheduleView> {
    let conn = lock(&state);
    respond(crate::schedule::build_view(&conn, semester_id))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct ToggleTeacherRequest {
    pub semester_id: i64,
    pub day: i64,
    pub slot: String,
    pub teacher_id: i64,
}

#[tauri::command]
pub fn toggle_teacher_in_slot(state: State<AppState>, req: ToggleTeacherRequest) -> ApiResponse<crate::schedule::ScheduleOutcome> {
    let conn = lock(&state);
    respond(crate::schedule::toggle_teacher(&conn, req.semester_id, req.day, &req.slot, req.teacher_id))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct SlotFacultiesRequest {
    pub semester_id: i64,
    pub day: i64,
    pub slot: String,
    pub faculty_ids: Vec<i64>,
}

#[tauri::command]
pub fn set_slot_faculties(state: State<AppState>, req: SlotFacultiesRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(crate::schedule::set_slot_faculties(&conn, req.semester_id, req.day, &req.slot, &req.faculty_ids))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct AddSlotRequest {
    pub semester_id: i64,
    pub day: i64,
    pub start_time: String,
    pub end_time: String,
    pub teacher_count: i64,
}

#[tauri::command]
pub fn add_slot(state: State<AppState>, req: AddSlotRequest) -> ApiResponse<crate::schedule::ScheduleOutcome> {
    let conn = lock(&state);
    respond(crate::schedule::add_slot(&conn, req.semester_id, req.day, &req.start_time, &req.end_time, req.teacher_count))
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct DeleteSlotRequest {
    pub semester_id: i64,
    pub day: i64,
    pub slot: String,
}

#[tauri::command]
pub fn delete_slot(state: State<AppState>, req: DeleteSlotRequest) -> ApiResponse<bool> {
    let conn = lock(&state);
    respond_flag(crate::schedule::delete_slot(&conn, req.semester_id, req.day, &req.slot))
}

// ---------- Экспорт ----------

/// Имя файла для диалога сохранения. Выбор пути делает фронтенд через плагин dialog.
#[tauri::command]
pub fn get_export_file_name(state: State<AppState>, semester_id: i64) -> ApiResponse<String> {
    let conn = lock(&state);
    respond(repo::get_semester(&conn, semester_id).map(|s| export::default_file_name(&s)))
}

/// Формирует XLSX расписания семестра и сохраняет его по пути, выбранному в диалоге.
#[tauri::command]
pub fn export_schedule_xlsx(state: State<AppState>, semester_id: i64, path: String) -> ApiResponse<String> {
    let conn = lock(&state);
    let result = (|| -> AppResult<String> {
        let semester = repo::get_semester(&conn, semester_id)?;
        let lessons = repo::lessons_for_semester(&conn, semester_id)?;
        let teachers: Vec<Teacher> = repo::teachers_for_semester(&conn, semester_id)?
            .into_iter()
            .filter(|t| t.is_bound)
            .collect();
        export::write_schedule(std::path::Path::new(&path), &semester, &lessons, &teachers)?;
        Ok(path.clone())
    })();
    respond(result)
}

// ---------- Вспомогательное ----------

fn lock<'a>(state: &'a State<AppState>) -> std::sync::MutexGuard<'a, rusqlite::Connection> {
    state.conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
