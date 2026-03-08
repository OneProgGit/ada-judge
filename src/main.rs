use apalis_postgres::PostgresStorage;
use axum::{Router, routing::post};
use dotenvy::dotenv;
use models::AppState;
use solutions_judger::push_submission_to_queue;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() {
    dotenv().expect("Invalid env");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to setup a database");

    PostgresStorage::setup(&pool)
        .await
        .expect("Failed to setup a queue");

    let backend = PostgresStorage::new(&pool);

    let app = Router::new()
        .route("/push-submission-to-queue", post(push_submission_to_queue))
        .with_state(Arc::new(AppState {
            db: pool,
            apalis_backend: Mutex::new(backend),
        }));

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Failed to serve");
}
