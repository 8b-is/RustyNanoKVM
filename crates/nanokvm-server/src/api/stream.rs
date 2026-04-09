//! Video streaming API handlers

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tracing::debug;

use crate::state::AppState;

/// Stream parameters
#[derive(Debug, Deserialize)]
pub struct StreamParams {
    /// Image width
    #[serde(default = "default_width")]
    pub width: u16,
    /// Image height
    #[serde(default = "default_height")]
    pub height: u16,
    /// Quality (1-100 for MJPEG)
    #[serde(default = "default_quality")]
    pub quality: u16,
}

fn default_width() -> u16 {
    1920
}
fn default_height() -> u16 {
    1080
}
fn default_quality() -> u16 {
    80
}

/// MJPEG stream handler
pub async fn mjpeg(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StreamParams>,
) -> impl IntoResponse {
    debug!(
        "MJPEG stream requested: {}x{} quality={}",
        params.width, params.height, params.quality
    );

    let video = state.video.read();

    match video.read_mjpeg(params.width, params.height, params.quality) {
        Ok(frame) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .body(Body::from(frame.data.to_vec()))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!(
                r#"{{"code": -1, "msg": "{}"}}"#,
                e
            )))
            .unwrap(),
    }
}

/// Snapshot handler (single JPEG image)
pub async fn snapshot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StreamParams>,
) -> impl IntoResponse {
    debug!(
        "Snapshot requested: {}x{} quality={}",
        params.width, params.height, params.quality
    );

    let video = state.video.read();

    match video.read_mjpeg(params.width, params.height, params.quality) {
        Ok(frame) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"snapshot.jpg\"",
            )
            .body(Body::from(frame.data.to_vec()))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!(
                r#"{{"code": -1, "msg": "{}"}}"#,
                e
            )))
            .unwrap(),
    }
}
