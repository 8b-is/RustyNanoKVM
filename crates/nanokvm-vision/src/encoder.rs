//! Video encoder types and configuration

use nanokvm_core::Result;

/// Video encoder type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderType {
    /// MJPEG encoding
    Mjpeg,
    /// H.264 encoding
    H264,
}

impl EncoderType {
    /// Get the codec type identifier
    pub fn as_u8(&self) -> u8 {
        match self {
            EncoderType::Mjpeg => 0,
            EncoderType::H264 => 1,
        }
    }

    /// Parse from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(EncoderType::Mjpeg),
            1 => Some(EncoderType::H264),
            _ => None,
        }
    }
}

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Encoder type
    pub encoder_type: EncoderType,
    /// Target width
    pub width: u16,
    /// Target height
    pub height: u16,
    /// Quality/bitrate (meaning depends on encoder type)
    pub quality: u16,
    /// Frame rate (FPS)
    pub frame_rate: u8,
    /// GOP size for H.264
    pub gop: u8,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            encoder_type: EncoderType::Mjpeg,
            width: 1920,
            height: 1080,
            quality: 80,
            frame_rate: 30,
            gop: 30,
        }
    }
}

/// Video encoder trait
pub trait Encoder {
    /// Configure the encoder
    fn configure(&mut self, config: &EncoderConfig) -> Result<()>;

    /// Encode a frame
    fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>>;

    /// Get encoder type
    fn encoder_type(&self) -> EncoderType;
}

/// MJPEG encoder settings
#[derive(Debug, Clone)]
pub struct MjpegSettings {
    /// JPEG quality (1-100)
    pub quality: u8,
}

impl Default for MjpegSettings {
    fn default() -> Self {
        Self { quality: 80 }
    }
}

/// H.264 encoder settings
#[derive(Debug, Clone)]
pub struct H264Settings {
    /// Target bitrate in kbps
    pub bitrate: u32,
    /// GOP (Group of Pictures) size
    pub gop: u8,
    /// Use I-frame only mode
    pub iframe_only: bool,
}

impl Default for H264Settings {
    fn default() -> Self {
        Self {
            bitrate: 4000,
            gop: 30,
            iframe_only: false,
        }
    }
}

/// Stream quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    /// Low quality, low bandwidth
    Low,
    /// Medium quality
    Medium,
    /// High quality
    High,
    /// Maximum quality
    Ultra,
}

impl QualityPreset {
    /// Get MJPEG quality for this preset
    pub fn mjpeg_quality(&self) -> u8 {
        match self {
            QualityPreset::Low => 50,
            QualityPreset::Medium => 70,
            QualityPreset::High => 85,
            QualityPreset::Ultra => 95,
        }
    }

    /// Get H.264 bitrate for this preset (kbps)
    pub fn h264_bitrate(&self) -> u32 {
        match self {
            QualityPreset::Low => 1000,
            QualityPreset::Medium => 2500,
            QualityPreset::High => 5000,
            QualityPreset::Ultra => 10000,
        }
    }

    /// Get resolution scale factor
    pub fn scale_factor(&self) -> f32 {
        match self {
            QualityPreset::Low => 0.5,
            QualityPreset::Medium => 0.75,
            QualityPreset::High => 1.0,
            QualityPreset::Ultra => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_type_conversion() {
        assert_eq!(EncoderType::Mjpeg.as_u8(), 0);
        assert_eq!(EncoderType::H264.as_u8(), 1);
        assert_eq!(EncoderType::from_u8(0), Some(EncoderType::Mjpeg));
        assert_eq!(EncoderType::from_u8(2), None);
    }

    #[test]
    fn test_quality_presets() {
        assert_eq!(QualityPreset::Low.mjpeg_quality(), 50);
        assert_eq!(QualityPreset::High.h264_bitrate(), 5000);
    }

    #[test]
    fn test_default_config() {
        let config = EncoderConfig::default();
        assert_eq!(config.encoder_type, EncoderType::Mjpeg);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }
}
