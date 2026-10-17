//! Запуск приложения. Окно создаётся только после успешной загрузки конфига и открытия базы,
//! иначе пользователь увидел бы пустое окно. Ошибки показываются нативным диалогом.
//!
//! Диалоги неблокирующие: блокирующий вызов в `setup` (главный поток) зависает.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult};

use crate::config::{self, Config};
use crate::{logger, storage, AppState};

const TITLE: &str = "Расписание ТулГУ по физкультуре";
const RESET_LABEL: &str = "Сбросить";
const PICK_LABEL: &str = "Выбрать файл";
const EXIT_LABEL: &str = "Выйти";

pub fn start(app: &AppHandle) {
    let config = match config::load() {
        Ok(config) => config,
        Err(e) => return config_failed(app, &e.to_string()),
    };
    let db_path = match config::validate_db_path(&config.db_path) {
        Ok(path) => path,
        Err(e) => return config_failed(app, &e.to_string()),
    };

    if let Err(e) = logger::init(&db_path) {
        warn_only(app, &format!("Журнал не работает: не удалось создать папку логов.\n\n{e}"));
    }

    let conn = match storage::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => return database_failed(app, config, &e.to_string()),
    };

    app.manage(AppState { conn: Mutex::new(conn), config: Mutex::new(config) });
    if let Err(e) = open_main_window(app) {
        fatal(app, &format!("Не удалось открыть окно приложения.\n\n{e}"));
    }
}

fn open_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window_config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    WebviewWindowBuilder::from_config(app, &window_config)?.build()?;
    Ok(())
}

fn config_failed(app: &AppHandle, detail: &str) {
    let handle = app.clone();
    let message = format!(
        "Не удалось прочитать настройки приложения.\n\n{detail}\n\nСбросить настройки на значения по умолчанию? \
         Прежний файл сохранится как config.json.bak."
    );
    app.dialog()
        .message(message)
        .title(TITLE)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(RESET_LABEL.to_owned(), EXIT_LABEL.to_owned()))
        .show_with_result(move |answer| {
            if !is_custom(&answer, RESET_LABEL) {
                handle.exit(1);
                return;
            }
            match config::reset_to_default() {
                Ok(_) => handle.restart(),
                Err(e) => {
                    show_error(&handle, &format!("Не удалось сбросить настройки.\n\n{e}"));
                    handle.exit(1);
                }
            }
        });
}

/// Базу не открыть: предлагаем выбрать другой файл. Путь сохраняется в config.json, затем перезапуск.
fn database_failed(app: &AppHandle, config: Config, detail: &str) {
    let handle = app.clone();
    let message = format!(
        "Не удалось открыть базу данных:\n{}\n\n{detail}\n\nВыбрать другой файл базы? \
         Иначе приложение закроется, путь можно исправить в config.json.",
        config.db_path
    );
    app.dialog()
        .message(message)
        .title(TITLE)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(PICK_LABEL.to_owned(), EXIT_LABEL.to_owned()))
        .show_with_result(move |answer| {
            if !is_custom(&answer, PICK_LABEL) {
                handle.exit(1);
                return;
            }
            let picker = handle.clone();
            handle
                .dialog()
                .file()
                .set_title("Файл базы данных")
                .add_filter("База данных", &["db", "sqlite", "sqlite3"])
                .pick_file(move |picked| {
                    let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
                        picker.exit(1);
                        return;
                    };
                    let next = Config { db_path: path.to_string_lossy().into_owned() };
                    match config::save(&next) {
                        Ok(()) => picker.restart(),
                        Err(e) => {
                            show_error(&picker, &format!("Не удалось сохранить путь к базе.\n\n{e}"));
                            picker.exit(1);
                        }
                    }
                });
        });
}

fn fatal(app: &AppHandle, message: &str) {
    let handle = app.clone();
    show_error_then(app, message, move || handle.exit(1));
}

fn show_error(app: &AppHandle, message: &str) {
    app.dialog().message(message).title(TITLE).kind(MessageDialogKind::Error).show(|_| {});
}

fn show_error_then(app: &AppHandle, message: &str, then: impl FnOnce() + Send + 'static) {
    app.dialog()
        .message(message)
        .title(TITLE)
        .kind(MessageDialogKind::Error)
        .show(move |_| then());
}

fn warn_only(app: &AppHandle, message: &str) {
    app.dialog().message(message).title(TITLE).kind(MessageDialogKind::Warning).show(|_| {});
}

fn is_custom(answer: &MessageDialogResult, label: &str) -> bool {
    matches!(answer, MessageDialogResult::Custom(text) if text == label)
}
