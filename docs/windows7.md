# Windows 7: статус, контекст и грабли

Файл для агентов и разработчиков. Здесь всё про сборку под Windows 7: что сделано, почему, что проверено и что нет. Перед любыми изменениями Win7-сборки читать целиком.

Обновлено: 2026-10-17. Если что-то устарело, сверьте с `git log`, `gh run list` и исправьте этот файл.

## 1. Цель и контекст

- Нужно: Tauri-версия работает на ПК кафедры физкультуры с Windows 7 SP1. Обычная сборка (Windows 10/11) — `.github/workflows/windows.yml`.
- Рабочая версия для Win7 сейчас — Go-версия из ветки `legacy-go-version-win7`: go1.20.14, Wails 2.7.1, WebView2 109 руками в `C:\WebView2`. Она работает на Win7-ПК. Пока Tauri на Win7 не заработает, её не удаляем.
- На Win7 максимум WebView2 109 (Chromium 109). Поэтому фронтенд для win7 собирается с `target: chrome109`.
- Репозиторий приватный: `archhaze24/tulsu-pe-timetable`. Пользователь работает на macOS (Apple Silicon). Проверки на Win7 делает он на своих ПК.
- Правило проекта: не коммитить и не пушить без явной просьбы.

## 2. Текущий статус (главное)

- Ветка `build` (= `origin/build`), HEAD `4c66424 feat: windows build action` (коммит пользователя). Это PR #77 в `main`. Проверка `check` из `windows.yml` на PR прошла.
- Установщик Win7 собирается в CI: `win7-installer.yml`, прогон `38970975453` на `main@98c100c`, результат успешный, артефакт `tulsu-pe-timetable-win7` 157.7 МБ. В нём нет правок из раздела 4.2.
- На Win7-ПК приложение НЕ запускается: `The procedure entry point EventSetInformation could not be located in the dynamic link library advapi32.dll`.
- Загрузчик Windows падает на первом отсутствующем импорте, поэтому каждый прогон на Win7-ПК показывает только одну причину.
- Сделано, но НЕ проверено ни в CI, ни на Win7 (незакоммичено, см. раздел 9): delay-load и хуки для трёх Win8+ импортов (раздел 4.2). Ожидаемый результат: `scripts/win7-imports.py` не находит подозрительных импортов в обычной таблице.
- Следующий шаг: коммит и прогон `win7-installer.yml` на `build`. Это нужно разрешение пользователя (раздел 7).

## 3. Win8+ импорты и что с ними сделано

Список из артефакта `38970975453`. Полный список функций по DLL: `python3 scripts/win7-imports.py <exe>`.

| Импорт | Источник | Решение | Статус |
|---|---|---|---|
| `advapi32.dll!EventSetInformation` (ETW, Windows 8+) | Статический `WebView2LoaderStatic.lib` (Microsoft). Динамический `WebView2Loader.dll` импортирует только `KERNEL32` (86 функций) | `advapi32.dll` в `/DELAYLOAD`. Заглушка в `__pfnDliFailureHook2` возвращает 0 | сделано, не проверено |
| `combase.dll`: `CoCreateFreeThreadedMarshaler`, `CoTaskMemFree`, `CoTaskMemAlloc` | Крейты `windows` и `windows-core` 0.62.2 | `combase.dll` в `/DELAYLOAD`. `__pfnDliNotifyHook2` подменяет DLL на `ole32.dll`, где эти функции есть и на Windows 7 | сделано, не проверено |
| `api-ms-win-core-winrt-error-l1-1-0.dll!RoOriginateErrorW` (Windows 8+) | Крейт `windows-result` 0.4.1, вызов в `originate_error` (только для сообщений ошибок) | DLL в `/DELAYLOAD`. Подмена на `kernel32.dll`. Дальше заглушка `RoOriginateErrorW` (возвращает FALSE) | сделано, не проверено |
| `api-ms-win-crt-*-l1-1-0.dll` (динамический UCRT, на Win7 нужен KB2999226) | `static_vcruntime` (метабилд Tauri) добавляет `/DEFAULTLIB:ucrt.lib`, если `staticVCRuntime: true` (по умолчанию) | `build.windows.staticVCRuntime: false` и `+crt-static` для MSVC | сделано, не проверено |

