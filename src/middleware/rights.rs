use crate::middleware::auth::Auth;
use aj_models::users::AdminLevel;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn require_admin(Auth(auth): Auth, req: Request, next: Next) -> Response {
    if auth.admin_level < AdminLevel::Admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(req).await
}

pub async fn require_owner(Auth(auth): Auth, req: Request, next: Next) -> Response {
    if auth.admin_level != AdminLevel::Owner {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(req).await
}
