//! Configuration management for NanoKVM
//!
//! Handles loading and validation of server configuration from
//! `/etc/kvm/server.yaml` with fallback to defaults.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::Result;

/// Global configuration instance
static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// Configuration file path
const CONFIG_PATH: &str = "/etc/kvm/server.yaml";
const CONFIG_DIR: &str = "/etc/kvm";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Protocol to use (http or https)
    pub proto: String,
    /// Port configuration
    pub port: PortConfig,
    /// TLS certificate configuration
    pub cert: CertConfig,
    /// Logger configuration
    pub logger: LoggerConfig,
    /// Authentication mode
    pub authentication: String,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// STUN server address
    pub stun: String,
    /// TURN server configuration
    pub turn: TurnConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Hardware configuration (runtime-determined)
    #[serde(skip)]
    pub hardware: HardwareConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proto: "http".to_string(),
            port: PortConfig::default(),
            cert: CertConfig::default(),
            logger: LoggerConfig::default(),
            authentication: "enable".to_string(),
            jwt: JwtConfig::default(),
            stun: "stun:stun.l.google.com:19302".to_string(),
            turn: TurnConfig::default(),
            security: SecurityConfig::default(),
            hardware: HardwareConfig::default(),
        }
    }
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PortConfig {
    pub http: u16,
    pub https: u16,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            http: 80,
            https: 443,
        }
    }
}

/// TLS certificate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CertConfig {
    pub crt: String,
    pub key: String,
}

impl Default for CertConfig {
    fn default() -> Self {
        Self {
            crt: "/etc/kvm/server.crt".to_string(),
            key: "/etc/kvm/server.key".to_string(),
        }
    }
}

/// Logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggerConfig {
    pub level: String,
    pub file: String,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: "/var/log/nanokvm.log".to_string(),
        }
    }
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct JwtConfig {
    pub secret_key: String,
    pub refresh_token_duration: u64,
    pub revoke_tokens_on_logout: bool,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret_key: uuid::Uuid::new_v4().to_string(),
            refresh_token_duration: 168, // 7 days in hours
            revoke_tokens_on_logout: true,
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TurnConfig {
    pub turn_addr: String,
    pub turn_user: String,
    pub turn_cred: String,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SecurityConfig {
    pub login_lockout_duration: i32,
    pub login_max_failures: i32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            login_lockout_duration: 300, // 5 minutes
            login_max_failures: 5,
        }
    }
}

/// Hardware configuration (determined at runtime)
#[derive(Debug, Clone, Default)]
pub struct HardwareConfig {
    pub version: HardwareVersion,
    pub gpio_reset: String,
    pub gpio_power: String,
    pub gpio_power_led: String,
    pub gpio_hdd_led: String,
}

/// Hardware version enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HardwareVersion {
    #[default]
    Unknown,
    NanoKVMCube,
    NanoKVMPCIe,
    NanoKVMPro,
}

impl Config {
    /// Get the global configuration instance
    pub fn instance() -> &'static RwLock<Config> {
        CONFIG.get_or_init(|| {
            let config = Self::load().unwrap_or_else(|e| {
                warn!("Failed to load config, using defaults: {}", e);
                Config::default()
            });
            RwLock::new(config)
        })
    }

    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_path = Path::new(CONFIG_PATH);

        if config_path.exists() {
            debug!("Loading config from {}", CONFIG_PATH);
            let content = fs::read_to_string(config_path)?;
            let mut config: Config = serde_yaml::from_str(&content)?;
            config.validate()?;
            config.detect_hardware();
            info!("Configuration loaded successfully");
            Ok(config)
        } else {
            info!("Config file not found, creating default configuration");
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        // Create directory if it doesn't exist
        fs::create_dir_all(CONFIG_DIR)?;

        let content = serde_yaml::to_string(self)?;
        fs::write(CONFIG_PATH, content)?;
        info!("Configuration saved to {}", CONFIG_PATH);
        Ok(())
    }

    /// Validate configuration
    fn validate(&mut self) -> Result<()> {
        // Ensure ports are valid
        if self.port.http == 0 || self.port.https == 0 {
            warn!("Invalid port configuration, using defaults");
            self.port = PortConfig::default();
        }

        // Validate protocol
        if self.proto != "http" && self.proto != "https" {
            self.proto = "http".to_string();
        }

        Ok(())
    }

    /// Detect hardware version and GPIO pins
    fn detect_hardware(&mut self) {
        // Read hardware version from device tree or configuration
        // This would be implemented based on actual hardware detection
        self.hardware = HardwareConfig {
            version: HardwareVersion::NanoKVMCube,
            gpio_reset: "504".to_string(),
            gpio_power: "505".to_string(),
            gpio_power_led: "440".to_string(),
            gpio_hdd_led: "441".to_string(),
        };

        debug!("Detected hardware: {:?}", self.hardware.version);
    }

    /// Check if authentication is disabled
    pub fn is_auth_disabled(&self) -> bool {
        self.authentication == "disable"
    }

    /// Check if HTTPS is enabled
    pub fn is_https(&self) -> bool {
        self.proto == "https"
    }
}
