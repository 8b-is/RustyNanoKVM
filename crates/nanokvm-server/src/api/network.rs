//! Network API handlers

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use tracing::debug;

use crate::state::AppState;

/// IP address response
#[derive(Debug, Serialize)]
pub struct IpResponse {
    pub code: i32,
    pub data: IpData,
}

#[derive(Debug, Serialize)]
pub struct IpData {
    pub ip: String,
    pub mac: String,
}

/// Get IP address
pub async fn get_ip(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<IpResponse>) {
    debug!("Getting IP address");

    // In a real implementation, this would query network interfaces
    (
        StatusCode::OK,
        Json(IpResponse {
            code: 0,
            data: IpData {
                ip: "192.168.1.100".to_string(),
                mac: "00:00:00:00:00:00".to_string(),
            },
        }),
    )
}

/// Hostname response
#[derive(Debug, Serialize)]
pub struct HostnameResponse {
    pub code: i32,
    pub data: String,
}

/// Get hostname
pub async fn get_hostname(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<HostnameResponse>) {
    debug!("Getting hostname");

    let hostname = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "nanokvm".to_string())
        .trim()
        .to_string();

    (
        StatusCode::OK,
        Json(HostnameResponse {
            code: 0,
            data: hostname,
        }),
    )
}