Проверено по документации Microsoft (`learn.microsoft.com`):
- `CompareStringOrdinal`: Windows Vista. На Win7 есть.
- `SetWindowDisplayAffinity`: Windows 7 (`WDA_EXCLUDEFROMCAPTURE` на Windows 7 работает как `WDA_MONITOR`). На Win7 есть.

По знанию API (страницы не открывал) считаю Win7-совместимыми функции обычной таблицы: kernel32, user32, gdi32, ole32, shell32, shlwapi, comctl32 (v6), dwmapi, imm32, oleaut32, ntdll и `advapi32` без `EventSetInformation`. Если на Win7-ПК появится новый импорт, сначала проверить его по документации.

Что ещё не проверено: импорты kernel32 из статического UCRT, которые появятся после `staticVCRuntime: false`. Проверим в следующем артефакте (скрипт + список функций вручную).

## 4. Что сделано и почему

### 4.1 Фронтенд (`frontend/`)

- `vite.config.ts`: режим `vite build --mode win7` даёт `build.target: 'chrome109'` и плагин `chrome109Css` (только для build).
  - Почему: Tailwind 4 пишет `oklch()` и `color-mix(in oklab, var(--color-…) P%, transparent)`. WebView2 109 их не понимает, и цвета пропадают.
  - Как: lightningcss с `targets: { chrome: 109 << 16 }` в два прохода. 1) `oklch` → hex у палитры `--color-*`. 2) `color-mix(... var(--color-X) P%, transparent)` → подстановка hex из карты палитры, затем второй проход lightningcss, который даёт `#rrggbbaa`.
  - Плейсхолдер: фолбэк Tailwind `::placeholder{color:currentColor}` (непрозрачный) заменяется на `…;opacity:.5}`. У placeholder нет фона, так что результат тот же, что у 50% цвета в современных браузерах.
  - Проверка: если после сборки остались `oklch(` или `color-mix(... var(`, сборка падает с ошибкой.
- `package.json`: скрипт `build:win7` = `vite build --mode win7`, devDependency `lightningcss ^1.33.0`.
- Проверено: в win7-CSS 0 вхождений `oklch(` и 0 `color-mix(... var(`. Все `lab()` стоят внутри `@supports (color:lab(...))`. У каждого `color-mix`-варианта внутри `@supports` есть аналог вне него, так что Chromium 109 получает те же стили.
- Почему плагин внутри `vite.config.ts`, а не отдельным файлом:
  - Импорт `./x.ts` без расширения даёт warning Vite (native config loader).
  - С расширением TS требует `allowImportingTsExtensions` и `noEmit` в `tsconfig.node.json`. Это даёт warning svelte-check: "Referenced project may not disable emit". Поэтому плагин держим в `vite.config.ts`, а `tsconfig.node.json` не трогаем.
- Go-версия использовала Tailwind 3 (rgb/hex), поэтому такой проблемы там не было.

### 4.2 Конфиги и Rust (`src-tauri/`)

- `tauri.win7.conf.json` (в `build`, коммит `4c66424`):
  - `build.beforeBuildCommand`: `pnpm --dir frontend build:win7`;
  - `build.windows.staticVCRuntime: false` (см. пункт про UCRT в разделе 3);
  - `bundle.targets`: `["nsis"]`;
  - `bundle.windows.webviewInstallMode`: `{ "type": "fixedRuntime", "path": "./WebView2/" }`.
  - Как работает fixedRuntime: при старте Tauri ставит `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` = папка exe + `path` (tauri 2.12.4, `app.rs`). `tauri-build` копирует папку для windows-таргетов.
- `.cargo/config.toml` (в `build`, коммит `4c66424`): `[target.'cfg(all(windows, target_env = "msvc"))'] rustflags = ["-C", "target-feature=+crt-static"]`.
  - Почему: `cc` (sqlite через `rusqlite` bundled) компилирует C с `/MD`, если нет `crt-static`. Tauri линкует VC-рантайм статически, поэтому возможен конфликт CRT. Это касается и обычной Windows-сборки.
