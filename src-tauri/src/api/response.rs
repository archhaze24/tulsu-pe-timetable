use serde::Serialize;

// Обёртка для хендлеров: используем при каждом ответе.
#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok { ok: bool, data: T },
    Err { ok: bool, error: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse::Ok { ok: true, data }
    }

    pub fn err<E: ToString>(error: E) -> Self {
        ApiResponse::Err {
            ok: false,
            error: error.to_string(),
        }
    }
}

impl<T> From<Result<T, String>> for ApiResponse<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(data) => ApiResponse::ok(data),
            Err(e) => ApiResponse::err(e),
        }
    }
}
