//! API routes module

pub mod application;
pub mod auth;
pub mod hid;
pub mod network;
pub mod storage;
pub mod stream;
pub mod vm;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

/// Create the API router
pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // Auth routes
        .route("/auth/account", get(auth::account))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/password", post(auth::change_password))
        .route("/auth/password", get(auth::password_status))
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
        .route_layer(axum::middleware::from_fn_with_state(state, crate::middleware::auth_middleware))
        // Login and refresh authenticate their own request bodies.
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
}
