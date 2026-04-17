//! Main `ada-judge` backend

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use crate::{
    api::{
        auth::{delete_account, login, register},
        contests::{
            create_contest, get_contest_by_id, get_contest_leaderboard, get_contest_problems,
            get_contests, get_problem_by_id,
        },
        submissions::{
            get_all_user_submisssions, get_contest_user_submissions, get_problem_user_submissions,
        },
        user_profiles::{get_private_user_profile, get_public_user_profile},
    },
    middleware::{
        auth::Auth,
        contests::{
            check_contest_ended, check_contest_started, check_contest_started_2_path_elements,
            check_contest_started_and_not_ended,
        },
    },
};
use apalis_redis::RedisStorage;
use api::submissions::push_submission_to_queue;
use app_state::AppState;
use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    http::{Method, header},
    routing::{delete, get, post},
};
use log::LevelFilter;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};

mod api;
mod app_state;
mod crypt;
mod jwt;
mod middleware;

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

    let state = Arc::new(AppState {
        db: pg_pool.clone(),
        apalis_backend: Mutex::new(backend.clone()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let routes_avaible_after_start_of_contest_1_path_element = Router::new()
        .route("/contests/{contest_id}/problems", get(get_contest_problems))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_started,
        ));
    let routes_avaible_after_start_of_contest_2_path_elements = Router::new()
        .route(
            "/contests/{contest_id}/problems/{problem_id}",
            get(get_problem_by_id),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_started_2_path_elements,
        ));

    let routes_avaible_during_the_contest = Router::new()
        .route(
            "/contests/{contest_id}/push-submission-to-queue",
            post(push_submission_to_queue),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_started_and_not_ended,
        ));

    let routes_avaible_after_end_of_contest = Router::new()
        .route(
            "/contests/{contest_id}/leaderboard",
            get(get_contest_leaderboard),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_ended,
        ));

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/submissions/my", get(get_all_user_submisssions))
        .route(
            "/submissions/my/by_contest/{contest_id}",
            get(get_contest_user_submissions),
        )
        .route(
            "/submissions/my/by_problem/{problem_id}",
            get(get_problem_user_submissions),
        )
        .route("/users/{user_id}", get(get_public_user_profile))
        .route("/users/me", get(get_private_user_profile))
        .route("/users/me/delete_account", delete(delete_account))
        .route("/contests", get(get_contests))
        .route("/contests/{contest_id}", get(get_contest_by_id))
        .route("/contests/new", post(create_contest))
        .merge(routes_avaible_after_start_of_contest_1_path_element)
        .merge(routes_avaible_after_start_of_contest_2_path_elements)
        .merge(routes_avaible_during_the_contest)
        .merge(routes_avaible_after_end_of_contest)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(Extension(Auth))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4444")
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Failed to serve");
}
