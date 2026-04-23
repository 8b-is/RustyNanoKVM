//! Storage API handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use serde::Serialize;
use tracing::debug;

use crate::state::AppState;

/// Image list response
#[derive(Debug, Serialize)]
pub struct ImageListResponse {
    pub code: i32,
    pub data: Vec<ImageInfo>,
}

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
}

/// List ISO images
pub async fn list_images(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<ImageListResponse>) {
    debug!("Listing images");

    // In a real implementation, this would scan the images directory
    (
        StatusCode::OK,
        Json(ImageListResponse {
            code: 0,
            data: vec![],
        }),
    )
}

/// Upload file handler
pub async fn upload(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<serde_json::Value>) {
    debug!("File upload started");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap_or_default();

        debug!(
            "Received file: {} ({}) - {} bytes",
            file_name,
            name,
            data.len()
        );

        // In a real implementation, this would save the file to disk
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "msg": "Upload complete"
        })),
    )
}
