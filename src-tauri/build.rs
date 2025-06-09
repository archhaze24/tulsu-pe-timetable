fn main() {
    // генерируем файл с макросом tauri_collect_commands!()
    tauri_helper::generate_command_file(Default::default());

    tauri_build::build();
}