- `tauri.conf.json` (в `build`, коммит `4c66424`): `bundle.icon` += `icons/icon.ico`, как в шаблонах Tauri. Для NSIS не обязательно, для MSI безопаснее.
- `build.rs` (ЛОКАЛЬНО, не закоммичено):
  - Если `CARGO_CFG_TARGET_VENDOR == "win7"`, в бинарник добавляются `delayimp.lib` и `/DELAYLOAD:` для трёх DLL: `advapi32.dll`, `combase.dll`, `api-ms-win-core-winrt-error-l1-1-0.dll`.
  - Флаги `cargo:rustc-link-arg-bins`, то есть только бинарник. Обычная Windows-сборка не меняется.
- `src/win7_compat.rs` и `src/main.rs` (ЛОКАЛЬНО, не закоммичено):
  - `main.rs`: `#[cfg(all(windows, target_vendor = "win7"))] mod win7_compat;`.
  - `__pfnDliNotifyHook2` (`dliNotePreLoadLibrary`): сначала `LoadLibraryA(исходная DLL)`. Если NULL, резерв: `combase.dll` → `ole32.dll`, `api-ms-win-core-winrt-error-l1-1-0.dll` → `kernel32.dll`. Для остальных DLL возвращает NULL, и загрузчик грузит их сам.
  - `__pfnDliFailureHook2` (`dliFailGetProc`): `EventSetInformation` → заглушка, возвращает 0 (успех без действия). `RoOriginateErrorW` → заглушка, возвращает 0 (FALSE). Другие имена → NULL, то есть при вызове будет исключение «procedure entry point», а не падение на старте.
  - Почему `EventSetInformation` возвращает успех: вызывает её трассировка WebView2 loader, и ненулевой код может увести её в ветку ошибки.
  - Почему хуки в бинарнике, а не в lib: объекты бинарника линкуются всегда. Объект из rlib-архива линкер может не подтянуть. Тогда хуки молча останутся пустыми, а delayimp возьмёт значения по умолчанию.
  - Edition 2024: `#[unsafe(no_mangle)]` и `unsafe extern "system" { ... }`.
  - `LoadLibraryA` объявлен вручную, новых крейтов нет.
  - Почему этот путь, а не динамический `WebView2Loader.dll`: проще (нет копии крейта `webview2-com-sys` ~10 МБ и правок `[patch]`). Динамический loader остаётся запасным вариантом, раздел 7.
- `.gitignore`:
  - `src-tauri/WebView2`: runtime 444 МБ, `msedge.dll` 229 МБ, а GitHub не принимает файлы больше 100 МБ.
  - `WebView2-*.zip`.

### 4.3 Скрипты (`scripts/`)

- `build-win7.mjs` (в `main` и `build`):
  - проверяет `src-tauri/WebView2/msedgewebview2.exe`;
  - запускает `pnpm tauri build --config src-tauri/tauri.win7.conf.json <аргументы>` с переменными окружения:
    - `RUSTUP_TOOLCHAIN` = `process.env.WIN7_TOOLCHAIN ?? 'stable'`;
    - `RUSTC_BOOTSTRAP=1` и `CARGO_UNSTABLE_BUILD_STD=std` (сборка std под tier-3 таргет из исходников);
    - `CARGO_BUILD_TARGET=x86_64-win7-windows-msvc`. Это переменная, а не `--target` (см. раздел 6, пункт 2).
- `win7-imports.py` (ЛОКАЛЬНО, переписан): разбор таблицы импортов PE на чистом Python 3 без зависимостей.
  - Смотрит обычную таблицу (грузится на старте) и отложенную (delay-load, грузится при первом вызове).
  - В обычной таблице подозрительный импорт — проблема. В отложенной — ок, если есть хук в `win7_compat.rs`.
  - Печатает DLL с числом функций и подозрительные импорты (Win8+ и UCRT).

### 4.4 CI (`.github/workflows/`)

