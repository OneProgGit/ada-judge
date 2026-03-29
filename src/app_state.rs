use apalis_redis::RedisStorage;
use models::testing::SubmissionTask;
use sqlx::PgPool;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Mutex<RedisStorage<SubmissionTask>>,
}
