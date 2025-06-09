use tauri_helper::auto_collect_command;
use crate::api::response::ApiResponse;

#[tauri::command]
#[auto_collect_command]
pub fn count(offset: i32) -> ApiResponse<i32> {
    ApiResponse::ok(offset + 1)
}
