use aj_models::errors::Error;
use axum::http::StatusCode;

pub trait MapHttpExt<T> {
    fn map_http(self) -> Result<T, (StatusCode, Error)>;
}

impl<T> MapHttpExt<T> for Result<T, Error> {
    fn map_http(self) -> Result<T, (StatusCode, Error)> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err((
                match e {
                    Error::InvalidUsernameOrPassword => StatusCode::BAD_REQUEST,
                    Error::AlreadyExists => StatusCode::CONFLICT,
                    Error::NotFound => StatusCode::NOT_FOUND,
                    Error::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                },
                e,
            )),
        }
    }
}
