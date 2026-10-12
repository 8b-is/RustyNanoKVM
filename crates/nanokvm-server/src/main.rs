//! NanoKVM Server - Main entry point
//!
//! HTTP/WebSocket server for remote KVM access.

use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tracing::{error, info};

mod api;
mod auth;
mod logger;
mod middleware;
mod state;
mod transport;
mod websocket;

use state::AppState;

/// Server version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Initialize logging
    logger::init();

    info!("NanoKVM Server v{}", VERSION);

    // Read a snapshot without holding the global lock during certificate I/O.
    let config = match nanokvm_core::Config::try_instance() {
        Ok(config) => config.read().clone(),
        Err(error) => {
            error!("Server configuration failed: {}", error);
            std::process::exit(1);
        }
    };
    let transport = match transport::PreparedTransport::prepare(
        &config.proto,
        config.port.http,
        config.port.https,
        &config.cert.crt,
        &config.cert.key,
    )
    .await
    {
        Ok(transport) => transport,
        Err(error) => {
            error!("Server transport initialization failed: {}", error);
            std::process::exit(1);
        }
    };

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

    info!(
        "Starting {} server on {}",
        config.proto,
        transport.address()
    );
    if let Err(error) = transport.serve(app, shutdown_signal()).await {
        error!("Server failed: {}", error);
        std::process::exit(1);
    }

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
        .nest("/api", api::router(state.clone()))
        .nest(
            "/ws",
            websocket::router().route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                middleware::websocket_auth_middleware,
            )),
        )
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
