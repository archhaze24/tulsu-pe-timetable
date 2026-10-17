# vendor

## wry 0.57.0 с патчем для Windows 7

`vendor/wry` — исходники `wry` 0.57.0 из crates.io (лицензия `Apache-2.0 OR MIT`, файлы `LICENSE-*` внутри).
Подключён через `[patch.crates-io]` в `src-tauri/Cargo.toml`. Нужен, чтобы Tauri создавал WebView2 на Windows 7.
Почему и как: `docs/windows7.md`, раздел 4.2.

Изменения относятся только к Windows 7 (`cfg(target_vendor = "win7")`). Сборки для Windows 10/11 и macOS работают как upstream:

- `src/webview2/fixed_runtime.rs` (новый файл): environment через `CreateWebViewEnvironmentWithOptionsInternal` из `EBWebView\x64\EmbeddedBrowserWebView.dll`, версия из `VS_FIXEDFILEINFO` этой DLL. Папка берётся из `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`, которую выставляет `src-tauri/src/win7_compat.rs`.
- `src/webview2/mod.rs`: `create_environment` (options с `TargetCompatibleBrowserVersion` = `86.0.616.0`) и `platform_webview_version` используют `fixed_runtime`, если папка задана. Без папки — путь upstream.
- `Cargo.toml`: фича `Win32_Storage_FileSystem` для `windows` (функции версии файла).

Из копии удалены служебные файлы cargo (`.cargo`, `Cargo.lock`, `Cargo.toml.orig`, `.cargo-ok`, `.cargo_vcs_info.json`, `renovate.json`, `.gitignore`).

### Обновление wry

1. Скопировать новую версию из crates.io в `vendor/wry` (без служебных файлов).
2. Перенести изменения из списка выше (поиск по `target_vendor = "win7"` и `fixed_runtime`).
3. Проверить `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-win7-windows-msvc` (см. `docs/windows7.md`, раздел 8).
