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
