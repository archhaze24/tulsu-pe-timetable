# Расписание ТулГУ по физкультуре

Приложение для составления расписания занятий по физкультуре: направления, преподаватели со ставкой,
факультеты, семестры и пары. Экспорт расписания в XLSX.

Стек: Tauri 2 (Rust) + Svelte 5 + TypeScript + Tailwind 4.

Предыдущая версия на Go + Wails сохранена в ветке `legacy-go-version`.

## Требования

- [Node.js](https://nodejs.org/) и [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (через rustup)
- Системные зависимости Tauri для вашей ОС: см. [документацию Tauri](https://tauri.app/start/prerequisites/)

## Установка и запуск

Все команды выполняются из корня проекта. Одна установка ставит зависимости и корня, и `frontend/`:

```sh
pnpm install
pnpm dev      # окно приложения с горячей перезагрузкой фронтенда
pnpm build    # сборка приложения
```

## Проверки

```sh
pnpm check    # cargo check + svelte-check + сборка фронтенда, без окна
```

Типы фронтенда для моделей и запросов генерируются из Rust (`ts-rs`):

```sh
pnpm gen:types
```

Файлы лежат в `frontend/src/lib/generated`: их не редактируют руками. Запускайте команду после изменения
структур в `src-tauri`.

## Релиз для Windows

Установщики (NSIS и MSI) собирает workflow `.github/workflows/windows.yml`. Он проверяет проект (`pnpm check`) на каждом PR и push в `main`.

Чтобы выпустить версию:

1. Поднять `version` в `src-tauri/tauri.conf.json`: именно он попадает в установщик.
2. Создать тег с таким же номером и отправить его: `git tag v1.2.3 && git push origin v1.2.3`.
3. Workflow проверит, что тег совпадает с `version`, соберёт установщики и опубликует их в Releases.

Без тега можно запустить сборку вручную (Actions → Windows → Run workflow): тогда установщики попадают только в артефакт запуска.

Установщики не подписаны: Windows SmartScreen покажет предупреждение («Выполнить в любом случае»). Подпись кода требует сертификата и отдельной настройки.

## Сборка для Windows 7

Основная сборка (`pnpm build`) рассчитана на Windows 10/11. Для ПК с Windows 7 есть отдельная сборка `pnpm build:win7`:

- установщик включает WebView2 Runtime 109 Fixed Version, отдельно ставить его не нужно;
- фронтенд собирается для Chromium 109: цвета Tailwind переводятся из `oklch` и `color-mix` в обычные `rgb`/`hex` (`frontend/vite.config.ts`);
- exe собирается со статической CRT, поэтому VC++ Runtime на ПК не нужен.

### Сборка в GitHub Actions

Workflow `.github/workflows/win7-installer.yml` собирает установщик на `windows-latest`. Запуск вручную: вкладка **Actions** → **Windows 7 installer** → **Run workflow**. Установщик появится в артефакте запуска и хранится 7 дней.

Архив WebView2 в git не храним: `msedge.dll` весит 229 МБ, а GitHub не принимает файлы больше 100 МБ. Его кладут в релиз, а workflow скачивает его сам:

1. Заархивировать содержимое папки `src-tauri/WebView2` так, чтобы `msedgewebview2.exe` лежал в корне архива:
   - macOS/Linux: `cd src-tauri/WebView2 && zip -r ../../WebView2-109.zip . && cd ../..`
   - Windows (PowerShell): `Compress-Archive -Path src-tauri\WebView2\* -DestinationPath WebView2-109.zip`
2. Создать релиз с тегом `webview2-109` и приложить архив:
   `gh release create webview2-109 WebView2-109.zip --title "WebView2 Fixed Version 109" --notes "Runtime для сборки Windows 7"`
   или через сайт: Releases → Draft a new release.
3. Если тег или имя архива другие, поправьте `WEBVIEW2_RELEASE_TAG` и `WEBVIEW2_ASSET` в workflow.

Релиз доступен всем, кто видит репозиторий. Если репозиторий публичный, WebView2 станет открыт всем, а это сторонний продукт Microsoft: для публичного репозитория лучше не выкладывать архив и сначала проверить условия лицензии Microsoft.

Версия Rust для CI зафиксирована в `WIN7_TOOLCHAIN` (`1.99.1`): с ней std для Windows 7 собиралась в проверке. Обновляйте её осознанно.

### Сборка на своём Windows-ПК

1. Rust stable и компонент `rust-src`: `rustup component add rust-src`. Таргет Windows 7 поставляется без готовой std, поэтому она собирается из исходников. Скрипт сам выставляет нужные переменные окружения.
2. Папка WebView2 Fixed Version 109 в `src-tauri/WebView2/`, внутри должен лежать `msedgewebview2.exe`. Проще всего скопировать `C:\WebView2` с рабочего Windows 7 ПК. Либо распаковать `Microsoft.WebView2.FixedVersionRuntime.109.*.x64.cab` командой `expand <cab> -F:* src-tauri` и переименовать появившуюся папку в `WebView2`. Microsoft публикует на сайте только две последние мажорные версии, поэтому старый cab может понадобиться из архива.
3. Сборка: `pnpm build:win7`. Установщик появится в `src-tauri/target/x86_64-win7-windows-msvc/release/bundle/nsis/`.

### Что проверено

Пока частично: конфигурация Tauri, фронтенд, сборка std для таргета Windows 7 и PowerShell-шаг workflow (на macOS). Workflow ещё не запускался на GitHub. Полный установщик и запуск на Windows 7 не проверялись: перед раздачей его нужно запустить на ПК кафедры. Новый CSS и JS должны работать в Chromium 109: в этой сборке они не проверяются автоматически.
