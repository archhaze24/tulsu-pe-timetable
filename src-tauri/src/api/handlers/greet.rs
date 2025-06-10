use tauri_helper::auto_collect_command;
use crate::api::response::ApiResponse;
use crate::wrap;

#[tauri::command]
#[auto_collect_command]
pub async fn greet(name: String) -> Result<ApiResponse<String>, String> {
    if name.trim().is_empty() {
        return wrap!(Err("Name is required"));
    }

    Ok(ApiResponse::ok(format!("Hello, {}!", name)))
}
