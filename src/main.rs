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
        auth::{delete_my_account, login, register},
        contests::{
            create_contest, create_contest_post, delete_contest, delete_contest_post,
            get_contest_by_id, get_contest_leaderboard, get_contest_post_by_id, get_contest_posts,
            get_contest_problems, get_contests, get_my_contests, get_problem_by_id, update_contest,
            update_contest_post,
        },
        problems::{
            answer_problem_question, create_problem, create_problem_question, delete_problem,
            delete_problem_question, get_all_problem_questions, get_my_problem_questions,
            get_my_problems, get_problem_by_id_admin, get_problem_question_by_id, get_problems,
        },
        submissions::{
            download_submission, get_all_my_submissions, get_all_submissions,
            get_all_user_submissions, get_contest_submissions, get_my_contest_submissions,
            get_my_problem_submissions, get_problem_submissions, get_submission,
            get_user_contest_submissions, get_user_problem_submissions, retest_problem_submissions,
        },
        users::{
            change_user_admin_level, delete_user_account, get_my_user_profile,
            get_private_user_profile, get_public_user_profile, get_users,
        },
    },
    middleware::{
        admin::{check_user_is_at_least_admin, check_user_is_owner},
        auth::Auth,
        contests::{
            check_contest_ended, check_contest_started, check_contest_started_2_path_elements,
        },
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
mod tools;

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

    let state = AppState {
        db: pg_pool.clone(),
        apalis_backend: Arc::new(Mutex::new(backend.clone())),
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

    let routes_avaible_after_start_of_contest_1_path_element = Router::new()
        .route("/contests/{contest_id}/problems", get(get_contest_problems))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_started,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let routes_avaible_after_start_of_contest_2_path_elements = Router::new()
        .route(
            "/contests/{contest_id}/problems/{problem_id}",
            get(get_problem_by_id),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_started_2_path_elements,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let routes_avaible_after_end_of_contest = Router::new()
        .route(
            "/contests/{contest_id}/leaderboard",
            get(get_contest_leaderboard),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_contest_ended,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let admin_routes = Router::new()
        .route("/contests/{contest_id}/update", patch(update_contest))
        .route("/contests/new", post(create_contest))
        .route(
            "/submissions/filter/contest/{contest_id}",
            get(get_contest_submissions),
        )
        .route(
            "/submissions/filter/problem/{problem_id}",
            get(get_problem_submissions),
        )
        .route(
            "/submissions/filter/contest/{contest_id}/user/{user_id}",
            get(get_user_contest_submissions),
        )
        .route(
            "/submissions/filter/problem/{problem_id}/user/{user_id}",
            get(get_user_problem_submissions),
        )
        .route(
            "/problems/{problem_id}/retest-submissions",
            post(retest_problem_submissions),
        )
        .route("/problems/my", get(get_my_problems))
        .route("/problems/{problem_id}", get(get_problem_by_id_admin))
        .route("/problems/{problem_id}/delete", delete(delete_problem))
        .route("/contests/my", get(get_my_contests))
        .route("/contests/{contest_id}/delete", delete(delete_contest))
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
        .route("/contests/posts", get(get_contest_posts))
        .route("/contests/posts/{post_id}", get(get_contest_post_by_id))
        .route(
            "/problems/questions/{question_id}/answer",
            patch(answer_problem_question),
        )
        .route(
            "/problems/{problem_id}/questions",
            get(get_all_problem_questions),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_user_is_at_least_admin,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let create_problem_route = Router::new()
        .route("/problems/new", post(create_problem))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_user_is_at_least_admin,
        ))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let owner_routes = Router::new()
        .route("/users", get(get_users))
        .route("/users/{user_id}/private", get(get_private_user_profile))
        .route(
            "/users/{user_id}/delete_account",
            delete(delete_user_account),
        )
        .route(
            "/users/{user_id}/change_admin_level",
            patch(change_user_admin_level),
        )
        .route("/submissions", get(get_all_submissions))
        .route(
            "/submissions/filter/user/{user_id}",
            get(get_all_user_submissions),
        )
        .route("/problems", get(get_problems))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            check_user_is_owner,
        ))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024));

    let default_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/submissions/{submission_id}", get(get_submission))
        .route(
            "/submissions/{submission_id}/download",
            get(download_submission),
        )
        .route("/submissions/my", get(get_all_my_submissions))
        .route(
            "/submissions/my/filter/contest/{contest_id}",
            get(get_my_contest_submissions),
        )
        .route(
            "/submissions/my/filter/problem/{problem_id}",
            get(get_my_problem_submissions),
        )
        .route("/users/{user_id}", get(get_public_user_profile))
        .route("/users/me", get(get_my_user_profile))
        .route("/users/me/delete_account", delete(delete_my_account))
        .route("/contests", get(get_contests))
        .route("/contests/{contest_id}", get(get_contest_by_id))
        .route("/contests/{contest_id}/submit", post(submit))
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
        .merge(routes_avaible_after_start_of_contest_1_path_element)
        .merge(routes_avaible_after_start_of_contest_2_path_elements)
        .merge(routes_avaible_after_end_of_contest)
        .merge(admin_routes)
        .merge(create_problem_route)
        .merge(owner_routes)
        .layer(Extension(Auth))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4444")
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Failed to serve");
}
