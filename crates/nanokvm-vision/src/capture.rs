//! Video capture from HDMI input
//!
//! Provides abstraction over the vendor-specific video capture APIs.

use std::sync::OnceLock;

use bytes::Bytes;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

use nanokvm_core::{Error, Result};

use crate::encoder::EncoderType;

/// Global video capture instance
static CAPTURE_INSTANCE: OnceLock<RwLock<VideoCapture>> = OnceLock::new();

/// Video capture error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CaptureResult {
    /// Successfully captured MJPEG frame
    Mjpeg = 0,
    /// Successfully captured H.264 SPS
    H264Sps = 1,
    /// Successfully captured H.264 PPS
    H264Pps = 2,
    /// Successfully captured H.264 I-frame
    H264IFrame = 3,
    /// Successfully captured H.264 P-frame
    H264PFrame = 4,
    /// Image not changed since last capture
    NotChanged = 5,
    /// No image available
    NoImage = -1,
    /// Video encoder error
    VencError = -2,
    /// Image buffer full
    BufferFull = -3,
    /// Resolution change in progress
    ResolutionChanging = -4,
    /// Retrieving image, please wait
    Retrieving = -5,
    /// Unsupported resolution
    UnsupportedResolution = -6,
    /// HDMI input resolution error
    HdmiResError = -7,
}

impl From<i32> for CaptureResult {
    fn from(value: i32) -> Self {
        match value {
            0 => CaptureResult::Mjpeg,
            1 => CaptureResult::H264Sps,
            2 => CaptureResult::H264Pps,
            3 => CaptureResult::H264IFrame,
            4 => CaptureResult::H264PFrame,
            5 => CaptureResult::NotChanged,
            -1 => CaptureResult::NoImage,
            -2 => CaptureResult::VencError,
            -3 => CaptureResult::BufferFull,
            -4 => CaptureResult::ResolutionChanging,
            -5 => CaptureResult::Retrieving,
            -6 => CaptureResult::UnsupportedResolution,
            -7 => CaptureResult::HdmiResError,
            _ => CaptureResult::NoImage,
        }
    }
}

/// Captured video frame
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame data
    pub data: Bytes,
    /// Frame type
    pub frame_type: CaptureResult,
    /// Frame width
    pub width: u16,
    /// Frame height
    pub height: u16,
}

/// Video capture manager
pub struct VideoCapture {
    initialized: bool,
    hdmi_enabled: bool,
    current_width: u16,
    current_height: u16,
    gop: u8,
    frame_detect: u8,
}

impl VideoCapture {
    /// Get the global video capture instance
    pub fn instance() -> &'static RwLock<VideoCapture> {
        CAPTURE_INSTANCE.get_or_init(|| {
            let mut capture = VideoCapture::new();
            if let Err(e) = capture.init() {
                error!("Failed to initialize video capture: {}", e);
            }
            RwLock::new(capture)
        })
    }

    /// Create a new video capture instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            hdmi_enabled: false,
            current_width: 1920,
            current_height: 1080,
            gop: 30,
            frame_detect: 1,
        }
    }

    /// Initialize the video capture subsystem
    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // In a real implementation, this would call kvmv_init()
        // For now, we'll use a mock implementation
        #[cfg(feature = "ffi")]
        {
            // FFI call to vendor SDK
            // unsafe { ffi::kvmv_init(0) }
        }

        self.initialized = true;
        info!("Video capture initialized");
        Ok(())
    }

    /// Deinitialize the video capture subsystem
    pub fn deinit(&mut self) {
        if !self.initialized {
            return;
        }

        #[cfg(feature = "ffi")]
        {
            // unsafe { ffi::kvmv_deinit() }
        }

        self.initialized = false;
        info!("Video capture deinitialized");
    }

    /// Read a video frame
    ///
    /// # Arguments
    /// * `width` - Desired output width
    /// * `height` - Desired output height
    /// * `encoder` - Encoder type (MJPEG or H.264)
    /// * `quality` - Quality parameter (50-100 for MJPEG, 500-10000 for H.264)
    pub fn read_frame(
        &self,
        width: u16,
        height: u16,
        encoder: EncoderType,
        quality: u16,
    ) -> Result<VideoFrame> {
        if !self.initialized {
            return Err(Error::vision("Video capture not initialized"));
        }

        if !self.hdmi_enabled {
            return Err(Error::vision("HDMI input disabled"));
        }

        // In a real implementation, this would call kvmv_read_img()
        // For now, return a mock frame
        #[cfg(feature = "mock")]
        {
            return Ok(VideoFrame {
                data: Bytes::from(vec![0xFF, 0xD8, 0xFF, 0xE0]), // JPEG magic bytes
                frame_type: CaptureResult::Mjpeg,
                width,
                height,
            });
        }

        #[cfg(not(feature = "mock"))]
        {
            // Real FFI implementation would go here
            Err(Error::vision("Video capture not available"))
        }
    }

    /// Read MJPEG frame
    pub fn read_mjpeg(&self, width: u16, height: u16, quality: u16) -> Result<VideoFrame> {
        self.read_frame(width, height, EncoderType::Mjpeg, quality)
    }

    /// Read H.264 frame
    pub fn read_h264(&self, width: u16, height: u16, bitrate: u16) -> Result<VideoFrame> {
        self.read_frame(width, height, EncoderType::H264, bitrate)
    }

    /// Enable or disable HDMI input
    pub fn set_hdmi_enabled(&mut self, enabled: bool) -> Result<()> {
        #[cfg(feature = "ffi")]
        {
            // unsafe { ffi::kvmv_hdmi_control(if enabled { 1 } else { 0 }) }
        }

        self.hdmi_enabled = enabled;
        debug!("HDMI input {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Check if HDMI is enabled
    pub fn is_hdmi_enabled(&self) -> bool {
        self.hdmi_enabled
    }

    /// Set H.264 GOP (Group of Pictures) size
    pub fn set_gop(&mut self, gop: u8) {
        self.gop = gop;

        #[cfg(feature = "ffi")]
        {
            // unsafe { ffi::set_h264_gop(gop) }
        }

        debug!("H.264 GOP set to {}", gop);
    }

    /// Set frame detection sensitivity
    pub fn set_frame_detect(&mut self, value: u8) {
        self.frame_detect = value;

        #[cfg(feature = "ffi")]
        {
            // unsafe { ffi::set_frame_detact(value) }
        }

        debug!("Frame detection set to {}", value);
    }

    /// Get current capture resolution
    pub fn current_resolution(&self) -> (u16, u16) {
        (self.current_width, self.current_height)
    }
}

impl Default for VideoCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VideoCapture {
    fn drop(&mut self) {
        self.deinit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_result_conversion() {
        assert_eq!(CaptureResult::from(0), CaptureResult::Mjpeg);
        assert_eq!(CaptureResult::from(-1), CaptureResult::NoImage);
        assert_eq!(CaptureResult::from(3), CaptureResult::H264IFrame);
    }

    #[test]
    fn test_video_capture_creation() {
        let capture = VideoCapture::new();
        assert!(!capture.initialized);
        assert!(!capture.hdmi_enabled);
    }
}
