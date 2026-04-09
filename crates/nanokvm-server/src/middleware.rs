//! Middleware for authentication, rate limiting, etc.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::state::AppState;

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = state.config();

    // Skip auth if disabled
    if config.is_auth_disabled() {
        drop(config);
        return Ok(next.run(request).await);
    }
    drop(config);

    // Get token from header
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(token) => {
            match state.auth.validate_token(token) {
                Ok(_claims) => {
                    debug!("Token validated successfully");
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
