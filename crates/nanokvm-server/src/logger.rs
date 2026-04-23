//! Logging initialization

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize logging subsystem
pub fn init() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,nanokvm=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true).with_thread_ids(false))
        .init();
}
