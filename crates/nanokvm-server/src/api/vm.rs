//! VM control API handlers

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::state::AppState;

/// VM info response
#[derive(Debug, Serialize)]
pub struct VmInfo {
    pub code: i32,
    pub data: VmInfoData,
}

#[derive(Debug, Serialize)]
pub struct VmInfoData {
    pub power_led: bool,
    pub hdd_led: bool,
    pub resolution: String,
    pub hdmi_connected: bool,
}

/// Get VM info
pub async fn info(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<VmInfo>) {
    // In a real implementation, this would query actual hardware
    (
        StatusCode::OK,
        Json(VmInfo {
            code: 0,
            data: VmInfoData {
                power_led: true,
                hdd_led: false,
                resolution: "1920x1080".to_string(),
                hdmi_connected: true,
            },
        }),
    )
}

/// GPIO control request
#[derive(Debug, Deserialize)]
pub struct GpioRequest {
    pub pin: u32,
    pub value: bool,
}

/// GPIO control handler
pub async fn gpio_control(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GpioRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("GPIO control: pin={}, value={}", req.pin, req.value);

    // In a real implementation, this would control actual GPIO
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "OK"
        })),
    )
}

/// Power short press handler
pub async fn power_short(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("Power short press");

    // In a real implementation, this would trigger ATX power
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "Power button pressed"
        })),
    )
}

/// Power long press handler (force off)
pub async fn power_long(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("Power long press (force off)");

    // In a real implementation, this would hold ATX power for 6 seconds
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "Force power off"
        })),
    )
}

/// Reset button handler
pub async fn reset(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    debug!("Reset button pressed");

    // In a real implementation, this would trigger ATX reset
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "Reset triggered"
        })),
    )
}

/// Screen info response
#[derive(Debug, Serialize)]
pub struct ScreenInfo {
    pub code: i32,
    pub data: ScreenData,
}

#[derive(Debug, Serialize)]
pub struct ScreenData {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

/// Get screen info
pub async fn screen(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<ScreenInfo>) {
    (
        StatusCode::OK,
        Json(ScreenInfo {
            code: 0,
            data: ScreenData {
                width: 1920,
                height: 1080,
                fps: 60,
            },
        }),
    )
}

/// OLED control request
#[derive(Debug, Deserialize)]
pub struct OledRequest {
    /// Enable or disable OLED
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Text to display
    #[serde(default)]
    pub text: Option<String>,
    /// Show QR code
    #[serde(default)]
    pub qr: Option<String>,
}

/// OLED control handler
pub async fn oled_control(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OledRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("OLED control: {:?}", req);

    // In a real implementation, this would control the OLED display
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "OK"
        })),
    )
}
