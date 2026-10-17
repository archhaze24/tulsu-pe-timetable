use std::fmt;

/// Ошибка приложения. Во фронтенд уходит только `code` — ключ i18n из `frontend/src/lib/locales/ru.json`.
/// Технические подробности (текст ошибки SQLite и т.п.) попадают только в журнал.
#[derive(Debug)]
pub struct AppError {
    code: &'static str,
    detail: Option<String>,
}

impl AppError {
    pub fn key(code: &'static str) -> Self {
        Self { code, detail: None }
    }

    pub fn with_cause(code: &'static str, cause: impl fmt::Display) -> Self {
        Self { code, detail: Some(cause.to_string()) }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {detail}", self.code),
            None => f.write_str(self.code),
        }
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    /// Каждый код `errors.*` из исходников должен быть переведён во фронтенде: иначе пользователь увидит ключ.
    #[test]
    fn every_error_code_has_translation() {
        let locale: serde_json::Value =
            serde_json::from_str(include_str!("../../frontend/src/lib/locales/ru.json")).unwrap();
        let mut translated = BTreeSet::new();
        collect_keys(&locale, "", &mut translated);

        let mut codes = BTreeSet::new();
        collect_codes(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(), &mut codes);

        let missing: Vec<_> = codes.difference(&translated).collect();
        assert!(missing.is_empty(), "нет перевода в ru.json: {missing:?}");
    }

    fn collect_keys(value: &serde_json::Value, prefix: &str, out: &mut BTreeSet<String>) {
        if let serde_json::Value::Object(map) = value {
            for (key, nested) in map {
                let full = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                collect_keys(nested, &full, out);
            }
        } else {
            out.insert(prefix.to_owned());
        }
    }

    fn collect_codes(dir: &Path, out: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_codes(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = fs::read_to_string(&path).unwrap();
                // Литералы в кавычках: нечётные элементы после split('"').
                for literal in source.split('"').skip(1).step_by(2) {
                    let is_code = literal.starts_with("errors.")
                        && literal.matches('.').count() >= 2
                        && literal.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '.');
                    if is_code {
                        out.insert(literal.to_owned());
                    }
                }
            }
        }
    }
}
