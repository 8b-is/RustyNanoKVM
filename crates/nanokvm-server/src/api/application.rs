//! Application info API handlers

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::debug;

use crate::VERSION;
use crate::state::AppState;

/// Application info response
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub code: i32,
    pub data: AppInfo,
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub hardware: String,
    pub authentication: bool,
}

/// Get application info
pub async fn info(State(state): State<Arc<AppState>>) -> (StatusCode, Json<InfoResponse>) {
    debug!("Getting application info");

    let config = state.config();

    (
        StatusCode::OK,
        Json(InfoResponse {
            code: 0,
            data: AppInfo {
                name: "NanoKVM".to_string(),
                version: VERSION.to_string(),
                hardware: "SG2002".to_string(),
                authentication: !config.is_auth_disabled(),
            },
        }),
    )
}

/// Version response
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub code: i32,
    pub data: String,
}

/// Get version
pub async fn version(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<VersionResponse>) {
    (
        StatusCode::OK,
        Json(VersionResponse {
            code: 0,
            data: VERSION.to_string(),
        }),
    )
}
