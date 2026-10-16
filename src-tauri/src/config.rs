use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{AppError, AppResult};

/// Совместим с config.json из Go-версии: те же ключи `dbPath` и `theme`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub db_path: String,
    pub theme: String,
}

pub fn config_file() -> AppResult<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| AppError::key("errors.config.path_failed"))?;
    let dir = base.join("tulsu-pe-timetable");
    fs::create_dir_all(&dir).map_err(|e| AppError::with_cause("errors.config.create_dir_failed", e))?;
    Ok(dir.join("config.json"))
}

/// Загружает конфиг. Если файла нет — создаёт конфиг по умолчанию.
pub fn load() -> AppResult<Config> {
    let path = config_file()?;
    if !path.exists() {
        return create_default();
    }

    let raw = fs::read_to_string(&path).map_err(|e| AppError::with_cause("errors.config.read_failed", e))?;
    let mut config: Config =
        serde_json::from_str(&raw).map_err(|e| AppError::with_cause("errors.config.parse_failed", e))?;
    if config.theme.is_empty() {
        config.theme = "light".to_owned();
        save(&config)?;
    }
    Ok(config)
}

fn create_default() -> AppResult<Config> {
    let home = dirs::home_dir().ok_or_else(|| AppError::key("errors.config.home_dir_failed"))?;
    let data_dir = home.join(".tulsu-pe-timetable");
    fs::create_dir_all(&data_dir).map_err(|e| AppError::with_cause("errors.config.data_dir_failed", e))?;

    let config = Config {
        db_path: data_dir.join("timetable.db").to_string_lossy().into_owned(),
        theme: "light".to_owned(),
    };
    save(&config)?;
    Ok(config)
}

pub fn save(config: &Config) -> AppResult<()> {
    let path = config_file()?;
    let json = serde_json::to_string_pretty(config).map_err(|e| AppError::with_cause("errors.config.save_failed", e))?;
    fs::write(&path, json).map_err(|e| AppError::with_cause("errors.config.save_failed", e))
}
