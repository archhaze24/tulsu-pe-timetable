use tauri::State;
use tauri_helper::auto_collect_command;
use crate::api::response::ApiResponse;
use crate::db::Db;
use crate::db::queries::count::increment_count;
use crate::wrap;

#[tauri::command]
#[auto_collect_command]
pub async fn count(id: i64, db: State<'_, Db>) -> Result<ApiResponse<i64>, String> {
   wrap!(increment_count(id, &db.pool).await)
}
