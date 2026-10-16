use std::collections::HashMap;
use std::sync::OnceLock;

const ERRORS_JSON: &str = include_str!("errors.json");

static ERRORS: OnceLock<HashMap<String, String>> = OnceLock::new();

fn flatten(prefix: &str, value: &serde_json::Value, out: &mut HashMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let full = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                flatten(&full, nested, out);
            }
        }
        serde_json::Value::String(text) => {
            out.insert(prefix.to_owned(), text.clone());
        }
        _ => {}
    }
}

fn load(source: &str) -> HashMap<String, String> {
    let value: serde_json::Value = serde_json::from_str(source).expect("embedded locale JSON is valid");
    let mut out = HashMap::new();
    flatten("", &value, &mut out);
    out
}


/// Текст ошибки по ключу из `errors.json`; при отсутствии ключа возвращает сам ключ.
pub fn error(key: &str) -> String {
    lookup(&ERRORS, ERRORS_JSON, key)
}

fn lookup(cell: &OnceLock<HashMap<String, String>>, source: &str, key: &str) -> String {
    let map = cell.get_or_init(|| load(source));
    map.get(key).cloned().unwrap_or_else(|| key.to_owned())
}
