//! NanoKVM Core - Shared types and utilities
//!
//! This crate provides the foundational types, configuration,
//! and error handling for the NanoKVM system.

pub mod config;
pub mod error;
pub mod hardware;
pub mod memnet;

pub use config::Config;
pub use error::{Error, Result};
pub use memnet::{
    MEMNET_AYEOS_PORT, MEMNET_CAPSULE64_MAGIC, MEMNET_ESP32_PORT, MEMNET_FS_PORT, MemnetAddress,
    MemnetCapsule64, MemnetFileChunk, MemnetSearchHit, SharedFileEntry, SharedFolderManifest,
    WaveFrame79, WaveRational, broadcast_memnet_capsule_udp, crc32_ieee, fnv32_1a, fnv64_1a,
    read_memnet_file_chunk, scan_smart_tree_directory, search_smart_tree_files,
};
