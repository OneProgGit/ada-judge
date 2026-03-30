//! Submissions judger worker for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use apalis::{
    layers::{WorkerBuilderExt, retry::RetryPolicy},
    prelude::WorkerBuilder,
};
use apalis_redis::RedisStorage;
use log::LevelFilter;
use sqlx::postgres::PgPoolOptions;
use std::env;
use submissions_judger::test_submission;

#[tokio::main]
async fn main() {
    let log_modules: [&str; 0] = [];
    pretty_logging::init(LevelFilter::Info, log_modules);

    let postgres_url = env::var("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&postgres_url)
        .await
        .expect("Failed to connect to Postgres");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_pool = apalis_redis::connect(redis_url)
        .await
        .expect("Failed to connect to Redis");

    let backend = RedisStorage::new(redis_pool);

    let worker = WorkerBuilder::new("worker")
        .backend(backend)
        .data(pg_pool)
        .retry(RetryPolicy::retries(0))
        .concurrency(20)
        .build(test_submission);

    worker.run().await.expect("Worker failed");
}
