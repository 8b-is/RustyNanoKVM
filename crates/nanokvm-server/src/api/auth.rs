//! Authentication API handlers

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::state::AppState;

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<TokenData>,
}

#[derive(Debug, Serialize)]
pub struct TokenData {
    pub token: String,
    pub refresh_token: String,
}

/// Login handler
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    debug!("Login attempt for user: {}", req.username);

    // Check if auth is disabled
    let config = state.config();
    if config.is_auth_disabled() {
        return (
            StatusCode::OK,
            Json(LoginResponse {
                code: 0,
                msg: "Authentication disabled".to_string(),
                data: Some(TokenData {
                    token: "disabled".to_string(),
                    refresh_token: "disabled".to_string(),
                }),
            }),
        );
    }
    drop(config);

    match state.auth.login(&req.username, &req.password) {
        Ok(auth) => (
            StatusCode::OK,
            Json(LoginResponse {
                code: 0,
                msg: "Success".to_string(),
                data: Some(TokenData {
                    token: auth.access_token,
                    refresh_token: auth.refresh_token,
                }),
            }),
        ),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                code: -1,
                msg: e.to_string(),
                data: None,
            }),
        ),
    }
}

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

/// Logout handler
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LogoutRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.auth.logout(&req.token);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "Logged out"
        })),
    )
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Refresh token handler
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    match state.auth.refresh_token(&req.refresh_token) {
        Ok(auth) => (
            StatusCode::OK,
            Json(LoginResponse {
                code: 0,
                msg: "Success".to_string(),
                data: Some(TokenData {
                    token: auth.access_token,
                    refresh_token: auth.refresh_token,
                }),
            }),
        ),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                code: -1,
                msg: e.to_string(),
                data: None,
            }),
        ),
    }
}

/// Change password request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}

/// Change password handler
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChangePasswordRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state
        .auth
        .change_password(&req.username, &req.old_password, &req.new_password)
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "code": 0,
                "msg": "Password changed"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": -1,
                "msg": e.to_string()
            })),
        ),
    }
}
