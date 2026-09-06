use aj_models::contests::ContestEvent;
use apalis_redis::RedisStorage;
use dashmap::DashMap;
use models::testing::SubmissionTask;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Arc<Mutex<RedisStorage<SubmissionTask>>>,
    pub contests_subs: DashMap<i64, broadcast::Sender<ContestEvent>>,
    //pub questions_subs: DashMap<(Option<i64>, i64), broadcast::Sender<QuestionEvent>>,
}
