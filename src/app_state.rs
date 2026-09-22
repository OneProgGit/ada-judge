use aj_models::contests::ContestEvent;
use apalis_redis::RedisStorage;
use dashmap::DashMap;
use models::testing::SubmissionTask;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;

use crate::api::contests::ContestsSubScope;

type ContestsSubsType = DashMap<Option<i64>, (broadcast::Sender<ContestEvent>, CancellationToken)>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Arc<Mutex<RedisStorage<SubmissionTask>>>,
    pub contests_subs: Arc<ContestsSubsType>,
}
