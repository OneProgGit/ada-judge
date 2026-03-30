//! Mapping tools

use axum::http::StatusCode;
use models::verdicts::TotalVerdict;

/// Extension for logging an error in `Result<T, E>`
pub trait MapLogExt<T, E: std::error::Error> {
    /// - If `Result<T, E>` is `Ok(T)`, just returns a result
    /// - If it is `Err(E)`, logs an error and returns a result
    /// # Errors
    /// Returns an error if `self` is `Err(E)`
    fn map_log(self, verdict: TotalVerdict) -> Result<T, TotalVerdict>;
}

impl<T, E: std::error::Error> MapLogExt<T, E> for Result<T, E> {
    fn map_log(self, verdict: TotalVerdict) -> Result<T, TotalVerdict> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                log::error!("{e}");
                Err(verdict)
            }
        }
    }
}

/// Extension converting `Result<T, TotalVerdict>` to `Result<T, StatusCode>`
pub trait MapHttpExt<T> {
    /// - If `Result<T, TotalVerdict>` is `Ok(T)`, just returns a result
    /// - If it is `Err(TotalVerdict)`, returns a `Result<T, StatusCode>`
    /// # Errors
    /// Returns an error if `self` is `Err(TotalVerdict)`
    fn map_http(self) -> Result<T, StatusCode>;
}

impl<T> MapHttpExt<T> for Result<T, TotalVerdict> {
    fn map_http(self) -> Result<T, StatusCode> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => match e {
                TotalVerdict::Bug => Err(StatusCode::INTERNAL_SERVER_ERROR),
                _ => Err(StatusCode::BAD_REQUEST),
            },
        }
    }
}
