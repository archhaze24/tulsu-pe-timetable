use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{AppError, AppResult};

/// Совместим с config.json из Go-версии: ключ `dbPath`. Поле `theme` больше не используется, в старых файлах оно игнорируется.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub db_path: String,
}

pub fn config_file() -> AppResult<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| AppError::key("errors.config.path_failed"))?;
    Ok(base.join("tulsu-pe-timetable").join("config.json"))
}

/// Загружает конфиг. Если файла нет — создаёт конфиг по умолчанию.
pub fn load() -> AppResult<Config> {
    let path = config_file()?;
    if !path.exists() {
        return create_default();
    }

    let raw = fs::read_to_string(&path).map_err(|e| AppError::with_cause("errors.config.read_failed", e))?;
    serde_json::from_str(&raw).map_err(|e| AppError::with_cause("errors.config.parse_failed", e))
}

/// Переносит повреждённый конфиг в `config.json.bak` и создаёт конфиг по умолчанию.
pub fn reset_to_default() -> AppResult<Config> {
    let path = config_file()?;
    if path.exists() {
        fs::rename(&path, path.with_extension("json.bak"))
            .map_err(|e| AppError::with_cause("errors.config.save_failed", e))?;
    }
    create_default()
}

fn create_default() -> AppResult<Config> {
    let home = dirs::home_dir().ok_or_else(|| AppError::key("errors.config.home_dir_failed"))?;
    let data_dir = home.join(".tulsu-pe-timetable");
    fs::create_dir_all(&data_dir).map_err(|e| AppError::with_cause("errors.config.data_dir_failed", e))?;

    let config = Config {
        db_path: data_dir.join("timetable.db").to_string_lossy().into_owned(),
    };
    save(&config)?;
    Ok(config)
}

/// Путь к базе должен быть абсолютным. Существование каталога проверяет открытие базы.
pub fn validate_db_path(path: &str) -> AppResult<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::key("errors.config.db_path_empty"));
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(AppError::key("errors.config.db_path_invalid"));
    }
    Ok(path)
}

/// Пишет во временный файл и переименовывает: сбой посередине не оставит битый config.json.
pub fn save(config: &Config) -> AppResult<()> {
    let path = config_file()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| AppError::with_cause("errors.config.create_dir_failed", e))?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| AppError::with_cause("errors.config.save_failed", e))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| AppError::with_cause("errors.config.save_failed", e))?;
    fs::rename(&tmp, &path).map_err(|e| AppError::with_cause("errors.config.save_failed", e))
}
