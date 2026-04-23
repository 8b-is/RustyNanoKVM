//! NanoKVM Server - Main entry point
//!
//! HTTP/WebSocket server for remote KVM access.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tracing::{error, info};

mod api;
mod auth;
mod logger;
mod middleware;
mod state;
mod websocket;

use state::AppState;

/// Server version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Initialize logging
    logger::init();

    info!("NanoKVM Server v{}", VERSION);

    // Initialize application state
    let state = match AppState::new() {
        Ok(s) => Arc::new(s),
        Err(e) => {
            error!("Failed to initialize application state: {}", e);
            std::process::exit(1);
        }
    };

    // Create router with all routes
    let app = create_router(state.clone());

    // Get server address from config
    let http_port = nanokvm_core::Config::instance().read().port.http;
    let addr: SocketAddr = format!("0.0.0.0:{}", http_port)
        .parse()
        .expect("Invalid server address");

    info!("Starting server on {}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Server shutdown complete");
}

/// Create the application router
fn create_router(state: Arc<AppState>) -> Router {
    use tower_http::{
        cors::{Any, CorsLayer},
        services::ServeDir,
        trace::TraceLayer,
    };

    let config = nanokvm_core::Config::instance().read();

    // CORS configuration
    let cors = if config.is_auth_disabled() {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Static file serving for web UI
    let static_files = ServeDir::new("web").not_found_service(ServeDir::new("web/index.html"));

    Router::new()
        .nest("/api", api::router())
        .nest("/ws", websocket::router())
        .fallback_service(static_files)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
