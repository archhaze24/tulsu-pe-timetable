use std::fmt;

use crate::locales;

/// Ошибка приложения. Сообщение всегда берётся из `errors.json`,
/// как и в Go-версии, поэтому пользователь видит тот же текст.
#[derive(Debug)]
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn key(key: &str) -> Self {
        Self { message: locales::error(key) }
    }

    /// Ошибка с числовым аргументом вместо `%d` в тексте локализации.
    pub fn key_with_arg(key: &str, arg: impl fmt::Display) -> Self {
        Self { message: locales::error(key).replace("%d", &arg.to_string()) }
    }

    /// Ошибка с кодом из локализации и подробностями от источника (например, SQLite).
    pub fn with_cause(key: &str, cause: impl fmt::Display) -> Self {
        Self { message: format!("{}: {}", locales::error(key), cause) }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;
