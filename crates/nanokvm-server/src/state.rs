//! Application state management

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::info;

use nanokvm_core::{Config, Result};
use nanokvm_hid::Hid;
use nanokvm_vision::VideoCapture;

use crate::auth::AuthManager;

/// Shared application state
pub struct AppState {
    /// Configuration
    pub config: &'static RwLock<Config>,
    /// Authentication manager
    pub auth: AuthManager,
    /// HID device manager
    pub hid: &'static Hid,
    /// Video capture manager
    pub video: &'static RwLock<VideoCapture>,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Result<Self> {
        info!("Initializing application state...");

        // Get configuration
        let config = Config::instance();

        // Initialize HID
        let hid = Hid::instance();

        // Initialize video capture
        let video = VideoCapture::instance();

        // Create auth manager
        let auth = AuthManager::new();

        info!("Application state initialized");

        Ok(Self {
            config,
            auth,
            hid,
            video,
        })
    }

    /// Get current configuration
    pub fn config(&self) -> parking_lot::RwLockReadGuard<'_, Config> {
        self.config.read()
    }
}
