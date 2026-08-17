use crate::submissions::update_submission;
use aj_models::{errors::AdaJudgeError, verdicts::TestingVerdict};
use sqlx::PgPool;

#[allow(async_fn_in_trait)]
pub trait MapDbExt<T> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TestingVerdict>;
}

impl<T: Send> MapDbExt<T> for Result<T, TestingVerdict> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Self {
        if let Err(_) = &self {
            update_submission(pool, submission_id, &TestingVerdict::Fail, 0.)
                .await
                .map_err(|_| TestingVerdict::Fail)?;
        }
        self
    }
}

impl<T: Send> MapDbExt<T> for Result<T, AdaJudgeError> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TestingVerdict> {
        if let Err(_) = &self {
            update_submission(pool, submission_id, &TestingVerdict::Fail, 0.)
                .await
                .map_err(|_| TestingVerdict::Fail)?;
        }
        self.map_err(|_| TestingVerdict::Fail)
    }
}
