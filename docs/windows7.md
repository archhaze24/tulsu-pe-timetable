# Windows 7: статус, контекст и грабли

Файл для агентов и разработчиков. Здесь всё про сборку под Windows 7: что сделано, почему, что проверено и что нет. Перед любыми изменениями Win7-сборки читать целиком.

Обновлено: 2026-10-17 (после прогона `38975552911`). Если что-то устарело, сверьте с `git log`, `gh run list` и исправьте этот файл.

## 1. Цель и контекст

- Нужно: Tauri-версия работает на ПК кафедры физкультуры с Windows 7 SP1. Обычная сборка (Windows 10/11) — `.github/workflows/windows.yml`.
- Рабочая версия для Win7 сейчас — Go-версия из ветки `legacy-go-version-win7`: go1.20.14, Wails 2.7.1, WebView2 109 руками в `C:\WebView2`. Она работает на Win7-ПК. Пока Tauri на Win7 не заработает, её не удаляем.
- На Win7 максимум WebView2 109 (Chromium 109). Поэтому фронтенд для win7 собирается с `target: chrome109`.
- Репозиторий приватный: `archhaze24/tulsu-pe-timetable`. Пользователь работает на macOS (Apple Silicon). Проверки на Win7 делает он на своих ПК.
- Правило проекта: не коммитить и не пушить без явной просьбы.

## 2. Текущий статус (главное)

- Импорты exe проходят загрузчик Windows 7. Подтверждено прогоном `38975552911` (`main@4d7d90a`, успех, 11m44s) и скриптом `scripts/win7-imports.py` (раздел 3).
- На Win7-ПК приложение теперь ЗАПУСКАЕТСЯ, но:
  - открывается пустое окно с рамкой;
  - показывается диалог `Could not find the WebView2 Runtime. ...` (текст из `tauri-runtime-wry`, `lib.rs`, при `webview_runtime_installed == false`);
  - после OK приложение закрывается. Это ожидаемо: Tauri выходит, когда runtime не найден.
- Причина не установлена. Это уже не импорты, а поиск WebView2 Runtime. Гипотезы:
  1. Путь. Tauri ставит `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` = `<папка exe>\./WebView2/`. В пути есть кириллица и пробелы (productName «Расписание ТулГУ по физкультуре»). Go-версия использует `C:\WebView2`.
  2. Версия загрузчика. Наш статический загрузчик от `webview2-com-sys 0.39.1` имеет версию файла 1.0.3800.47. Старый загрузчик из `go-webview2 v1.0.10` (тот, что работает в Go-версии) имеет версию 1.0.992.28. Обе версии содержат одинаковый набор строк, включая `GetCurrentApplicationUserModelId` и `api-ms-win-appmodel-runtime-l1-1-2`. Статически различие не выявлено.
- Текущая сборка (незакоммичена, раздел 4.2) выбирает папку runtime сама и пишет диагностику в `%USERPROFILE%\.tulsu-pe-timetable\webview2.log`. Следующий шаг: результат её прогона на Win7-ПК (раздел 7).

## 3. Импорты exe

Список из артефакта `38970975453` (до фиксов) и из `38975552911` (после фиксов). Полный список функций по DLL: `python3 scripts/win7-imports.py <exe>`.

| Импорт (до фиксов) | Что сделано | Статус |
|---|---|---|
| `advapi32.dll!EventSetInformation` (ETW, Windows 8+). Приходит из статического `WebView2LoaderStatic.lib`, динамический `WebView2Loader.dll` импортирует только `KERNEL32` | `advapi32.dll` в `/DELAYLOAD`. Заглушка в `__pfnDliFailureHook2` возвращает 0 | решено, подтверждено скриптом |
| `combase.dll`: `CoCreateFreeThreadedMarshaler`, `CoTaskMemFree`, `CoTaskMemAlloc` (крейты `windows` и `windows-core` 0.62.2) | `combase.dll` в `/DELAYLOAD`. `__pfnDliNotifyHook2` подменяет DLL на `ole32.dll` (на Windows 7 эти функции там есть) | решено, подтверждено скриптом |
| `api-ms-win-core-winrt-error-l1-1-0.dll!RoOriginateErrorW` (крейт `windows-result`, вызов только для сообщений ошибок) | DLL в `/DELAYLOAD`. Подмена на `kernel32.dll`, дальше заглушка возвращает FALSE | решено, подтверждено скриптом |
| `api-ms-win-crt-*-l1-1-0.dll` (динамический UCRT, на Win7 нужен KB2999226). Причина: `static_vcruntime` добавляет `/DEFAULTLIB:ucrt.lib` при `staticVCRuntime: true` | `build.windows.staticVCRuntime: false` и `+crt-static` для MSVC | решено, подтверждено скриптом |

