//! Video streaming API handlers

use std::sync::Arc;
use std::time::Duration;

use async_stream::stream;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde::Deserialize;
use tokio::time::sleep;
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

/// MJPEG stream handler (HTTP multipart stream)
pub async fn mjpeg(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StreamParams>,
) -> impl IntoResponse {
    debug!(
        "MJPEG stream requested: {}x{} quality={}",
        params.width, params.height, params.quality
    );

    let stream = stream! {
        let mut last_frame = Vec::new();
        loop {
            let frame_res = {
                let video = state.video.read();
                video.read_mjpeg(params.width, params.height, params.quality)
            };

            match frame_res {
                Ok(frame) => {
                    if !frame.data.is_empty() {
                        last_frame = frame.data.to_vec();
                    }
                }
                Err(e) => {
                    debug!("Error reading MJPEG frame: {}", e);
                }
            }

            if !last_frame.is_empty() {
                let header_str = format!(
                    "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                    last_frame.len()
                );
                yield Ok::<_, std::convert::Infallible>(Bytes::from(header_str));
                yield Ok(Bytes::from(last_frame.clone()));
                yield Ok(Bytes::from("\r\n"));
            }

            sleep(Duration::from_millis(33)).await; // ~30 FPS
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "multipart/x-mixed-replace; boundary=frame",
        )
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from_stream(stream))
        .unwrap()
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
            .body(Body::from(format!(r#"{{"code": -1, "msg": "{}"}}"#, e)))
            .unwrap(),
    }
}
