use crate::submissions::update_total_testing_result;
use aj_models::verdicts::TestingVerdict;
use sqlx::PgPool;

#[allow(async_fn_in_trait)]
pub trait MapDbExt<T> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TestingVerdict>;
}

impl<T: Send> MapDbExt<T> for Result<T, TestingVerdict> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Self {
        if let Err(verdict) = &self {
            update_total_testing_result(pool, submission_id, verdict, 0).await?;
        }
        self
    }
}
