use std::sync::Arc;

use apalis_redis::RedisStorage;
use models::testing::SubmissionTask;
use sqlx::PgPool;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Arc<Mutex<RedisStorage<SubmissionTask>>>,
}
