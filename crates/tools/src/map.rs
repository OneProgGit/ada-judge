use aj_models::errors::{AdaJudgeError, Deletion, InvalidProblem, Register};
use axum::{Json, http::StatusCode};

pub trait MapHttpExt<T> {
    fn map_http(self) -> Result<T, (StatusCode, Json<AdaJudgeError>)>;
}

impl<T> MapHttpExt<T> for Result<T, AdaJudgeError> {
    fn map_http(self) -> Result<T, (StatusCode, Json<AdaJudgeError>)> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err((
                match &e {
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
                    AdaJudgeError::InvalidJwt => todo!(),
                    AdaJudgeError::Register(kind) => match kind {
                        Register::InvalidLoginOrPassword => StatusCode::BAD_REQUEST,
                        Register::AlreadyExists => StatusCode::CONFLICT,
                        Register::PasswordsDontMatch => StatusCode::BAD_REQUEST,
                    },
                    AdaJudgeError::Deletion(kind) => match kind {
                        Deletion::InvalidLoginOrPassword => StatusCode::BAD_REQUEST,
                        Deletion::MissingDeletionConfirmation => StatusCode::BAD_REQUEST,
                    },
                },
                Json(e),
            )),
        }
    }
}
