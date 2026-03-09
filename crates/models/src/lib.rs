use crate::testing::SubmissionTask;
use apalis_postgres::PostgresStorage;
use sqlx::PgPool;
use tokio::sync::Mutex;

pub mod enums;
pub mod testing;

pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Mutex<PostgresStorage<SubmissionTask>>,
}
