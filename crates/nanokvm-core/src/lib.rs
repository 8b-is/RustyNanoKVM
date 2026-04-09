//! NanoKVM Core - Shared types and utilities
//!
//! This crate provides the foundational types, configuration,
//! and error handling for the NanoKVM system.

pub mod config;
pub mod error;
pub mod hardware;

pub use config::Config;
pub use error::{Error, Result};
