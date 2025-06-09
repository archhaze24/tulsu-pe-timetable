fn main() {
    // генерируем временный файл с вызовом generate_handler![]
    tauri_helper::generate_command_file(Default::default());
    
    // стандартная сборка Tauri
    tauri_build::build();
}