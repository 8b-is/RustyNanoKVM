//! NanoKVM Vision - Video Capture and Streaming
//!
//! This crate provides video capture and encoding functionality
//! for the NanoKVM system, including MJPEG and H.264 streaming.

pub mod capture;
pub mod encoder;
pub mod hdmi;

#[cfg(not(feature = "mock"))]
pub mod ffi;

pub use capture::VideoCapture;
pub use encoder::{Encoder, EncoderType};
pub use hdmi::HdmiInput;
