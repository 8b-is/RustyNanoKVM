//! API routes module

pub mod auth;
pub mod hid;
pub mod stream;
pub mod storage;
pub mod vm;
pub mod network;
pub mod application;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

/// Create the API router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Auth routes
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/password", post(auth::change_password))
        // Application routes
        .route("/application/info", get(application::info))
        .route("/application/version", get(application::version))
        // VM routes
        .route("/vm/info", get(vm::info))
        .route("/vm/gpio", post(vm::gpio_control))
        .route("/vm/power/short", post(vm::power_short))
        .route("/vm/power/long", post(vm::power_long))
        .route("/vm/reset", post(vm::reset))
        .route("/vm/screen", get(vm::screen))
        .route("/vm/oled", post(vm::oled_control))
        // HID routes
        .route("/hid/keyboard", post(hid::keyboard))
        .route("/hid/mouse", post(hid::mouse))
        .route("/hid/paste", post(hid::paste))
        // Stream routes
        .route("/stream/mjpeg", get(stream::mjpeg))
        .route("/stream/snapshot", get(stream::snapshot))
        // Storage routes
        .route("/storage/images", get(storage::list_images))
        .route("/storage/upload", post(storage::upload))
        // Network routes
        .route("/network/ip", get(network::get_ip))
        .route("/network/hostname", get(network::get_hostname))
}
