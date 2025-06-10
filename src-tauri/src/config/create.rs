use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_path: "test.db".into(),
        }
    }
}

impl AppConfig {
    fn new(db_path: String) -> AppConfig {
        AppConfig { db_path }
    }
}

pub fn get_or_create_config() -> anyhow::Result<AppConfig> {
    let project_dirs = ProjectDirs::from("com", "tulsu", "pe-timetable")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let mut config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir)?;

    let mut data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    if cfg!(debug_assertions) {
        config_dir = Path::new("test-app-data");
        fs::create_dir_all(config_dir)?;

        data_dir = Path::new("test-app-data");
        fs::create_dir_all(data_dir)?;
    }

    let config_file_path = config_dir.join("config.toml");
    if !config_file_path.exists() {
        let db_path: PathBuf = if cfg!(debug_assertions) {
            data_dir.join("test.sqlite")
        } else {
            data_dir.join("db.sqlite")
        };
        let config = AppConfig::new(db_path.to_str().unwrap().to_string());
        let toml_string = toml::to_string_pretty(&config)?;
        fs::write(&config_file_path, toml_string)?;
        Ok(config)
    } else {
        let contents = fs::read_to_string(config_file_path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}