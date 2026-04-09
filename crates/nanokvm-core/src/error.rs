//! Error types for NanoKVM

use thiserror::Error;

/// Result type alias using NanoKVM Error
pub type Result<T> = std::result::Result<T, Error>;

/// NanoKVM error types
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Unauthorized(String),

    /// HID device error
    #[error("HID error: {0}")]
    Hid(String),

    /// GPIO error
    #[error("GPIO error: {0}")]
    Gpio(String),

    /// Video/Vision error
    #[error("Vision error: {0}")]
    Vision(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Hardware error
    #[error("Hardware error: {0}")]
    Hardware(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// HDMI error
    #[error("HDMI error: {0}")]
    Hdmi(String),
}

impl Error {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an auth error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create an unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a HID error
    pub fn hid(msg: impl Into<String>) -> Self {
        Self::Hid(msg.into())
    }

    /// Create a GPIO error
    pub fn gpio(msg: impl Into<String>) -> Self {
        Self::Gpio(msg.into())
    }

    /// Create a vision error
    pub fn vision(msg: impl Into<String>) -> Self {
        Self::Vision(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
