# Windows 7: статус, контекст и грабли

Файл для агентов и разработчиков. Здесь всё про сборку под Windows 7: что сделано, почему, что проверено и что нет. Перед любыми изменениями Win7-сборки читать целиком.

Обновлено: 2026-10-17 (патч `wry` для Win7 написан и собран локально для `x86_64-win7-windows-msvc`, на VM не проверен; см. разделы 4.2, 5, 7). Если что-то устарело, сверьте с `git log`, `gh run list` и исправьте этот файл.

## 1. Цель и контекст

- Нужно: Tauri-версия работает на ПК кафедры физкультуры с Windows 7 SP1. Обычная сборка (Windows 10/11) — `.github/workflows/windows.yml`.
- Рабочая Go-версия для Win7: ветка `legacy-go-version-win7` (Wails 2.7.1, go-webview2 v1.0.10). Работает на Win7-ПК с WebView2 109 в `C:\WebView2`. Пока Tauri на Win7 не заработает, её не удаляем.
- Файл `tulsu-pe-timetable.exe` в ветке `legacy-go-version-win7` собран Go 1.25.0 и требует `bcryptprimitives.dll` (Windows 8+). Это не Win7-сборка, для тестов не годится.
- На Win7 максимум WebView2 109 (Chromium 109). Фронтенд для win7 собирается с `target: chrome109`.
- Тестовая среда: VirtualBox VM `vboxuser`, Windows 7 SP1 (6.1.7601), английская локализация, из обновлений только KB2534111 и KB976902. Рабочий ПК (русская Win7 SP1) сейчас недоступен.
- Репозиторий приватный: `archhaze24/tulsu-pe-timetable`. Пользователь работает на macOS. Правило проекта: не коммитить и не пушить без явной просьбы.

## 2. Текущий статус (главное)

