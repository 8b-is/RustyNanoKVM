//! HDMI input handling

use std::fs;
use std::path::Path;

use tracing::{debug, info, warn};

use nanokvm_core::{Error, Result};

/// HDMI input status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdmiStatus {
    /// HDMI connected and signal detected
    Connected,
    /// HDMI disconnected or no signal
    Disconnected,
    /// Unknown status
    Unknown,
}

/// HDMI resolution information
#[derive(Debug, Clone, Copy)]
pub struct HdmiResolution {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

impl HdmiResolution {
    pub const RES_1080P60: Self = Self {
        width: 1920,
        height: 1080,
        refresh_rate: 60,
    };

    pub const RES_720P60: Self = Self {
        width: 1280,
        height: 720,
        refresh_rate: 60,
    };

    pub const RES_4K30: Self = Self {
        width: 3840,
        height: 2160,
        refresh_rate: 30,
    };

    /// Check if this is a valid resolution
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0 && self.width <= 4096 && self.height <= 2160
    }

    /// Check if resolution is supported by hardware
    pub fn is_supported(&self) -> bool {
        // NanoKVM Cube supports up to 1080p60
        // NanoKVM Pro supports up to 4K30
        self.is_valid() && (self.width <= 1920 && self.height <= 1080)
    }
}

/// HDMI disable status file
const HDMI_DISABLE_FILE: &str = "/etc/kvm/disable_hdmi";

/// HDMI input manager
pub struct HdmiInput {
    enabled: bool,
}

impl HdmiInput {
    /// Create a new HDMI input manager
    pub fn new() -> Self {
        let enabled = !Self::is_disabled_by_config();
        Self { enabled }
    }

    /// Check if HDMI is disabled by configuration file
    pub fn is_disabled_by_config() -> bool {
        Path::new(HDMI_DISABLE_FILE).exists()
    }

    /// Enable HDMI input
    pub fn enable(&mut self) -> Result<()> {
        // Remove disable file if exists
        if Path::new(HDMI_DISABLE_FILE).exists() {
            fs::remove_file(HDMI_DISABLE_FILE)?;
        }

        self.enabled = true;
        info!("HDMI input enabled");
        Ok(())
    }

    /// Disable HDMI input
    pub fn disable(&mut self) -> Result<()> {
        // Create disable file
        fs::write(HDMI_DISABLE_FILE, "")?;

        self.enabled = false;
        info!("HDMI input disabled");
        Ok(())
    }

    /// Check if HDMI input is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current HDMI status
    pub fn status(&self) -> HdmiStatus {
        if !self.enabled {
            return HdmiStatus::Disconnected;
        }

        // In a real implementation, this would query the hardware
        // For now, assume connected if enabled
        HdmiStatus::Connected
    }

    /// Get current resolution
    pub fn resolution(&self) -> Option<HdmiResolution> {
        if self.status() != HdmiStatus::Connected {
            return None;
        }

        // In a real implementation, this would query the hardware
        Some(HdmiResolution::RES_1080P60)
    }

    /// Toggle HDMI power (for reset)
    pub fn toggle(&mut self) -> Result<()> {
        debug!("Toggling HDMI input");
        self.disable()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.enable()?;
        Ok(())
    }
}

impl Default for HdmiInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_constants() {
        assert!(HdmiResolution::RES_1080P60.is_valid());
        assert!(HdmiResolution::RES_1080P60.is_supported());
    }

    #[test]
    fn test_invalid_resolution() {
        let res = HdmiResolution {
            width: 0,
            height: 0,
            refresh_rate: 0,
        };
        assert!(!res.is_valid());
    }

    #[test]
    fn test_hdmi_input_creation() {
        let hdmi = HdmiInput::new();
        // Initial state depends on config file
    }
}
