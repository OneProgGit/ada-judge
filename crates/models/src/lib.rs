use apalis_postgres::PostgresStorage;
use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;

use crate::testing::Submission;

pub mod enums;
pub mod testing;

pub struct AppState {
    pub db: Pool<Postgres>,
    pub apalis_backend: Mutex<PostgresStorage<Submission>>,
}