- Pure-путь (как в Go-версии) работает на VM в тесте `test-env` v3: environment создаётся с options (`TargetCompatibleBrowserVersion` = `86.0.616.0`) и без нативного loader, controller создаётся, версия `109.0.1518.78`. Подробности: раздел 5.
- Импорты exe проходят загрузчик Windows 7. Подтверждено прогоном `38975552911` (`main@f47b4a9`, PR #78 смёржен) и скриптом `scripts/win7-imports.py`.
- На VM приложение запускается, но показывает диалог `Could not find the WebView2 Runtime` и после OK закрывается. Диалог создаёт `tauri-runtime-wry` 2.12.2 (`src/lib.rs:2783`: `webview_runtime_installed: wry::webview_version().is_ok()`, показ диалога `src/lib.rs:4678`).
- Нативный loader `WebView2Loader` вшит в exe статически: `webview2-com-sys` 0.39.1 на msvc линкует `WebView2LoaderStatic.lib`. На Win7 VM его `GetAvailableCoreWebView2BrowserVersionString` даёт `0x80070032` для любой папки и `0x80070002` для `NULL`.
  - Это не путь, не версия загрузчика и не AppModel-API: shim `api-ms-win-appmodel-runtime-l1-1-2.dll` не помог (`result-shim.txt`).
  - Причина нативного `0x80070032` не найдена. Platform Update KB2670838 её не закрывает (раздел 5).
- Go-версия работает на Win7-ПК, потому что не использует нативный loader. `go-webview2` v1.0.10 без тега `native_webview2loader` (это умолчание) идёт pure-Go путём (`webviewloader/env_create.go`):
  1. `LoadLibrary("<папка>\EBWebView\x64\EmbeddedBrowserWebView.dll")`;
  2. `GetProcAddress("CreateWebViewEnvironmentWithOptionsInternal")` и вызов `(1, 1, userDataFolder, options, handler)`. Второй параметр `1` = redistributable (фиксированная версия);
  3. версия = `ProductVersion` файла client DLL (`findEmbeddedBrowserVersion`). Нативный `GetAvailable` не вызывается;
    4. options-объект обязателен: `ICoreWebView2EnvironmentOptions` (IID `{2fde08a8-1e9a-4766-8c05-95a9ceb9d1c5}`) и `ICoreWebView2EnvironmentOptions2` (IID `{ff85c98a-1ba7-4a6b-90c8-2b752c89e9e2}`). `TargetCompatibleBrowserVersion` = `86.0.616.0`, если пусто (`kMinimumCompatibleVersion`). Перед вызовом все `WEBVIEW2_*` из `preventEnvAndRegistryOverrides` пустые, в том числе `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`.
- Upstream `wry` 0.57.0 идёт только нативным путём: `create_environment` вызывает `CreateCoreWebView2EnvironmentWithOptions(NULL, …)`, проверка вызывает `GetAvailable(NULL)`. Поэтому без патча Tauri на Win7 не запустится, даже если `EmbeddedBrowserWebView.dll` грузится. Патч: `src-tauri/vendor/wry` (раздел 4.2).
- Вероятная причина `0x80070032` (гипотеза, не проверена отдельно: нужен тест нативного пути с `145.0.3800.47` и с `86.0.616.0`; фикс от неё не зависит): `webview2-com` 0.39.1 по умолчанию передаёт `TargetCompatibleBrowserVersion` = `145.0.3800.47` (`CORE_WEBVIEW_TARGET_PRODUCT_VERSION`). Runtime 109 старее, и загрузчик отвечает `ERROR_NOT_SUPPORTED`. Go передаёт `86.0.616.0`, поэтому ей версия 109 подходит.
- `main@f47b4a9` выбирал папку (`<exe>\WebView2`, затем `C:\WebView2`) через нативный `GetAvailable`: на VM оба варианта дают `0x80070032`. Текущая правка (не закоммичена) выбирает папку через `LoadLibraryW` + `GetProcAddress`, а environment создаёт патченный `wry` (раздел 4.2). Лог: `%USERPROFILE%\.tulsu-pe-timetable\webview2.log` (туда же пишется паника).

## 3. Импорты exe (решено)

| Импорт (до фиксов) | Что сделано |
|---|---|
| `advapi32.dll!EventSetInformation` (ETW, Windows 8+), из статического `WebView2LoaderStatic.lib` | `advapi32.dll` в `/DELAYLOAD`, заглушка возвращает 0 |
| `combase.dll`: `CoCreateFreeThreadedMarshaler`, `CoTaskMemFree`, `CoTaskMemAlloc` | `combase.dll` в `/DELAYLOAD`, подмена на `ole32.dll` |
| `api-ms-win-core-winrt-error-l1-1-0.dll!RoOriginateErrorW` | DLL в `/DELAYLOAD`, подмена на `kernel32.dll`, заглушка возвращает FALSE |
| `api-ms-win-crt-*` (UCRT, нужен KB2999226) | `staticVCRuntime: false` и `+crt-static` |

В `kernel32` после этого +22 функции из статического CRT (`CompareStringW`, `GetACP`, `IsProcessorFeaturePresent`, `VirtualQuery` и др.). Все из эпохи Windows 2000/XP/Vista (по знанию API, страницы Microsoft не открывал).

Итог `scripts/win7-imports.py` на `38975552911`: в обычной таблице 11 DLL, подозрительных импортов нет. В отложенной: `advapi32`, `combase`, `api-ms-win-core-winrt-error-l1-1-0`.

## 4. Что сделано и почему

### 4.1 Фронтенд (`frontend/`)

- `vite.config.ts`: режим `vite build --mode win7` даёт `build.target: 'chrome109'` и плагин `chrome109Css`.
  - Почему: Tailwind 4 пишет `oklch()` и `color-mix(in oklab, var(--color-…) P%, transparent)`. WebView2 109 их не понимает.
  - Как: lightningcss с `targets: { chrome: 109 << 16 }` в два прохода. Если после сборки остались `oklch(` или `color-mix(... var(`, сборка падает.
  - Плейсхолдер `::placeholder` заменяется на непрозрачный вариант с `opacity:.5`.
- `package.json`: скрипт `build:win7` = `vite build --mode win7`, devDependency `lightningcss ^1.33.0`.
- Плагин держим внутри `vite.config.ts`: отдельный `.ts`-файл без расширения даёт warning Vite, с расширением нужны `allowImportingTsExtensions` и `noEmit`, что даёт warning svelte-check.

### 4.2 Конфиги и Rust (`src-tauri/`)

- `tauri.win7.conf.json`:
  - `build.beforeBuildCommand`: `pnpm --dir frontend build:win7`;
  - `build.windows.staticVCRuntime: false`;
  - `bundle.targets`: `["nsis"]`;
  - `bundle.resources`: `{ "WebView2": "WebView2" }`. Папка runtime попадает в установку как `$INSTDIR\WebView2`;
  - `bundle.windows.webviewInstallMode`: `{ "type": "skip" }`. Почему: при `fixedRuntime` Tauri (2.12.4, `app.rs`) всегда перезаписывает `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` значением `<exe>\./WebView2/`.
- `.cargo/config.toml`: `[target.'cfg(all(windows, target_env = "msvc"))'] rustflags = ["-C", "target-feature=+crt-static"]`. Почему: `cc` (sqlite) собирает C с `/MD` без него.
- `tauri.conf.json`: `bundle.icon` += `icons/icon.ico`.
- `build.rs`: при `CARGO_CFG_TARGET_VENDOR == "win7"` добавляет `delayimp.lib` и `/DELAYLOAD:` для `advapi32.dll`, `combase.dll`, `api-ms-win-core-winrt-error-l1-1-0.dll` (`cargo:rustc-link-arg-bins`).
- `src/main.rs`: `#[cfg(all(windows, target_vendor = "win7"))] mod win7_compat;` и вызов `win7_compat::prepare_webview2_runtime()` перед `run()`.
- `src/win7_compat.rs`:
  - хуки delay-load: `__pfnDliNotifyHook2` (combase → ole32, winrt-error → kernel32), `__pfnDliFailureHook2` (`EventSetInformation` → 0, `RoOriginateErrorW` → FALSE). Это работает, оставить;
  - `prepare_webview2_runtime()`: выбирает папку через `LoadLibraryW` + `GetProcAddress` (точка входа `CreateWebViewEnvironmentWithOptionsInternal`) и ставит `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. `install_panic_log()` пишет панику в `webview2.log`: в release-сборке (`windows_subsystem = "windows"`) её не видно;
  - хуки в бинарнике, а не в rlib: объекты бинарника линкуются всегда;
  - edition 2024: `#[unsafe(no_mangle)]`, `unsafe extern "system" {}`.
- `src-tauri/vendor/wry/` — патч `wry` 0.57.0 для Windows 7 (`[patch.crates-io]` в `src-tauri/Cargo.toml`). Описание: `src-tauri/vendor/README.md`.
  - `src/webview2/fixed_runtime.rs` (новый): `CreateWebViewEnvironmentWithOptionsInternal(1, 1, userData, options, handler)` из `EBWebView\x64\EmbeddedBrowserWebView.dll`. Папка берётся из `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` один раз, перед вызовом очищаются `WEBVIEW2_*` (как в `test-env.c`). Версия берётся из `VS_FIXEDFILEINFO` (для локальной DLL: `109.0.1518.78`).
  - `src/webview2/mod.rs`: `create_environment` ставит `TargetCompatibleBrowserVersion` = `86.0.616.0` и вызывает `create_environment_with_options`; `platform_webview_version` берёт версию из `fixed_runtime`. Без папки всё как upstream.
  - `Cargo.toml`: фича `Win32_Storage_FileSystem`; `[lints.rust]` глушит `deprecated` и `unused_unsafe`, в `check-cfg` добавлен `macos_12_unavailable` (раздел 6, п. 24).
  - Почему патч, а не настройка: loader вшит статически (раздел 6, п. 17).
- `.gitignore`: `src-tauri/WebView2` (444 МБ, `msedge.dll` 229 МБ > лимита GitHub), `WebView2-*.zip`.

### 4.3 Скрипты (`scripts/`)

- `build-win7.mjs`: проверяет `src-tauri/WebView2/msedgewebview2.exe` и запускает `pnpm tauri build --config src-tauri/tauri.win7.conf.json` с `RUSTUP_TOOLCHAIN=${WIN7_TOOLCHAIN ?? 'stable'}`, `RUSTC_BOOTSTRAP=1`, `CARGO_UNSTABLE_BUILD_STD=std`, `CARGO_BUILD_TARGET=x86_64-win7-windows-msvc`.
- `win7-imports.py`: разбор таблиц импорта PE (обычная и отложенная), запускать как `python3 -B`.
- `scripts/win7/` (тесты для Windows 7 без сборки Tauri, в репозитории):
  - `test-env.c` (v3): создание environment через `EmbeddedBrowserWebView.dll`, повторяет pure-Go loader: options-объект, пустые `WEBVIEW2_*`, лог после каждого шага, vectored exception handler (код, адрес, смещение в модуле). Сборка и запуск в разделе 8. Главный тест текущего этапа;
  - `check-runtime.ps1`: какие функции из импортов runtime-DLL отсутствуют в системе;
  - `test-loader2.ps1`: нативный `GetAvailable` для старого загрузчика и наличие API-наборов. Ожидает `C:\test\WebView2Loader.dll` (старый загрузчик из `go-webview2` v1.0.10, файл `webviewloader/x64/WebView2Loader.dll`);
  - `test-shim.ps1`: тест shim `api-ms-win-appmodel-runtime-l1-1-2.dll` (отрицательный результат).

### 4.4 CI (`.github/workflows/`)

- `win7-installer.yml` (`main`): `workflow_dispatch`, `windows-latest`, Rust `1.99.1` с `rust-src`, Node 24, pnpm 12.6.0. Шаг «Скачать WebView2» из релиза `webview2-109`. Артефакт `tulsu-pe-timetable-win7`, хранится 7 дней.
- `windows.yml` (`main`): `check` на PR, push в `main` и теги (`pnpm check`, прошёл на PR #77 и #78). `build` и `release` на теги `v*` ещё не запускались.

### 4.5 Релиз WebView2

- Релиз `webview2-109`, ассет `WebView2-109.zip` (207 755 364 байт), `msedgewebview2.exe` в корне архива. Версия 109.0.1518.78.
- Внутри: `EBWebView/x64/EmbeddedBrowserWebView.dll` (4.8 МБ, его проверяет загрузчик), `msedge.dll` (229 МБ), `msedgewebview2.exe`.
- Локальная копия для анализа: `src-tauri/WebView2/` (в git не попадает).

## 5. Проверки: что подтверждено

- CI: `38970975453` (импорты не проходили загрузчик), `38975552911` (`main@f47b4a9`): успех, импорты проходят. Установщик содержит `WebView2/msedgewebview2.exe` и `WebView2/EBWebView/x64/EmbeddedBrowserWebView.dll`.
- Лог с VM (`webview2 (2).log`): `<exe>\WebView2` → `hr=0x80070032`, `C:\WebView2` → `hr=0x80070032`, итоговая проверка Tauri (`env=None`) → `hr=0x80070002`.
- `result.txt` (старый загрузчик, PowerShell): `C:\WebView2` → `0x80070032`, `NULL` → `0x80070002`.
- `check-vm.txt`: runtime-DLL загружается, отсутствуют 0 из 207 функций `kernel32` и 0 из 13 `oleaut32`.
- `result2-vm.txt`: отсутствуют API-наборы `api-ms-win-core-version-l1-1-0`, `api-ms-win-core-synch-l1-2-0`, `api-ms-win-appmodel-runtime-l1-1-2` (ошибка 126), а также `kernelbase.dll!GetCurrentApplicationUserModelId` и `GetCurrentPackageFullName`. `version.dll!GetFileVersionInfoExW` есть.
- `result-shim.txt`: shim загружен, `GetAvailable(C:\WebView2)` → `0x80070032`, `GetAvailable(NULL)` → `0x80070002`. Гипотеза AppModel отвергнута.
- Сравнение загрузчиков: версии файлов 1.0.3800.47 (`webview2-com-sys` 0.39.1) и 1.0.992.28 (`go-webview2` 1.0.10). Строки одинаковые, кроме новых в 1.0.3800 (ETW, каналы `ChannelSearchKind`, `ReleaseChannels`).
- Импорты `WebView2Loader.dll` (`webview2-com-sys` 0.39.1, x64): eager-импорт только `KERNEL32`. Остальное грузится динамически (строки): `kernelbase.dll`, `version.dll`, `api-ms-win-core-version-l1-1-0`, `api-ms-win-appmodel-runtime-l1-1-2`, `api-ms-win-core-synch-l1-2-0`, `ole32`, `shell32`. Есть `OutputDebugString` и сообщения `WebView2: Failed to find …` (их можно поймать DebugView на VM).
- Импорты runtime (`src-tauri/WebView2`, `objdump -p`): `msedgewebview2.exe` → `KERNEL32`, `msedge_elf.dll`; `msedge.dll` → `KERNEL32`, `OLEAUT32`, `WINMM`, `msedge_elf.dll`; `EmbeddedBrowserWebView.dll` → `KERNEL32`, `OLEAUT32`; `msedge_elf.dll` → `KERNEL32`. Прямых `api-ms-win-*` и UCRT нет.
- Экспорты `EmbeddedBrowserWebView.dll`: `CreateWebViewEnvironmentWithOptionsInternal`, `DllCanUnloadNow`.
- v2 (`env-result.txt`, Win7 VM): `CoInitializeEx`, `LoadLibraryW`, `GetProcAddress` OK. `CreateWebViewEnvironmentWithOptionsInternal` → `0x00000000`, `environment completion hr: 0x00000000`, `QueryInterface(ICoreWebView2Environment)` → `0x00000000`. Значит, environment на runtime 109 создаётся с options (`TargetCompatibleBrowserVersion` = `86.0.616.0`) и без нативного loader.
- APPCRASH v2 (`crash2.txt`): тот же RVA `0xbf80d`, `read target=0xFFFFFFFFFFFFFFFF`. Падение на `get_BrowserVersionString` (vtbl[5]) у env, то есть уже после создания. Thunk читает `[env+0x10]`, а затем `[rax+0x38]`, значит первое слово внутреннего объекта равно `0xFFFFFFFFFFFFFFC7` (невалидно). Вероятная причина: env освобождён. Тест v2 не делал AddRef внутри callback и вызвал QueryInterface уже после возврата из callback. По правилам COM объект из callback живёт только до возврата. Go делает QueryInterface внутри callback. Не проверено, но согласуется с данными.
- APPCRASH `test-env` v1 (`crash.txt`): `EmbeddedBrowserWebView.dll` 109.0.1518.78, `Exception Offset` `0xbf80d` = VA `0x1800bf80d` (ImageBase `0x180000000`, timestamp DLL совпадает с локальной копией). Thunk `0x1800bf806`: `mov 0x10(%rcx),%rcx; mov (%rcx),%rax; mov 0x38(%rax),%rax`, то есть вызов метода vtable №7 = `get_TargetCompatibleBrowserVersion` options-объекта. v1 передавал `options = NULL` и выставлял `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` в папку. Оба отличия от Go. Вывод: options обязателен, пустые env обязательны.
- Обновления Windows 7 (проверено по содержимому `.msu` на Mac):
  - KB2670838 x64 (Platform Update, GUID `0edb0e28-0c35-41ba-b52c-c8576e52c395`): содержит `api-ms-win-downlevel-*` (advapi32, normaliz, ole32, shell32, shlwapi, user32, version) и DirectX (`d2d1`, `d3d10*`, `d3d11`, `dwrite`, `dxgi`). `api-ms-win-core-*` и `kernelbase.dll` в нём нет. Значит, он не закрывает недостающие `api-ms-win-core-version/synch` и `appmodel`. Не ставить как фикс.
  - KB2533623 в каталоге Microsoft Update только для Vista x64 (`windows6.0-…`). KB2999226 в поиске только для Vista и Server 2008. Win7-вариантов не нашёл.
  - SSU KB4490628 x64 (GUID `e45cd0f7-e37e-405e-8204-ade6d492b713`) и SHA-2 KB4474419 v3 x64 (GUID `38222b3b-2bd2-4269-880d-35feddf42984`) скачаны в `/tmp/pu/` (временно, вне repo). Не проверены на VM. Ставить, только если тест покажет проблему с подписью браузера.
- Go-версия: `pkg/edge/chromium.go:136` для фиксированного пути вызывает Go-реализацию версии, а не нативный `GetAvailable`.
- `webview2-com` 0.39.1 (`src/options.rs`): `CoreWebView2EnvironmentOptions::default()` берёт `TargetCompatibleBrowserVersion` из `CORE_WEBVIEW_TARGET_PRODUCT_VERSION` = `145.0.3800.47` (`webview2-com-sys` `bindings.rs:1013`). Go-версия подставляет `86.0.616.0` (`kMinimumCompatibleVersion`).
- Локально (macOS): `rustc --emit=metadata --target x86_64-pc-windows-msvc` для `win7_compat.rs` без предупреждений, `cargo check` чистый, `pnpm check` exit 0. Тест `scripts/win7/test-env.c` собирается mingw-w64 без предупреждений, импорты только `KERNEL32`, `USER32`, `ole32`.
- v3 (`env-result.txt` и `result.txt` идентичны, Win7 VM): все шаги прошли. `CreateWebViewEnvironmentWithOptionsInternal` → `0x00000000`, `environment completion hr` → `0x00000000`, `CreateCoreWebView2Controller` → `0x00000000`, `controller completion hr` → `0x00000000`, `get_BrowserVersionString` → `0x00000000`, `browser version: 109.0.1518.78`. `Crashpad/reports` пуст, падений нет. Совпадает с ожидаемым успехом из раздела 7, п. 1.
- Локально (macOS, 2026-10-17, с патчем `wry`):
  - `pnpm check` проходит, `cargo check` без warning'ов (с `src-tauri/vendor/wry`);
  - `wry` с патчем проходит `cargo check` для `x86_64-pc-windows-msvc` и `x86_64-win7-windows-msvc` (с `build-std`, `target_vendor = "win7"` активен). Проверка через отдельную копию, раздел 8;
  - `win7_compat.rs` через `rustc --emit=metadata` для `x86_64-pc-windows-msvc`: без ошибок и предупреждений;
  - версия по `VS_FIXEDFILEINFO` локальной `src-tauri/WebView2/EBWebView/x64/EmbeddedBrowserWebView.dll`: `109.0.1518.78`.
- IID из локальной копии runtime (поиск байтов GUID в DLL): найдены `ICoreWebView2_2`, `_4`, `_10`, `_13`, `ICoreWebView2Environment9`, `ICoreWebView2Environment10`. Не найдены `ICoreWebView2_19`, `_22`, `ICoreWebView2Settings2..9`, `ICoreWebView2Controller2`, `ICoreWebView2Controller4`, `ICoreWebView2Profile7`. Отсутствие IID не доказывает, что интерфейса нет: `ICoreWebView2Controller2` старый, и runtime 109 его скорее всего знает. По коду `wry` ненайденные интерфейсы вызываются только в опциональных функциях (`set_memory_usage_level`, `load_extensions`, `clear_all_browsing_data`, цвет фона, drag-and-drop) или через `if let Ok` (`Settings*`, `_22`). Раздел 6, п. 27.
- Не проверено: сборка установщика в CI, импорты exe после патча (`scripts/win7-imports.py`), окно Tauri на VM, рабочий ПК (русская Win7 SP1).

## 6. Грабли (не повторять)

1. Nightly от 2026-10-15 не собирает std для `x86_64-win7-windows-msvc`. Работает stable 1.99.1 с `RUSTC_BOOTSTRAP=1`.
2. Tauri CLI отвергает `--target x86_64-win7-windows-msvc`. Использовать `CARGO_BUILD_TARGET`.
3. `build.windows.staticVCRuntime` по умолчанию `true` → `ucrt.lib` → `api-ms-win-crt-*`. Для win7 — `false`.
4. `cc` без `crt-static` собирает C с `/MD`. Нужен `+crt-static`.
5. Ошибка загрузчика Windows = первый отсутствующий импорт. Полный список — скриптом.
6. Git: runtime не коммитить. `msedge.dll` 229 МБ > лимита GitHub 100 МБ. Только релиз.
7. Vite: `.ts`-плагин держим в `vite.config.ts`.
8. Edition 2024: `#[unsafe(no_mangle)]`, `unsafe extern "system" {}`.
9. Хуки delay-load — в бинарнике, не в rlib.
10. При `fixedRuntime` Tauri перезаписывает `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. Свой выбор папки работает только при `skip`.
11. `tauri build --no-bundle` на macOS с `bundle.resources` копирует `WebView2` (444 МБ) в `src-tauri/target/debug/`. Удалить после проверки.
12. `0x80070032` у нативного `GetAvailable` на голой Win7: причина не найдена. Не путь, не версия загрузчика, не AppModel (shim не помог). Не путать с `0x80070002` (runtime не найден, нормально для `NULL`).
13. Go-exe из `legacy-go-version-win7` — Go 1.25.0, требует `bcryptprimitives.dll`. Не тестовая сборка для Win7 и не baseline. Baseline нужен Go 1.20 (последний с Win7).
14. `wmic qfe get HotFixID` показывает только QFE-обновления. Platform Update (KB2670838) даёт `api-ms-win-downlevel-*` и DirectX, а не `api-ms-win-core-*`. Не полагаться на него как на фикс.
15. Git: после squash-мерджа PR нельзя продолжать ту же ветку с её старой историей: появляются add/add конфликты. Новую работу делать от `main`.
16. `gh` работает без песочницы в этом треде.
17. `webview2-com-sys` на msvc линкует loader статически. Подменить `CreateCoreWebView2EnvironmentWithOptions` из отдельной DLL нельзя. Обход только через `EmbeddedBrowserWebView.dll` (патч `wry`).
18. Upstream-проверка Tauri `wry::webview_version()` идёт через `GetAvailable(NULL)` и не видит папку из `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. В патче при заданной папке версия берётся из DLL (`fixed_runtime::version`), `GetAvailable` не вызывается.
19. Shim-DLL собирать mingw-w64 с `-nostdlib` и входом `DllMain`. Иначе появляются `api-ms-win-crt-*` импорты, которых нет без KB2999226. Для exe аналогично: `-nostdlib -nostartfiles -Wl,-e,mainCRTStartup`.
20. Options-объект для `CreateWebViewEnvironmentWithOptionsInternal` обязателен. `NULL` даёт access violation в `EmbeddedBrowserWebView.dll` (APPCRASH на VM, v1).
21. Pure-путь требует пустые `WEBVIEW2_*` при вызове. Папка передаётся через `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`: патч читает её один раз и очищает переменные перед вызовом (решено в патче).
22. `TargetCompatibleBrowserVersion` больше версии runtime даёт `ERROR_NOT_SUPPORTED` (гипотеза). В `webview2-com` 0.39.1 по умолчанию это `145.0.3800.47`. В патче на Win7 с папкой ставится `86.0.616.0`.
23. COM: объект, переданный в callback (environment, controller), валиден только до возврата из callback. Нужно AddRef или QueryInterface внутри callback, иначе use-after-free. Ошибка была в `test-env` v2, исправлено в v3.
24. `[patch.crates-io]` делает `wry` path-зависимостью, и cargo показывает её предупреждения: на macOS было 17 warning'ов (`deprecated`, `unused_unsafe`, `macos_12_unavailable`). Глушим в `src-tauri/vendor/wry/Cargo.toml` (`[lints.rust]`, `check-cfg`), исходники не трогаем.
25. `libsqlite3-sys` (bundled C) не собирается с macOS для `x86_64-pc-windows-msvc` и `x86_64-win7-windows-msvc`: `cc` вызывает Apple clang без Windows-заголовков (`stdlib.h not found`). Полный Windows-`cargo check` локально не работает. Проверять отдельную копию `wry` (раздел 8), полную сборку делать в CI.
26. `cargo check --offline` для отдельной копии `wry` падает на `http-range` (зависимость для Linux, в индексе кэша её нет). Нужна сеть на чтение `index.crates.io` и `static.crates.io`.
27. Отсутствие IID интерфейса в DLL не доказывает, что runtime его не поддерживает. Окончательно проверяет только запуск на VM.

## 7. Открытые решения и следующие шаги

Статус: импорты решены. Pure-путь подтверждён тестом v3 на VM. Патч `wry` и выбор папки написаны (раздел 4.2), собраны локально, на VM не проверены. Не закоммичено. Следующий шаг: проверка на VM (п. 2).

1. Выполнено: `scripts/win7/test-env.c` v3 прошёл на VM (раздел 5). Повтор: собрать на Mac (`/tmp/envtest/test-env.exe`, архив `~/Downloads/test-env.zip`), запустить на VM: распаковать в `C:\test\`, в cmd `cd C:\test`, затем `test-env.exe`. Результат: в консоли и в `C:\test\env-result.txt` (файл пишется после каждого шага; при падении там строки `exception code=… module_offset=…`). Ожидаемый успех:
   - `CreateWebViewEnvironmentWithOptionsInternal returned: 0x00000000`;
   - `environment completion hr: 0x00000000`;
   - `browser version: 109.0.1518.78`;
   - `controller completion hr: 0x00000000`.
   Тест ждёт до 60 с на каждый этап и печатает таймаут. В v3 сначала создаётся controller, потом вызывается `get_BrowserVersionString`. В `env-result.txt` дополнительно пишутся `environment vtbl[0..7]` и `+0x10` (смещения относительно DLL), это диагностика.
2. Патч `wry` 0.57.0 для Windows 7 сделан и собран локально (раздел 4.2), на VM не проверен. Дальше:
   - Коммит и CI-артефакт (`win7-installer.yml`) только с разрешения пользователя (раздел 9).
   - Проверка на VM: установить, запустить и прислать `%USERPROFILE%\.tulsu-pe-timetable\webview2.log`. Ожидаемо: строка `выбрана папка: …` и окно Tauri без диалога `Could not find the WebView2 Runtime`.
   - Нет строки `выбрана папка`: в логе видно, на каком шаге папка отвергнута (`LoadLibraryW` или `GetProcAddress`, с кодом ошибки).
   - Окно не открывается: в логе будет `panic: …` с текстом ошибки Tauri или `wry`. Для подробностей DebugView (Sysinternals), фильтр `WebView2:`.
3. Если environment создаётся, а controller падает: проверить подпись браузера (SSU `KB4490628` + SHA-2 `KB4474419` v3 из `/tmp/pu/`) и недостающие `api-ms-win-*` (через импорты `msedge_elf.dll` и `msedge.dll`).
4. Если environment не создаётся: DebugView (Sysinternals) на VM, фильтр по `WebView2:`. Loader пишет причину через `OutputDebugString`.
5. Путь B (запасной для пользователя): Go-версия на Win7-ПК, Tauri только для Windows 10/11. Рекомендовать, если пункты 2–4 не работают.

## 8. Команды

- Локально: `pnpm check`; `cargo check --manifest-path src-tauri/Cargo.toml`; `rustc --edition 2024 --crate-type lib --emit=metadata --target x86_64-pc-windows-msvc src-tauri/src/win7_compat.rs --out-dir <tmp>`; `pnpm tauri build --debug --no-bundle --config src-tauri/tauri.win7.conf.json` (потом удалить `src-tauri/target/debug/WebView2`).
- Win7-таргет для `wry` без sqlite (полная локальная сборка не работает, раздел 6, п. 25): `rm -rf /tmp/wry-check && cp -R src-tauri/vendor/wry /tmp/wry-check && cp src-tauri/Cargo.lock /tmp/wry-check/`, затем `RUSTC_BOOTSTRAP=1 CARGO_UNSTABLE_BUILD_STD=std cargo check --manifest-path /tmp/wry-check/Cargo.toml --target x86_64-win7-windows-msvc` (нужен `rust-src`, сеть на чтение crates.io). Для `x86_64-pc-windows-msvc` — то же без `RUSTC_BOOTSTRAP` и `CARGO_UNSTABLE_BUILD_STD`.
- Тест pure-Go пути: `x86_64-w64-mingw32-gcc -O2 -fno-stack-protector -nostdlib -nostartfiles -Wl,-e,mainCRTStartup -o test-env.exe scripts/win7/test-env.c -lkernel32 -lole32 -luser32`; проверка импортов `x86_64-w64-mingw32-objdump -p test-env.exe | grep "DLL Name"` (ожидаем только `KERNEL32`, `USER32`, `ole32`).
- CI: `gh workflow run win7-installer.yml --ref <ветка>`; `gh run list --workflow win7-installer.yml`; `gh run view <id> --log-failed`.
- Артефакт: `gh run download <id> --name tulsu-pe-timetable-win7 --dir <tmp>`; `7z x "<tmp>/*.exe" -o<tmp>/extract`; `python3 -B scripts/win7-imports.py <tmp>/extract/tulsu-pe-timetable.exe`.
- Тесты на VM: `scripts/win7/test-env.c` (главный), `scripts/win7/test-loader2.ps1` и `scripts/win7/check-runtime.ps1` (для `test-loader2.ps1` нужен `C:\test\WebView2Loader.dll`).
- Обновления (скачивание на Mac): Microsoft Update Catalog, `https://www.catalog.update.microsoft.com/Search.aspx?q=KB…`, затем POST на `DownloadDialog.aspx` с `updateIDs` (GUID из поиска). Ссылки `.msu` на `catalog.s.download.windowsupdate.com`.

## 9. Состояние git

- `main`: Win7-код из PR #76, #77 и #78 (squash). Актуальный хеш и последние коммиты см. в `git log`.
- `build` = `origin/build` = `99e1fa6`: старая ветка PR #78. После squash её история не совпадает с `main`, новую работу из неё не вести.
- Локально есть `backup/build-ad449dc` (старая история, только локально) и `win7` (старая ветка, удалённой нет).
- Диагностика Win7 (`docs/windows7.md`, `scripts/win7/`) отправлена веткой `win7-diagnostics` и смёржена в `main` через PR (squash). Номер PR и хеш см. в `git log`. `docs/code-review.md` принадлежит другому агенту, его не трогать и не коммитить.
- Текущая правка Win7 (патч `wry`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/win7_compat.rs`, `src-tauri/vendor/`, `AGENTS.md`, `docs/windows7.md`) не закоммичена. Коммит и ветку делаем только по явной просьбе (правило проекта).
- Проверить: `git --no-pager log --oneline -5` и `git --no-optional-locks status --short`.
