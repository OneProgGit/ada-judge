#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]

use apalis::{
    layers::{WorkerBuilderExt, retry::RetryPolicy},
    prelude::WorkerBuilder,
};
use apalis_redis::RedisStorage;
use sqlx::postgres::PgPoolOptions;
use std::env;
use submissions_judger::test_submission;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let postgres_url =
        env::var("DATABASE_URL").expect("environment variable DATABASE_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&postgres_url)
        .await
        .expect("failed to connect to PostgreSQL");

    tracing::info!("connected to PostgreSQL");

    let redis_url = env::var("REDIS_URL").expect("environment variable REDIS_URL must be set");
    let redis_pool = apalis_redis::connect(redis_url)
        .await
        .expect("failed to connect to Redis");

    let backend = RedisStorage::new(redis_pool);

    tracing::info!("connected to Redis");

    let host_name = env::var("HOSTNAME").expect("environment variable HOSTNAME must be set");

    let worker = WorkerBuilder::new(format!("worker-{host_name}"))
        .backend(backend)
        .data(pg_pool)
        .retry(RetryPolicy::retries(0))
        .concurrency(20)
        .build(test_submission);

    worker.run().await.expect("worker failed");
}
