#[macro_export]
macro_rules! wrap {
    ($res:expr) => {
        match $res {
            Ok(val) => Ok($crate::api::response::ApiResponse::ok(val)),
            Err(err) => Ok($crate::api::response::ApiResponse::err(err)),
        }
    };
}
