//! Main `ada-judge` backend

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use apalis_redis::RedisStorage;
use api::push_submission_to_queue::push_submission_to_queue;
use app_state::AppState;
use axum::{Router, extract::DefaultBodyLimit, routing::post};
use log::LevelFilter;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

mod api;
mod app_state;

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

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run sqlx migrations");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_pool = apalis_redis::connect(redis_url)
        .await
        .expect("Failed to connect to Redis");

    let backend = RedisStorage::new(redis_pool);

    let app = Router::new()
        .route("/push-submission-to-queue", post(push_submission_to_queue))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(Arc::new(AppState {
            db: pg_pool.clone(),
            apalis_backend: Mutex::new(backend.clone()),
        }));

    let listener = TcpListener::bind("0.0.0.0:4444")
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Failed to serve");
}
