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
use std::{env, process, time::Duration};
use tester::test_submission;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn required_env(key: &str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(variable = key, %error, "required environment variable is missing");
            process::exit(1);
        }
    }
}

fn concurrency_from_env() -> usize {
    const DEFAULT: usize = 1;
    env::var("CONCURRENCY").map_or(DEFAULT, |value| {
        value.parse().unwrap_or_else(|error| {
            tracing::warn!(
                variable = "CONCURRENCY",
                %value,
                %error,
                "invalid value, falling back to {DEFAULT}"
            );
            DEFAULT
        })
    })
}

fn restart_delay_from_env() -> Duration {
    const DEFAULT_SECS: u64 = 5;
    env::var("WORKER_RESTART_DELAY_SECS").map_or_else(
        |_| Duration::from_secs(DEFAULT_SECS),
        |value| {
            let secs = value.parse().unwrap_or_else(|error| {
                tracing::warn!(
                    variable = "WORKER_RESTART_DELAY_SECS",
                    %value,
                    %error,
                    "invalid value, falling back to {DEFAULT_SECS}s"
                );
                DEFAULT_SECS
            });
            Duration::from_secs(secs)
        },
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing::info!(%log_level, "starting ada-judge tester");

    let postgres_url = required_env("DATABASE_URL");
    let pg_pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&postgres_url)
        .await
    {
        Ok(pg_pool) => pg_pool,
        Err(error) => {
            tracing::error!(%error, "failed to connect to PostgreSQL");
            process::exit(1);
        }
    };

    tracing::info!("connected to PostgreSQL");

    let redis_url = required_env("REDIS_URL");
    let redis_pool = match apalis_redis::connect(redis_url).await {
        Ok(redis_pool) => redis_pool,
        Err(error) => {
            tracing::error!(%error, "failed to connect to Redis");
            process::exit(1);
        }
    };

    tracing::info!("connected to Redis");

    let host_name = required_env("HOSTNAME");
    let concurrency = concurrency_from_env();
    let restart_delay = restart_delay_from_env();
    let worker_name = format!("worker-{host_name}");
    let backend = RedisStorage::new(redis_pool);

    tracing::info!(
        worker = %worker_name,
        concurrency,
        restart_delay_secs = restart_delay.as_secs(),
        "worker configured"
    );

    // `Worker::run` consumes the worker, so rebuild it to restart after a backend error.
    loop {
        let worker = WorkerBuilder::new(worker_name.clone())
            .backend(backend.clone())
            .data(pg_pool.clone())
            .retry(RetryPolicy::retries(3))
            .concurrency(concurrency)
            .build(test_submission);

        tracing::info!(worker = %worker_name, "worker started");

        match worker.run().await {
            Ok(()) => tracing::warn!(worker = %worker_name, "worker stopped gracefully"),
            Err(error) => {
                tracing::error!(worker = %worker_name, %error, ?error, "worker failed");
            }
        }

        tracing::warn!(
            worker = %worker_name,
            delay_secs = restart_delay.as_secs(),
            "restarting worker"
        );
        tokio::time::sleep(restart_delay).await;
    }
}
