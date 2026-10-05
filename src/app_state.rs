use aj_models::{contests::ContestsEvent, problems::ProblemsEvent, users::UsersEvent};
use apalis_redis::RedisStorage;
use dashmap::DashMap;
use models::testing::SubmissionTask;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

type ContestsSubsType = DashMap<Option<i64>, broadcast::Sender<ContestsEvent>>;
type ProblemsSubsType = broadcast::Sender<ProblemsEvent>;
type UsersSubsType = DashMap<Option<i64>, broadcast::Sender<UsersEvent>>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub apalis_backend: Arc<Mutex<RedisStorage<SubmissionTask>>>,
    pub contests_subs: Arc<ContestsSubsType>,
    pub problems_subs: Arc<ProblemsSubsType>,
    pub users_subs: Arc<UsersSubsType>,
}
