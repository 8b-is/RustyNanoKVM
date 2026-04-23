//! HID (Human Interface Device) API handlers

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tracing::debug;

use nanokvm_hid::{Keyboard, Mouse};

use crate::state::AppState;

/// Keyboard input request
#[derive(Debug, Deserialize)]
pub struct KeyboardRequest {
    /// Modifier keys (bitfield)
    pub modifier: u8,
    /// Key codes
    pub keys: Vec<u8>,
}

/// Keyboard handler
pub async fn keyboard(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<KeyboardRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!(
        "Keyboard input: modifier={:#04x}, keys={:?}",
        req.modifier, req.keys
    );

    match Keyboard::press(req.modifier, &req.keys) {
        Ok(_) => {
            // Release keys after a brief moment
            let _ = Keyboard::release();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "code": 0,
                    "msg": "OK"
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": -1,
                "msg": e.to_string()
            })),
        ),
    }
}

/// Mouse input request
#[derive(Debug, Deserialize)]
pub struct MouseRequest {
    /// Mouse mode (relative or absolute)
    #[serde(default)]
    pub mode: MouseModeRequest,
    /// Button state
    pub button: u8,
    /// X coordinate/movement
    pub x: i32,
    /// Y coordinate/movement
    pub y: i32,
    /// Scroll wheel
    #[serde(default)]
    pub wheel: i8,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseModeRequest {
    #[default]
    Relative,
    Absolute,
}

/// Mouse handler
pub async fn mouse(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MouseRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!(
        "Mouse input: mode={:?}, button={}, x={}, y={}, wheel={}",
        req.mode, req.button, req.x, req.y, req.wheel
    );

    let result = match req.mode {
        MouseModeRequest::Relative => {
            // Clamp to i8 range
            let dx = req.x.clamp(-127, 127) as i8;
            let dy = req.y.clamp(-127, 127) as i8;
            Mouse::action_relative(req.button, dx, dy, req.wheel)
        }
        MouseModeRequest::Absolute => {
            // Absolute coordinates (0-32767)
            let x = (req.x.clamp(0, 32767)) as u16;
            let y = (req.y.clamp(0, 32767)) as u16;
            Mouse::action_absolute(req.button, x, y)
        }
    };

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "code": 0,
                "msg": "OK"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": -1,
                "msg": e.to_string()
            })),
        ),
    }
}

/// Paste text request
#[derive(Debug, Deserialize)]
pub struct PasteRequest {
    /// Text to paste
    pub text: String,
}

/// Paste text handler (types text as keyboard input)
pub async fn paste(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PasteRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("Paste request: {} characters", req.text.len());

    match Keyboard::type_string(&req.text) {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "code": 0,
                "msg": "OK"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": -1,
                "msg": e.to_string()
            })),
        ),
    }
}
