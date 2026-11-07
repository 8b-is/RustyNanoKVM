//! NanoKVM Vision - Video Capture and Streaming
//!
//! This crate provides video capture and encoding functionality
//! for the NanoKVM system, including MJPEG and H.264 streaming.

pub mod capture;
pub mod encoder;
pub mod hdmi;
pub mod screen_ocr;
pub mod ternary_context;

#[cfg(all(target_arch = "riscv64", not(feature = "mock")))]
pub mod ffi;

pub use capture::VideoCapture;
pub use encoder::{Encoder, EncoderType};
pub use hdmi::HdmiInput;
pub use screen_ocr::{
    CONSOLE_COLS, CONSOLE_ROWS, LlmScreenTranscriptBuffer, transcribe_rgb_console_80x25,
};
pub use ternary_context::{
    ScreenSemanticClass, TernaryAudioContext, TernaryMcpContextPacket, TernaryVideoAnalyzer,
};
