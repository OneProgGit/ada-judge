use apalis::prelude::{IntervalStrategy, StrategyBuilder, WorkerBuilder};
use apalis_postgres::{Config, PostgresStorage};
use axum::{Router, routing::post};
use models::AppState;
use solutions_judger::{push_submission_into_queue, test};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to setup a database");

    PostgresStorage::setup(&pool)
        .await
        .expect("Failed to setup a queue");

    let lazy_strategy = StrategyBuilder::new()
        .apply(IntervalStrategy::new(Duration::from_secs(1)))
        .build();
    let config = Config::new("submission").with_poll_interval(lazy_strategy);

    let backend = PostgresStorage::new_with_config(&pool, &config);

    let app = Router::new()
        .route(
            "/push-submission-into-queue",
            post(push_submission_into_queue),
        )
        .with_state(Arc::new(AppState {
            db: pool.clone(),
            apalis_backend: Mutex::new(backend.clone()),
        }));

    let listener = TcpListener::bind("0.0.0.0:4444")
        .await
        .expect("Failed to start server");

    let worker = WorkerBuilder::new("worker")
        .backend(backend)
        .data(pool)
        .build(test);

    tokio::spawn(async move { worker.run().await });
    axum::serve(listener, app).await.expect("Failed to serve");
}
