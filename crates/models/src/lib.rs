use crate::testing::SubmissionTask;
use apalis_redis::RedisStorage;
use sqlx::PgPool;
use tokio::sync::Mutex;

pub mod error;
pub mod testing;
pub mod verdicts;

pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Mutex<RedisStorage<SubmissionTask>>,
}
