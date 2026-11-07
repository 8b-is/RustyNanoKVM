//! NanoKVM Vision - Video Capture and Streaming
//!
//! This crate provides video capture and encoding functionality
//! for the NanoKVM system, including MJPEG and H.264 streaming.

pub mod capture;
pub mod encoder;
pub mod hdmi;
pub mod ternary_context;

#[cfg(all(target_arch = "riscv64", not(feature = "mock")))]
pub mod ffi;

pub use capture::VideoCapture;
pub use encoder::{Encoder, EncoderType};
pub use hdmi::HdmiInput;
pub use ternary_context::{
    ScreenSemanticClass, TernaryAudioContext, TernaryMcpContextPacket, TernaryVideoAnalyzer,
};