Что проверено по документации Microsoft (`learn.microsoft.com`): `CompareStringOrdinal` (Windows Vista), `SetWindowDisplayAffinity` (Windows 7). По знанию API (страницы не открывал) на Windows 7 есть функции обычной таблицы: kernel32, user32, gdi32, ole32, shell32, shlwapi, comctl32 v6, dwmapi, imm32, oleaut32, ntdll.

Новые импорты `kernel32` из статического UCRT (+22 функции, появились после `staticVCRuntime: false`): `CompareStringW`, `ExitThread`, `FreeLibraryAndExitThread`, `GetACP`, `GetCPInfo`, `GetCommandLineA`, `GetConsoleOutputCP`, `GetFileType`, `GetOEMCP`, `GetStartupInfoW`, `GetStringTypeW`, `GetTimeZoneInformation`, `IsDebuggerPresent`, `IsProcessorFeaturePresent`, `IsValidCodePage`, `LCMapStringW`, `SetFilePointerEx`, `SetStdHandle`, `TerminateProcess`, `UnhandledExceptionFilter`, `VirtualProtect`, `VirtualQuery`. Все они из эпохи Windows 2000/XP/Vista (по знанию API).

Итог скрипта на `38975552911`: в обычной таблице 11 DLL (`kernel32`, `ntdll`, `user32`, `ole32`, `gdi32`, `shlwapi`, `comctl32`, `dwmapi`, `shell32`, `imm32`, `oleaut32`), подозрительных импортов нет. В отложенной: `advapi32`, `combase`, `api-ms-win-core-winrt-error-l1-1-0`.

## 4. Что сделано и почему

### 4.1 Фронтенд (`frontend/`)

- `vite.config.ts`: режим `vite build --mode win7` даёт `build.target: 'chrome109'` и плагин `chrome109Css` (только для build).
  - Почему: Tailwind 4 пишет `oklch()` и `color-mix(in oklab, var(--color-…) P%, transparent)`. WebView2 109 их не понимает, и цвета пропадают.
  - Как: lightningcss с `targets: { chrome: 109 << 16 }` в два прохода. 1) `oklch` → hex у палитры `--color-*`. 2) `color-mix(... var(--color-X) P%, transparent)` → подстановка hex из карты палитры, затем второй проход lightningcss, который даёт `#rrggbbaa`.
  - Плейсхолдер: фолбэк Tailwind `::placeholder{color:currentColor}` заменяется на `…;opacity:.5}`. У placeholder нет фона, так что результат тот же.
  - Проверка: если после сборки остались `oklch(` или `color-mix(... var(`, сборка падает с ошибкой.
- `package.json`: скрипт `build:win7` = `vite build --mode win7`, devDependency `lightningcss ^1.33.0`.
- Проверено: в win7-CSS 0 вхождений `oklch(` и 0 `color-mix(... var(`. Все `lab()` стоят внутри `@supports (color:lab(...))`.
- Плагин держим внутри `vite.config.ts`: отдельный `.ts`-файл без расширения даёт warning Vite, с расширением нужны `allowImportingTsExtensions` и `noEmit`, что даёт warning svelte-check.

### 4.2 Конфиги и Rust (`src-tauri/`)

