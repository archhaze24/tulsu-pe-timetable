//! Журнал приложения: читаемые строки в `logs/app-<дата>.log` рядом с базой.
//!
//! Пишем операции записи, экспорт, старт и ошибки с первопричиной. Клики по интерфейсу не логируем.
//! Формат строки: `время  УРОВЕНЬ  [раздел] сообщение`.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

struct Logger {
    dir: PathBuf,
    /// Файл текущего дня. Перед записью проверяем дату, чтобы переключаться на новый файл в полночь.
    current: Option<(String, File)>,
}

static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

/// Инициализирует журнал в папке `logs` рядом с базой данных.
pub fn init(db_path: &Path) {
    let dir = db_path.parent().map(|p| p.join("logs")).unwrap_or_else(|| PathBuf::from("logs"));
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("не удалось создать папку логов {}: {e}", dir.display());
        return;
    }
    let _ = LOGGER.set(Mutex::new(Logger { dir, current: None }));
    info("app", "приложение запущено");
}

pub fn info(section: &str, message: impl AsRef<str>) {
    write(Level::Info, section, message.as_ref());
}

pub fn warn(section: &str, message: impl AsRef<str>) {
    write(Level::Warn, section, message.as_ref());
}

pub fn error(section: &str, message: impl AsRef<str>) {
    write(Level::Error, section, message.as_ref());
}

fn write(level: Level, section: &str, message: &str) {
    let now = Local::now();
    let line = format!(
        "{}  {}  [{}] {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        level.label(),
        section,
        message
    );

    // Дублируем в stderr, чтобы при запуске из терминала ошибки были видны сразу.
    if level != Level::Info {
        eprint!("{line}");
    }

    let Some(cell) = LOGGER.get() else { return };
    let Ok(mut logger) = cell.lock() else { return };
    let day = now.format("%Y-%m-%d").to_string();
    if logger.current.as_ref().map(|(d, _)| d != &day).unwrap_or(true) {
        let path = logger.dir.join(format!("app-{day}.log"));
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => logger.current = Some((day, file)),
            Err(e) => {
                eprintln!("не удалось открыть лог {}: {e}", path.display());
                return;
            }
        }
    }
    if let Some((_, file)) = logger.current.as_mut() {
        let _ = file.write_all(line.as_bytes());
    }
}
