use tauri_helper::auto_collect_command;
use crate::api::response::ApiResponse;

#[tauri::command]
#[auto_collect_command]
pub fn greet(name: String) -> ApiResponse<String> {
    if name.trim().is_empty() {
        return ApiResponse::err("Name is required");
    }

    ApiResponse::ok(format!("Hello, {}!", name))
}
