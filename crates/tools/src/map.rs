use aj_models::errors::{AdaJudgeError, InvalidProblem};
use axum::http::StatusCode;

pub trait MapHttpExt<T> {
    fn map_http(self) -> Result<T, (StatusCode, AdaJudgeError)>;
}

impl<T> MapHttpExt<T> for Result<T, AdaJudgeError> {
    fn map_http(self) -> Result<T, (StatusCode, AdaJudgeError)> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err((
                match &e {
                    AdaJudgeError::InvalidUsernameOrPassword => StatusCode::BAD_REQUEST,
                    AdaJudgeError::AlreadyExists => StatusCode::CONFLICT,
                    AdaJudgeError::NotFound => StatusCode::NOT_FOUND,
                    AdaJudgeError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                    AdaJudgeError::InvalidProblem(kind) => match kind {
                        InvalidProblem::SubgroupConflict {
                            subgroup: _,
                            depends_on: _,
                        } => StatusCode::CONFLICT,
                        InvalidProblem::InvalidSubgroupScoring { subgroup: _ } => {
                            StatusCode::BAD_REQUEST
                        }
                    },
                },
                e,
            )),
        }
    }
}