- `win7-installer.yml` (в `main`):
  - триггер `workflow_dispatch`, runner `windows-latest`;
  - Rust: `dtolnay/rust-toolchain@master` с `toolchain: ${{ env.WIN7_TOOLCHAIN }}` и компонентом `rust-src`. `Swatinem/rust-cache@v2` с `workspaces: src-tauri`;
  - Node 24, pnpm 12.6.0;
  - шаг «Скачать WebView2»: `gh release download webview2-109 --pattern WebView2-109.zip`, затем `Expand-Archive` в `src-tauri/WebView2` и проверка `msedgewebview2.exe` в корне;
  - затем `pnpm install --frozen-lockfile`, `pnpm build:win7`, артефакт `tulsu-pe-timetable-win7` (`*.exe` из `bundle/nsis`), retention 7 дней.
- `windows.yml` (в `build`, PR #77): обычная сборка для Win10/11.
  - `check` (`windows-latest`): PR, push в `main`, теги. На теге проверка, что тег `vX.Y.Z` совпадает с `version` в `src-tauri/tauri.conf.json`. Затем `pnpm check`. На PR #77 прошёл.
  - `build` (`needs: check`): теги `v*` и вручную. `pnpm build`, артефакт `windows-installers` (nsis и msi), retention 14 дней.
  - `release` (ubuntu, теги): `gh release create` или `gh release upload --clobber`.
  - `build` и `release` ещё не запускались.
- Версии, которые важны: `WIN7_TOOLCHAIN: 1.99.1` (std под win7 собиралась именно на ней), `PNPM_VERSION: 12.6.0`, `NODE_VERSION: '24'`.

### 4.5 Релиз WebView2

- Релиз `webview2-109` в приватном репозитории, ассет `WebView2-109.zip` (207 755 364 байт). Загружен пользователем.
- Тег `webview2-109` указывает на старый коммит `5666346`. Это не мешает: `windows.yml` работает только на теги `v*`.
- Архив = содержимое папки runtime, `msedgewebview2.exe` лежит в корне. Создание: `cd src-tauri/WebView2 && zip -r ../../WebView2-109.zip . && cd ../..`.
- Версия: 109.0.1518.78 (файл `109.0.1518.78.manifest` в папке).

## 5. Проверки: что уже подтверждено

- Артефакт `38970975453`, `tulsu-pe-timetable.exe` (сборка до правок из раздела 4.2): обычная таблица импорта — kernel32, advapi32, ntdll, user32, ole32, gdi32, shlwapi, comctl32, dwmapi, combase, shell32, imm32, oleaut32, `api-ms-win-core-winrt-error-l1-1-0`, `api-ms-win-crt-{string,math,heap,utility,time,runtime,convert,stdio,locale}-l1-1-0`. Нет `vcruntime140` и `msvcp140`, то есть VC-рантайм статический.
- Тот же артефакт через `scripts/win7-imports.py`: ровно четыре проблемы из раздела 3 (EventSetInformation, combase, winrt-error, UCRT). Скрипт работает.
- Установщик содержит `WebView2/msedgewebview2.exe`.
- `WebView2Loader.dll` (x64, из `webview2-com-sys` 0.39.1) импортирует только KERNEL32.
- `WebView2LoaderStatic.lib` содержит строки `EventSetInformation` (4 раза) и `CoTaskMemAlloc`.
- Локально на macOS:
  - `rustc --edition 2024 --crate-type lib --emit=metadata --target x86_64-pc-windows-msvc src-tauri/src/win7_compat.rs` (rustc 1.99.1): без ошибок и предупреждений. Линковки это не проверяет. Для запуска нужен `rustup target add x86_64-pc-windows-msvc`, он уже установлен.
  - `cargo check --manifest-path src-tauri/Cargo.toml` (host): без предупреждений. cfg `target_vendor = "win7"` rustc принимает.
  - `pnpm check`: exit 0, svelte-check 0 ошибок и 0 предупреждений (ранее).
  - `pnpm tauri build --debug --no-bundle --config src-tauri/tauri.win7.conf.json`: схема конфига принята, `beforeBuildCommand` отработал (ранее).
  - `actionlint` по обоим workflow: чисто (ранее).
- CI `windows.yml` (`check`) на PR #77: success. Это `cargo check` на Windows с новым `.cargo/config.toml`.
- Документация Microsoft: `CompareStringOrdinal` (Vista), `SetWindowDisplayAffinity` (Windows 7).
- Не проверено: линковка win7-exe с delay-load (нужна сборка в CI); импорты нового exe; запуск на Win7.

## 6. Грабли (не повторять)

1. Nightly от 2026-10-15 не собирает std для `x86_64-win7-windows-msvc`: ошибка `is_console` в `library/std/src/sys/stdio/windows.rs`. Работает stable 1.99.1 с `RUSTC_BOOTSTRAP=1`.
2. Tauri CLI отвергает `--target x86_64-win7-windows-msvc` с сообщением "does not exist": проверка идёт по `rustup target list`. Решение: переменная `CARGO_BUILD_TARGET` (Tauri читает её раньше конфигов, см. `cargo_config.rs`).
3. `build.windows.staticVCRuntime` по умолчанию `true` → `static_vcruntime` добавляет `ucrt.lib` → динамический UCRT → `api-ms-win-crt-*`. Для win7 ставим `false`.
4. `cc` без `crt-static` собирает C с `/MD`. Нужен `+crt-static` для всех MSVC-таргетов.
5. Динамическая загрузка и статический импорт — разные вещи. tao, wry и tauri-runtime-wry грузят DPI-функции через `get_function!` (то есть `GetProcAddress`): это безопасно для Win7. Проблемы только у статических импортов, которые видны в таблице импорта.
6. Ошибка загрузчика = первый отсутствующий импорт, а не полный список. Исправление одного открывает следующий. Поэтому одна ошибка на Win7-ПК ничего не говорит о количестве остальных. Полный список берём скриптом.
7. Git: runtime не коммитить. `msedge.dll` 229 МБ > лимит GitHub 100 МБ, и файл навсегда останется в истории. Только релиз.
8. Microsoft публикует Fixed Version только для двух последних мажорных версий. 109 брать из `C:\WebView2` рабочего Win7-ПК или из архива.
9. Vite: локальный `.ts`-импорт без расширения даёт warning, с расширением нужен `allowImportingTsExtensions` и `noEmit`, что ломает svelte-check. Плагин держим в `vite.config.ts`.
10. `tsc -p frontend/tsconfig.node.json` выдаёт 40 ошибок про `Buffer` из типов vite. Это не наши ошибки, в `pnpm check` они не участвуют.
11. `pnpm/action-setup@v4` даёт предупреждение про Node 20. Работает, так как принудительно идёт на node24.
12. wry: часть вызовов WebView2 через `cast()?` на интерфейсах новее 109 (`set_theme`, `clear_all_browsing_data`, `set_memory_usage_level` и др.). Это не стартовый путь (проверено чтением кода), но проверить при тесте на Win7.
13. Хуки delay-load должны жить в бинарнике (`main.rs`), а не в rlib. Иначе линкер может не подтянуть объект, и хуки останутся пустыми без ошибки сборки.
14. Edition 2024: `#[no_mangle]` пишется как `#[unsafe(no_mangle)]`, `extern "system" {}` как `unsafe extern "system" {}`.
15. Код только для Windows 7 проверяется на macOS через `rustc --emit=metadata --target x86_64-pc-windows-msvc` (без линковки). Если cargo показывает старый результат («Fresh»), для повторной проверки предупреждений cfg сделайте `touch` файла.
16. Ответ «нет» на вопрос «заработает ли текущий код» был верен до правок раздела 4.2. Сейчас ожидаем, но не доказали.
17. `gh` в этом треде работает без песочницы. Для `gh` не нужны `allow_hosts` и `unsandboxed`.

## 7. Открытые решения и следующие шаги

Статус: путь A (Tauri на Win7) выбран пользователем: «нам ОЧЕНЬ НЕОБХОДИМО». Старый шаг 1 (динамический `WebView2Loader.dll`) заменён delay-load и хуками (раздел 4.2). Динамический loader остаётся запасным вариантом.

Следующие шаги:
1. Закоммитить изменения и запустить `gh workflow run win7-installer.yml --ref build`. Это нужно разрешение пользователя (правило проекта).
2. Скачать артефакт и запустить `scripts/win7-imports.py`. Ожидаем:
   - в обычной таблице нет `advapi32!EventSetInformation`, `combase`, `api-ms-win-core-winrt-error`, `api-ms-win-crt-*`;
   - в отложенной таблице есть `advapi32`, `combase`, `api-ms-win-core-winrt-error-l1-1-0`.
3. Вручную просмотреть функции kernel32 и user32 из статического UCRT. Скрипт видит только список `SUSPECT_FUNCTIONS`, остальное проверять по документации.
4. Пользователь ставит установщик на Win7-ПК и сообщает результат. Если ошибка — новый первый импорт или исключение «procedure entry point» при вызове, разбираем её.
5. Запасной вариант, если статический loader не работает на Win7: динамический `WebView2Loader.dll`.
   - (a) `[patch.crates-io]` с локальной копией `webview2-com-sys` и условием `cfg(target_vendor = "win7")` в макросе `link_webview2`. Минус: копия крейта ~10 МБ.
   - (b) таргет `x86_64-win7-windows-gnu` (не проверено, нужен MinGW в CI).
   - Положить `WebView2Loader.dll` рядом с exe через `bundle.resources`.
6. Путь B (запасной для пользователя): оставить Go-версию (`legacy-go-version-win7`) на Win7-ПК, Tauri выпускать только для Windows 10/11. Риск низкий, работает сразу.

Рекомендация: если после двух-трёх итераций на Win7-ПК Tauri не запускается, перейти на путь B, а не тратить время на подмену импортов.

## 8. Команды

- Локально:
  - `pnpm check`
  - `pnpm --filter frontend run build:win7` (проверка CSS: 0 `oklch(`, 0 `color-mix(... var(`)
  - `pnpm tauri build --debug --no-bundle --config src-tauri/tauri.win7.conf.json` (валидация конфига на macOS)
  - `rustc --edition 2024 --crate-type lib --emit=metadata --target x86_64-pc-windows-msvc src-tauri/src/win7_compat.rs --out-dir <tmp>` (проверка типов Win7-кода, без линковки)
  - `cargo check --manifest-path src-tauri/Cargo.toml` (предупреждения host-сборки)
- CI Win7:
  - `gh workflow run win7-installer.yml --ref build` (после коммита и пуша) или через вкладку Actions
  - статус: `gh run list --workflow win7-installer.yml`
  - логи упавшего шага: `gh run view <id> --log-failed`
- Артефакт и импорты:
  - `gh run download <id> --name tulsu-pe-timetable-win7 --dir <tmp>`
  - `7z x "<tmp>/<имя>-setup.exe" -o<tmp>/extract` (нужен p7zip: `brew install p7zip`)
  - `python3 scripts/win7-imports.py "<tmp>/extract/tulsu-pe-timetable.exe"`
  - временную папку удалять после проверки, она не в `.gitignore`.
- Не запускать `pnpm dev` без явной просьбы (см. `AGENTS.md`).

## 9. Состояние git (на момент записи)

- Ветка `build` = `origin/build`, HEAD `4c66424 feat: windows build action`. Коммит пользователя, PR #77 → `main`. Основа: `98c100c` (`main`, PR #76 «windows 7 support»).
- Коммит `4c66424` содержит: `windows.yml`, `AGENTS.md`, `README.md`, `src-tauri/.cargo/config.toml`, `src-tauri/tauri.conf.json`, `src-tauri/tauri.win7.conf.json`.
- Не закоммичено:
  - изменены: `src-tauri/build.rs`, `src-tauri/src/main.rs`;
  - новые: `src-tauri/src/win7_compat.rs`, `scripts/win7-imports.py` (переписан), `docs/windows7.md`.
- Проверить актуальное состояние: `git --no-pager log --oneline -5` и `git --no-optional-locks status --short`.
