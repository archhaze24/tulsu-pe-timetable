// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: при первом запуске создается конфиг + бд
    // 1. конфиг: крейт config???
    // конфиг хранится по умолчанию в дефолтных папках ос (винда: appdata, линукс: .local/share, мак: library
    // 2. бд: sqlite, крейт sqlx
    // проверяется наличие бд, если нет то создается и мигрируется
    // в будущем при обновлении приложения новые миграции применяются автоматически!!!
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri_collect_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
