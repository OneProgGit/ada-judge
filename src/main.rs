use apalis::prelude::WorkerBuilder;
use apalis_redis::RedisStorage;
use axum::{Router, routing::post};
use log::LevelFilter;
use models::AppState;
use solutions_judger::{push_submission_to_queue, test};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

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
        .with_state(Arc::new(AppState {
            db: pg_pool.clone(),
            apalis_backend: Mutex::new(backend.clone()),
        }));

    let listener = TcpListener::bind("0.0.0.0:3333")
        .await
        .expect("Failed to start server");

    let worker = WorkerBuilder::new("worker")
        .backend(backend)
        .data(pg_pool)
        .build(test);

    tokio::spawn(async move { worker.run().await });
    axum::serve(listener, app).await.expect("Failed to serve");
}
