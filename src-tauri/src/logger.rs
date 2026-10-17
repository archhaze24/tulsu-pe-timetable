//! Журнал действий: строки в `logs/app-<дата>.log` рядом с базой.
//!
//! Пишем операции, которые меняют данные или показывают результат (создание, изменение, назначение,
//! снятие, привязка, архивация, экспорт), старт и ошибки с кодом и технической причиной.
//! Клики и навигацию не пишем (`AGENTS.md`). Формат строки: `время  УРОВЕНЬ  [раздел] сообщение`.
//!
//! Журналы прошлых дней сжимаются в `app-<дата>.log.zip` при старте и при смене дня.
//! Файлы старше `RETENTION_DAYS` (2 года) удаляются.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::{Duration, Local, NaiveDate};

use crate::archive;

/// Журналы старше этого срока (2 года) удаляются при старте.
const RETENTION_DAYS: i64 = 730;

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
pub fn init(db_path: &Path) -> io::Result<()> {
    let dir = db_path.parent().map(|p| p.join("logs")).unwrap_or_else(|| PathBuf::from("logs"));
    fs::create_dir_all(&dir)?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    archive_old_logs(&dir, &today);
    remove_old_files(&dir);
    let _ = LOGGER.set(Mutex::new(Logger { dir, current: None }));
    info("app", "приложение запущено");
    Ok(())
}

/// Дата журнала из имени `app-<дата>.log` или `app-<дата>.log.zip`.
fn log_date(name: &str) -> Option<NaiveDate> {
    let rest = name.strip_prefix("app-")?;
    let date = rest.strip_suffix(".log.zip").or_else(|| rest.strip_suffix(".log"))?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// Упаковывает журналы всех дней, кроме текущего: в него идёт запись.
fn archive_old_logs(dir: &Path, today: &str) {
    let current = format!("app-{today}.log");
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with("app-") && name.ends_with(".log") && name != current {
            let path = entry.path();
            if let Err(e) = archive::zip_file(&path) {
                eprintln!("не удалось упаковать журнал {}: {e}", path.display());
            }
        }
    }
}

fn remove_old_files(dir: &Path) {
    let cutoff = Local::now().date_naive() - Duration::days(RETENTION_DAYS);
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(date) = name.to_str().and_then(log_date) else { continue };
        if date < cutoff {
            let _ = fs::remove_file(entry.path());
        }
    }
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
            // Старый файл закрывается здесь, до упаковки.
            Ok(file) => logger.current = Some((day.clone(), file)),
            Err(e) => {
                eprintln!("не удалось открыть лог {}: {e}", path.display());
                return;
            }
        }
        archive_old_logs(&logger.dir, &day);
    }
    if let Some((_, file)) = logger.current.as_mut() {
        let _ = file.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_date_reads_plain_and_zipped_names() {
        assert_eq!(log_date("app-2026-10-16.log"), NaiveDate::from_ymd_opt(2026, 10, 16));
        assert_eq!(log_date("app-2026-10-16.log.zip"), NaiveDate::from_ymd_opt(2026, 10, 16));
        assert_eq!(log_date("timetable.db"), None);
        assert_eq!(log_date("app-latest.log"), None);
    }
}