- `tauri.win7.conf.json` (в `main` с PR #77, менялся в текущей сборке):
  - `build.beforeBuildCommand`: `pnpm --dir frontend build:win7`;
  - `build.windows.staticVCRuntime: false` (см. UCRT в разделе 3);
  - `bundle.targets`: `["nsis"]`;
  - `bundle.resources`: `{ "WebView2": "WebView2" }`. Папка runtime кладётся в установку как ресурс: `$INSTDIR\WebView2\msedgewebview2.exe`. Раньше это делал `fixedRuntime`;
  - `bundle.windows.webviewInstallMode`: `{ "type": "skip" }`. Раньше было `fixedRuntime` с `path: "./WebView2/"`. Почему изменено: при `fixedRuntime` Tauri (tauri 2.12.4, `app.rs`, блок с `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`) всегда перезаписывает переменную значением `<папка exe>\./WebView2/`. Выбрать другую папку было нельзя. При `skip` Tauri переменную не трогает, и её выставляет наш код в `main.rs` (см. ниже).
- `.cargo/config.toml` (в `main` с PR #77): `[target.'cfg(all(windows, target_env = "msvc"))'] rustflags = ["-C", "target-feature=+crt-static"]`. Почему: `cc` (sqlite через `rusqlite` bundled) компилирует C с `/MD`, если нет `crt-static`. Касается и обычной Windows-сборки.
- `tauri.conf.json` (в `main` с PR #77): `bundle.icon` += `icons/icon.ico`.
- `build.rs` (в `main` с PR #77): если `CARGO_CFG_TARGET_VENDOR == "win7"`, в бинарник добавляются `delayimp.lib` и `/DELAYLOAD:` для `advapi32.dll`, `combase.dll`, `api-ms-win-core-winrt-error-l1-1-0.dll`. Флаги `cargo:rustc-link-arg-bins`. Обычная Windows-сборка не меняется.
- `src/main.rs`: `#[cfg(all(windows, target_vendor = "win7"))] mod win7_compat;` и вызов `win7_compat::prepare_webview2_runtime()` перед `tulsu_pe_timetable_lib::run()` (тоже только win7; в текущей сборке, незакоммичено).
- `src/win7_compat.rs` (в `main` с PR #77, функция выбора папки и лог добавлены в текущей сборке):
  - Хуки delay-load: `__pfnDliNotifyHook2` (`dliNotePreLoadLibrary`) сначала пробует исходную DLL через `LoadLibraryA`, при неудаче берёт резерв (`combase.dll` → `ole32.dll`, winrt-error → `kernel32.dll`). `__pfnDliFailureHook2` (`dliFailGetProc`) даёт заглушки: `EventSetInformation` возвращает 0 (успех без действия), `RoOriginateErrorW` возвращает FALSE. Для остальных имён возвращает NULL, тогда при вызове будет исключение «procedure entry point».
  - Хуки в бинарнике, а не в rlib: объекты бинарника линкуются всегда. Объект из rlib-архива линкер может не подтянуть, и хуки молча останутся пустыми.
  - `prepare_webview2_runtime()` (до старта Tauri):
    1. Если `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` уже задана, выбор пропускается.
    2. Иначе кандидаты по порядку: `<папка exe>\WebView2`, затем `C:\WebView2` (её использует Go-версия). Для каждого вызывается `GetAvailableCoreWebView2BrowserVersionString(папка)`. Первая папка с HRESULT ≥ 0 записывается в переменную окружения.
    3. Финальная проверка `GetAvailable(NULL)`: та же, что делает Tauri (`wry::webview_version()`).
    4. Лог (после каждого шага, чтобы не потерять данные при аварии): `%USERPROFILE%\.tulsu-pe-timetable\webview2.log`. В строке: папка, есть ли `msedgewebview2.exe` и `EBWebView\x64\EmbeddedBrowserWebView.dll`, HRESULT в hex и версия.
  - Почему `EventSetInformation` возвращает успех: её вызывает трассировка WebView2 loader, и ненулевой код может увести её в ветку ошибки.
  - Edition 2024: `#[unsafe(no_mangle)]` и `unsafe extern "system" { ... }`. `LoadLibraryA` и `GetAvailableCoreWebView2BrowserVersionString` объявлены вручную, новых крейтов нет. Символ `GetAvailable...` есть в `WebView2LoaderStatic.lib` (его линкует `webview2-com-sys`).
- `.gitignore`: `src-tauri/WebView2` (runtime 444 МБ, `msedge.dll` 229 МБ, а GitHub не принимает файлы больше 100 МБ), `WebView2-*.zip`.

### 4.3 Скрипты (`scripts/`)

- `build-win7.mjs`: проверяет `src-tauri/WebView2/msedgewebview2.exe` и запускает `pnpm tauri build --config src-tauri/tauri.win7.conf.json` с переменными:
  - `RUSTUP_TOOLCHAIN` = `process.env.WIN7_TOOLCHAIN ?? 'stable'`;
  - `RUSTC_BOOTSTRAP=1` и `CARGO_UNSTABLE_BUILD_STD=std` (сборка std под tier-3 таргет из исходников);
  - `CARGO_BUILD_TARGET=x86_64-win7-windows-msvc`. Это переменная, а не `--target` (раздел 6, пункт 2).
- `win7-imports.py`: разбор таблицы импортов PE на чистом Python 3. Смотрит обычную таблицу (грузится на старте) и отложенную (delay-load). В обычной таблице подозрительный импорт — проблема, в отложенной — ок, если есть хук в `win7_compat.rs`.

### 4.4 CI (`.github/workflows/`)

- `win7-installer.yml` (в `main`): триггер `workflow_dispatch`, runner `windows-latest`.
  - Rust `1.99.1` (`WIN7_TOOLCHAIN`) с `rust-src`, `Swatinem/rust-cache@v2` с `workspaces: src-tauri`. Node 24, pnpm 12.6.0.
  - Шаг «Скачать WebView2»: `gh release download webview2-109 --pattern WebView2-109.zip`, затем `Expand-Archive` в `src-tauri/WebView2` и проверка `msedgewebview2.exe` в корне.
  - `pnpm install --frozen-lockfile`, `pnpm build:win7`, артефакт `tulsu-pe-timetable-win7` (`*.exe` из `bundle/nsis`), retention 7 дней.
- `windows.yml` (в `main` с PR #77): обычная сборка для Win10/11.
  - `check` (`windows-latest`): PR, push в `main`, теги. На теге проверка, что тег `vX.Y.Z` совпадает с `version` в `tauri.conf.json`. Затем `pnpm check`. На PR #77 прошёл.
  - `build` (`needs: check`): теги `v*` и вручную, `pnpm build`, артефакт `windows-installers` (nsis и msi). `release` (ubuntu, теги): `gh release create` или `gh release upload --clobber`. Эти джобы ещё не запускались.
- Версии: `WIN7_TOOLCHAIN: 1.99.1` (std под win7 собиралась на ней), `PNPM_VERSION: 12.6.0`, `NODE_VERSION: '24'`.

### 4.5 Релиз WebView2

- Релиз `webview2-109` в приватном репозитории, ассет `WebView2-109.zip` (207 755 364 байт), загружен пользователем. Тег указывает на старый коммит `5666346`, это не мешает (`windows.yml` работает на теги `v*`).
- Архив = содержимое папки runtime, `msedgewebview2.exe` лежит в корне. Создание: `cd src-tauri/WebView2 && zip -r ../../WebView2-109.zip . && cd ../..`.
- Версия: 109.0.1518.78 (файл `109.0.1518.78.manifest` в папке). Внутри: `EBWebView/x64/EmbeddedBrowserWebView.dll` (нужен загрузчику), `msedge.dll`, `msedgewebview2.exe`.

## 5. Проверки: что подтверждено

- CI:
  - `win7-installer.yml`, прогон `38970975453` (`main@98c100c`): успех, но импорты не проходили загрузчик (ошибка `EventSetInformation` на Win7-ПК).
  - `win7-installer.yml`, прогон `38975552911` (`main@4d7d90a`): успех, 11m44s, артефакт `tulsu-pe-timetable-win7`. Импорты проходят (раздел 3). Установщик содержит `WebView2/msedgewebview2.exe` и `WebView2/EBWebView/x64/EmbeddedBrowserWebView.dll`.
  - `windows.yml`, `check` на PR #77: успех.
- Загрузчики (сравнение, статический из `webview2-com-sys 0.39.1` и динамический из `go-webview2 v1.0.10`):
  - версии файлов 1.0.3800.47 и 1.0.992.28. Одинаковый набор строк (`EBWebView\x64\EmbeddedBrowserWebView.dll`, `version.dll`, `GetFileVersionInfoSizeW`, `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` и др.);
  - импорты: KERNEL32 (86 у нового, 80 у старого), отложенно `ADVAPI32`, `ole32`;
  - версионных констант (минимальной версии runtime) в строках не найдено.
- Документация Microsoft:
  - `GetAvailableCoreWebView2BrowserVersionString`: если задано переопределение (переменная окружения), оно используется; `ERROR_FILE_NOT_FOUND`, если runtime не найден;
  - `CreateCoreWebView2EnvironmentWithOptions`: путь к fixed runtime может быть относительным (от exe) или абсолютным.
- Go-версия (`origin/legacy-go-version-win7`, `main.go`): `WebviewBrowserPath: C:\WebView2`.
- Локально на macOS (текущая сборка):
  - `rustc --edition 2024 --crate-type lib --emit=metadata --target x86_64-pc-windows-msvc src-tauri/src/win7_compat.rs`: без ошибок и предупреждений;
  - `cargo check --manifest-path src-tauri/Cargo.toml`: без предупреждений;
  - `pnpm tauri build --debug --no-bundle --config src-tauri/tauri.win7.conf.json`: конфиг принят, сборка прошла;
  - `pnpm check`: exit 0.
- Не проверено: запуск на Win7 с текущей сборкой; линковка `GetAvailableCoreWebView2BrowserVersionString` в CI; то, что NSIS раскладывает ресурс `WebView2` в `$INSTDIR\WebView2`.

## 6. Грабли (не повторять)

1. Nightly от 2026-10-15 не собирает std для `x86_64-win7-windows-msvc`: ошибка `is_console` в `library/std/src/sys/stdio/windows.rs`. Работает stable 1.99.1 с `RUSTC_BOOTSTRAP=1`.
2. Tauri CLI отвергает `--target x86_64-win7-windows-msvc` с сообщением "does not exist": проверка идёт по `rustup target list`. Решение: переменная `CARGO_BUILD_TARGET`.
3. `build.windows.staticVCRuntime` по умолчанию `true` → `static_vcruntime` добавляет `ucrt.lib` → динамический UCRT → `api-ms-win-crt-*`. Для win7 ставим `false`.
4. `cc` без `crt-static` собирает C с `/MD`. Нужен `+crt-static` для всех MSVC-таргетов.
5. Динамическая загрузка и статический импорт — разные вещи. tao, wry и tauri-runtime-wry грузят DPI-функции через `GetProcAddress`: это безопасно. Проблемы только у статических импортов в таблице.
6. Ошибка загрузчика = первый отсутствующий импорт, а не полный список. Поэтому полный список берём скриптом.
7. Git: runtime не коммитить. `msedge.dll` 229 МБ > лимит GitHub 100 МБ, и файл навсегда останется в истории. Только релиз.
8. Microsoft публикует Fixed Version только для двух последних мажорных версий. 109 брать из `C:\WebView2` рабочего Win7-ПК или из архива.
9. Vite: локальный `.ts`-импорт без расширения даёт warning, с расширением нужен `allowImportingTsExtensions` и `noEmit`. Плагин держим в `vite.config.ts`.
10. `tsc -p frontend/tsconfig.node.json` выдаёт 40 ошибок про `Buffer` из типов vite. Это не наши ошибки, в `pnpm check` они не участвуют.
11. `pnpm/action-setup@v4` даёт предупреждение про Node 20. Работает, так как принудительно идёт на node24.
12. wry: часть вызовов WebView2 через `cast()?` на интерфейсах новее 109 (`set_theme`, `clear_all_browsing_data`, `set_memory_usage_level` и др.). Это не стартовый путь, но проверить при тесте на Win7.
13. Хуки delay-load должны жить в бинарнике (`main.rs`), а не в rlib. Иначе линкер может не подтянуть объект, и хуки останутся пустыми без ошибки сборки.
14. Edition 2024: `#[no_mangle]` пишется как `#[unsafe(no_mangle)]`, `extern "system" {}` как `unsafe extern "system" {}`.
15. Код только для Windows 7 проверяется на macOS через `rustc --emit=metadata --target x86_64-pc-windows-msvc` (без линковки). Если cargo показывает старый результат («Fresh»), для повторной проверки предупреждений cfg сделайте `touch` файла.
16. Tauri при `fixedRuntime` сам выставляет `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` и перезаписывает любое значение. Чтобы выбирать папку самим, нужен `webviewInstallMode: skip`.
17. `tauri build --no-bundle` на macOS с `bundle.resources` копирует `WebView2` (444 МБ) в `src-tauri/target/debug/`. После проверки удалить `src-tauri/target/debug/WebView2`.
18. Диалог «Could not find the WebView2 Runtime» показывает `tauri-runtime-wry`, когда `GetAvailableCoreWebView2BrowserVersionString(NULL)` вернул ошибку. Закрытие диалога завершает приложение (так устроен Tauri).
19. `gh` в этом треде работает без песочницы. Для `gh` не нужны `allow_hosts` и `unsandboxed`.

## 7. Открытые решения и следующие шаги

Статус: путь A (Tauri на Win7) выбран пользователем. Импорты решены. Сейчас проблема в поиске WebView2 Runtime.

Следующие шаги:
1. Закоммитить текущие изменения и запустить `win7-installer.yml` (нужно разрешение пользователя, правило проекта). Незакоммичено: `src-tauri/src/win7_compat.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.win7.conf.json`, `docs/windows7.md`.
2. Пользователь ставит установщик на Win7-ПК, запускает и присылает содержимое файла `%USERPROFILE%\.tulsu-pe-timetable\webview2.log` (например, `C:\Users\<имя>\.tulsu-pe-timetable\webview2.log`). Там три типа строк:
   - `<папка>: msedgewebview2.exe=..., EmbeddedBrowserWebView.dll=..., hr=0x..., версия=...` для `<папка exe>\WebView2` и `C:\WebView2`;
   - `как проверяет Tauri ...` — итоговая проверка, та же, что делает Tauri.
3. Как читать лог:
   - `<exe>\WebView2` даёт `hr=0x00000000` → путь был не тем. Дальше проверить, что приложение открывается.
   - `<exe>\WebView2` даёт ошибку, а `C:\WebView2` даёт `hr=0x00000000` → дело в пути (кириллица или пробелы). Варианты: ставить в ASCII-путь (например, поменять `productName` для win7 или копировать runtime в `%LOCALAPPDATA%`).
   - Оба дают ошибку с одним и тем же HRESULT → дело в загрузчике или версии runtime. Вариант: загрузчик из `go-webview2 v1.0.10` (тот, что работает в Go-версии) через динамическую загрузку `WebView2Loader.dll`. Это пункт 5 ниже.
   - Окно не открылось, а лог пуст или обрывается → упала диагностика или загрузчик, нужен текст ошибки Windows и содержимое лога.
4. Если запуск падает при вызове функции, сообщение «procedure entry point» назовёт функцию. Для неё нужна заглушка в `win7_compat.rs`.
5. Запасной вариант, если загрузчик не принимает runtime 109: динамический `WebView2Loader.dll` из `go-webview2 v1.0.10`.
   - (a) `[patch.crates-io]` с локальной копией `webview2-com-sys` и условием `cfg(target_vendor = "win7")` в макросе `link_webview2`. Минус: копия крейта ~10 МБ.
   - (b) таргет `x86_64-win7-windows-gnu` (не проверено, нужен MinGW в CI).
   - Положить DLL рядом с exe через `bundle.resources`.
6. Путь B (запасной для пользователя): оставить Go-версию на Win7-ПК, Tauri выпускать только для Windows 10/11. Рекомендация: если после двух-трёх итераций на Win7-ПК Tauri не работает, перейти на путь B.

## 8. Команды

- Локально:
  - `pnpm check`
  - `pnpm --filter frontend run build:win7` (проверка CSS: 0 `oklch(`, 0 `color-mix(... var(`)
  - `pnpm tauri build --debug --no-bundle --config src-tauri/tauri.win7.conf.json` (валидация конфига на macOS; потом удалить `src-tauri/target/debug/WebView2`)
  - `rustc --edition 2024 --crate-type lib --emit=metadata --target x86_64-pc-windows-msvc src-tauri/src/win7_compat.rs --out-dir <tmp>` (типы Win7-кода, без линковки)
  - `cargo check --manifest-path src-tauri/Cargo.toml` (предупреждения host-сборки)
- CI Win7:
  - `gh workflow run win7-installer.yml --ref <ветка>` (после коммита и пуша) или через вкладку Actions
  - статус: `gh run list --workflow win7-installer.yml`
  - логи упавшего шага: `gh run view <id> --log-failed`
- Артефакт и импорты:
  - `gh run download <id> --name tulsu-pe-timetable-win7 --dir <tmp>`
  - `7z x "<tmp>/<имя>-setup.exe" -o<tmp>/extract` (нужен p7zip: `brew install p7zip`)
  - `python3 -B scripts/win7-imports.py "<tmp>/extract/tulsu-pe-timetable.exe"` (`-B`, чтобы не создавать `__pycache__`)
  - временную папку удалять после проверки, она не в `.gitignore`.
- Не запускать `pnpm dev` без явной просьбы (см. `AGENTS.md`).

## 9. Состояние git (на момент записи)

- `main` = `origin/main` = `4d7d90a feat: windows build action (#77)` (squash-мердж PR #77). Основа: `98c100c` (PR #76 «windows 7 support»).
- Ветка `build` = `origin/build` = `8cec776 feat: win7 fixes`. Дерево такое же, как у `main`.
- Локальный `main` отстаёт (`4c66424`) и расходится с `origin/main` из-за squash-мерджа. Синхронизировать по решению пользователя.
- Незакоммичено: `src-tauri/src/win7_compat.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.win7.conf.json`, `docs/windows7.md`.
- Проверить актуальное состояние: `git --no-pager log --oneline -5` и `git --no-optional-locks status --short`.
