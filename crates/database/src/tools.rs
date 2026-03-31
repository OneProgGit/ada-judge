//! Database map tools

use crate::update_total_testing_result;
use models::verdicts::TotalVerdict;
use sqlx::PgPool;

/// Extension updating total verdict if `Self` is `Err(TotalVerdict)`
#[allow(async_fn_in_trait)]
pub trait MapDbExt<T> {
    /// Updates total verdict if `Self` is `Err(TotalVerdict)`
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TotalVerdict>;
}

impl<T: Send> MapDbExt<T> for Result<T, TotalVerdict> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Self {
        if let Err(verdict) = &self {
            log::error!("Error verdict: {verdict}");
            update_total_testing_result(pool, submission_id, verdict, 0).await?;
        }
        self
    }
}
