fn main() {
    tauri_build::build();

    // Windows 7: эти импорты есть только в Windows 8+, загрузчик упал бы на старте.
    // Грузим их лениво, а подмену делает src/win7_compat.rs.
    if std::env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("win7") {
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
        for dll in ["advapi32.dll", "combase.dll", "api-ms-win-core-winrt-error-l1-1-0.dll"] {
            println!("cargo:rustc-link-arg-bins=/DELAYLOAD:{dll}");
        }
    }
}
