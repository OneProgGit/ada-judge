#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::{
    api::{
        auth::{delete_my_account, login, register},
        contests::{
            create_contest, create_contest_post, delete_contest, delete_contest_post,
            get_contest_by_id, get_contest_leaderboard, get_contest_post_by_id, get_contest_posts,
            get_contest_problems, get_contests, get_my_contests, get_problem_by_id, update_contest,
            update_contest_post,
        },
        problems::{
            answer_problem_question, create_problem, create_problem_question, delete_problem,
            delete_problem_question, download_problem, get_all_problem_questions,
            get_my_problem_questions, get_my_problems, get_problem_by_id_admin,
            get_problem_question_by_id, get_problems, update_problem,
        },
        submissions::{
            download_submission, get_all_submissions, get_all_user_submissions,
            get_contest_submissions, get_my_contest_submissions, get_submission,
            retest_problem_submissions,
        },
        users::{
            delete_user_account, get_my_user_profile, get_private_user_profile,
            get_public_user_profile, get_users, update_user_admin_level,
        },
    },
    middleware::{
        auth::Auth,
        contests::{ensure_contest_finished, ensure_contest_started_1, ensure_contest_started_2},
        rights::{require_admin, require_owner},
    },
};
use apalis_redis::RedisStorage;
use api::submissions::submit;
use app_state::AppState;
use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    http::{Method, header},
    routing::{delete, get, patch, post},
};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod app_state;
mod crypt;
mod jwt;
mod middleware;
mod tools;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let postgres_url =
        env::var("POSTGRES_URL").expect("environment variable POSTGRES_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&postgres_url)
        .await
        .expect("failed to connect to PostgreSQL");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("failed to run database migrations");

    tracing::info!("connected to PostgreSQL");

    let redis_url = env::var("REDIS_URL").expect("environment variable REDIS_URL must be set");
    let redis_pool = apalis_redis::connect(redis_url)
        .await
        .expect("failed to connect to Redis");

    let apalis_backend = RedisStorage::new(redis_pool);

    tracing::info!("connected to Redis");

    let state = AppState {
        db: pg_pool.clone(),
        apalis_backend: Arc::new(Mutex::new(apalis_backend.clone())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let contest_problems_routes_1 = Router::new()
        .route("/contests/{contest_id}/problems", get(get_contest_problems))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            ensure_contest_started_1,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let contest_problems_routes_2 = Router::new()
        .route(
            "/contests/{contest_id}/problems/{problem_id}",
            get(get_problem_by_id),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            ensure_contest_started_2,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let leaderboard_routes = Router::new()
        .route(
            "/contests/{contest_id}/leaderboard",
            get(get_contest_leaderboard),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            ensure_contest_finished,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let admin_routes = Router::new()
        .route("/contests/new", post(create_contest))
        .route("/contests/{contest_id}/update", patch(update_contest))
        .route("/contests/{contest_id}/delete", delete(delete_contest))
        .route("/contests/my", get(get_my_contests))
        .route(
            "/contest/{contest_id}/submissions",
            get(get_contest_submissions),
        )
        .route(
            "/contests/{contest_id}/posts/new",
            post(create_contest_post),
        )
        .route(
            "/contests/posts/{post_id}/update",
            patch(update_contest_post),
        )
        .route(
            "/contests/posts/{post_id}/delete",
            delete(delete_contest_post),
        )
        .route("/problems/{problem_id}/delete", delete(delete_problem))
        .route("/problems/my", get(get_my_problems))
        .route("/problems/{problem_id}", get(get_problem_by_id_admin))
        .route(
            "/problems/{problem_id}/retest",
            post(retest_problem_submissions),
        )
        .route("/problems/{problem_id}/download", get(download_problem))
        .route(
            "/problems/{problem_id}/questions",
            get(get_all_problem_questions),
        )
        .route(
            "/problems/questions/{question_id}/answer",
            patch(answer_problem_question),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_admin,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let heavy_problem_routes = Router::new()
        .route("/problems/new", post(create_problem))
        .route("/problems/{problem_id}/update", patch(update_problem))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_admin,
        ))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let owner_routes = Router::new()
        .route("/users", get(get_users))
        .route("/users/{user_id}/private", get(get_private_user_profile))
        .route(
            "/users/{user_id}/update_admin_level",
            patch(update_user_admin_level),
        )
        .route(
            "/users/{user_id}/delete_account",
            delete(delete_user_account),
        )
        .route("/problems", get(get_problems))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_owner,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let default_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/contest/{contest_id}/submissions/my",
            get(get_my_contest_submissions),
        )
        .route("/submissions/{submission_id}", get(get_submission))
        .route(
            "/submissions/{submission_id}/download",
            get(download_submission),
        )
        .route("/users/{user_id}", get(get_public_user_profile))
        .route("/users/me", get(get_my_user_profile))
        .route("/users/me/delete_account", delete(delete_my_account))
        .route("/contests", get(get_contests))
        .route("/contests/{contest_id}", get(get_contest_by_id))
        .route("/contests/{contest_id}/submit", post(submit))
        .route("/contests/{contest_id}/posts", get(get_contest_posts))
        .route("/contests/posts/{post_id}", get(get_contest_post_by_id))
        .route(
            "/problems/{problem_id}/questions/new",
            post(create_problem_question),
        )
        .route(
            "/problems/questions/{question_id}/delete",
            delete(delete_problem_question),
        )
        .route(
            "/problems/{problem_id}/questions/my",
            get(get_my_problem_questions),
        )
        .route(
            "/problems/questions/{question_id}",
            get(get_problem_question_by_id),
        )
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let app = Router::new()
        .merge(default_routes)
        .merge(contest_problems_routes_1)
        .merge(contest_problems_routes_2)
        .merge(leaderboard_routes)
        .merge(admin_routes)
        .merge(heavy_problem_routes)
        .merge(owner_routes)
        .layer(Extension(Auth))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4444")
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("failed to serve application");
}
