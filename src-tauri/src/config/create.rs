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

pub fn _get_or_create_config() -> anyhow::Result<AppConfig> {
   todo!()
}
