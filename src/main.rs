use axum::{Router, routing::get};
use solutions_judger::push_submission_to_queue;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/push-submission-to-queue", get(push_submission_to_queue));

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Failed to serve");
}
