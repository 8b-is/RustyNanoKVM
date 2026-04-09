//! Hardware abstraction for NanoKVM
//!
//! Provides types and utilities for different hardware versions.

use std::path::Path;

use tracing::info;

/// Hardware version detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareVersion {
    /// NanoKVM Cube (standard version)
    Cube,
    /// NanoKVM PCIe (internal mount version)
    PCIe,
    /// NanoKVM Pro (high-performance version)
    Pro,
    /// Unknown or unsupported hardware
    Unknown,
}

impl HardwareVersion {
    /// Detect the current hardware version
    pub fn detect() -> Self {
        // Check device tree or board identification files
        if Path::new("/sys/firmware/devicetree/base/model").exists() {
            if let Ok(model) = std::fs::read_to_string("/sys/firmware/devicetree/base/model") {
                return Self::from_model(&model);
            }
        }

        // Check for NanoKVM-specific identification
        if Path::new("/etc/kvm/hw_version").exists() {
            if let Ok(version) = std::fs::read_to_string("/etc/kvm/hw_version") {
                return Self::from_version_file(&version);
            }
        }

        info!("Unable to detect hardware version, assuming Cube");
        HardwareVersion::Cube
    }

    fn from_model(model: &str) -> Self {
        let model_lower = model.to_lowercase();
        if model_lower.contains("pcie") {
            HardwareVersion::PCIe
        } else if model_lower.contains("pro") {
            HardwareVersion::Pro
        } else if model_lower.contains("cube") || model_lower.contains("nanokvm") {
            HardwareVersion::Cube
        } else {
            HardwareVersion::Unknown
        }
    }

    fn from_version_file(version: &str) -> Self {
        match version.trim() {
            "cube" | "CUBE" => HardwareVersion::Cube,
            "pcie" | "PCIE" => HardwareVersion::PCIe,
            "pro" | "PRO" => HardwareVersion::Pro,
            _ => HardwareVersion::Unknown,
        }
    }

    /// Get GPIO pin configuration for this hardware version
    pub fn gpio_config(&self) -> GpioConfig {
        match self {
            HardwareVersion::Cube => GpioConfig {
                reset_pin: 504,
                power_pin: 505,
                power_led_pin: 440,
                hdd_led_pin: 441,
            },
            HardwareVersion::PCIe => GpioConfig {
                reset_pin: 504,
                power_pin: 505,
                power_led_pin: 508,
                hdd_led_pin: 509,
            },
            HardwareVersion::Pro => GpioConfig {
                reset_pin: 100,
                power_pin: 101,
                power_led_pin: 102,
                hdd_led_pin: 103,
            },
            HardwareVersion::Unknown => GpioConfig::default(),
        }
    }

    /// Check if this is a high-performance variant
    pub fn is_high_performance(&self) -> bool {
        matches!(self, HardwareVersion::Pro)
    }

    /// Get maximum supported resolution
    pub fn max_resolution(&self) -> (u32, u32) {
        match self {
            HardwareVersion::Pro => (3840, 2160), // 4K
            _ => (1920, 1080),                     // 1080p
        }
    }
}

/// GPIO pin configuration
#[derive(Debug, Clone, Copy)]
pub struct GpioConfig {
    pub reset_pin: u32,
    pub power_pin: u32,
    pub power_led_pin: u32,
    pub hdd_led_pin: u32,
}

impl Default for GpioConfig {
    fn default() -> Self {
        Self {
            reset_pin: 504,
            power_pin: 505,
            power_led_pin: 440,
            hdd_led_pin: 441,
        }
    }
}

/// Screen resolution information
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const RES_1080P: Self = Self {
        width: 1920,
        height: 1080,
    };
    pub const RES_720P: Self = Self {
        width: 1280,
        height: 720,
    };
    pub const RES_4K: Self = Self {
        width: 3840,
        height: 2160,
    };
    pub const RES_2K: Self = Self {
        width: 2560,
        height: 1440,
    };
}

/// Display type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayType {
    /// 0.96" 128x64 OLED
    Oled128x64,
    /// 1.47" 320x172 LCD
    Lcd320x172,
    /// No display attached
    None,
}

impl DisplayType {
    /// Get display dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            DisplayType::Oled128x64 => (128, 64),
            DisplayType::Lcd320x172 => (320, 172),
            DisplayType::None => (0, 0),
        }
    }
}
