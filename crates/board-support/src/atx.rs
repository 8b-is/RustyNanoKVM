//! ATX power control for NanoKVM
//!
//! Controls computer power and reset via GPIO pins connected to
//! the ATX front panel header.

use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
<<<<<<< Updated upstream
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
use tracing::{debug, info};

use nanokvm_core::{Error, Result};

use crate::gpio::{Gpio, Value};
<<<<<<< Updated upstream
=======
=======
use tracing::{debug, info, warn};

use nanokvm_core::{Error, Result};

use crate::gpio::{Direction, Gpio, Value};
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
use tracing::{debug, info};

use nanokvm_core::{Error, Result};

use crate::gpio::{Gpio, Value};
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes

/// Default power button press duration (ms)
const POWER_PRESS_DURATION: u64 = 200;
/// Force power off duration (ms)
const FORCE_POWER_OFF_DURATION: u64 = 6000;
/// Reset button press duration (ms)
const RESET_PRESS_DURATION: u64 = 200;

/// ATX control pins configuration
#[derive(Debug, Clone, Copy)]
pub struct AtxPins {
    /// GPIO pin for power button control
    pub power: u32,
    /// GPIO pin for reset button control
    pub reset: u32,
    /// GPIO pin for power LED input
    pub power_led: u32,
    /// GPIO pin for HDD LED input
    pub hdd_led: u32,
}

impl Default for AtxPins {
    fn default() -> Self {
        // Default pins for NanoKVM Cube
        Self {
            power: 505,
            reset: 504,
            power_led: 440,
            hdd_led: 441,
        }
    }
}

/// ATX power controller
pub struct AtxControl {
    power_gpio: Mutex<Option<Gpio>>,
    reset_gpio: Mutex<Option<Gpio>>,
    power_led_gpio: Mutex<Option<Gpio>>,
    hdd_led_gpio: Mutex<Option<Gpio>>,
    pins: AtxPins,
}

impl AtxControl {
    /// Create a new ATX controller with default pins
    pub fn new() -> Result<Self> {
        Self::with_pins(AtxPins::default())
    }

    /// Create a new ATX controller with custom pin configuration
    pub fn with_pins(pins: AtxPins) -> Result<Self> {
        let controller = Self {
            power_gpio: Mutex::new(None),
            reset_gpio: Mutex::new(None),
            power_led_gpio: Mutex::new(None),
            hdd_led_gpio: Mutex::new(None),
            pins,
        };

        Ok(controller)
    }

    /// Initialize all GPIO pins
    pub fn init(&self) -> Result<()> {
        // Initialize power button GPIO
        if let Ok(mut gpio) = Gpio::new(self.pins.power) {
            gpio.configure_output(Value::High)?; // Active low
            *self.power_gpio.lock() = Some(gpio);
            debug!("Power GPIO initialized on pin {}", self.pins.power);
        }

        // Initialize reset button GPIO
        if let Ok(mut gpio) = Gpio::new(self.pins.reset) {
            gpio.configure_output(Value::High)?; // Active low
            *self.reset_gpio.lock() = Some(gpio);
            debug!("Reset GPIO initialized on pin {}", self.pins.reset);
        }

        // Initialize power LED GPIO
        if let Ok(mut gpio) = Gpio::new(self.pins.power_led) {
            gpio.configure_input()?;
            *self.power_led_gpio.lock() = Some(gpio);
            debug!("Power LED GPIO initialized on pin {}", self.pins.power_led);
        }

        // Initialize HDD LED GPIO
        if let Ok(mut gpio) = Gpio::new(self.pins.hdd_led) {
            gpio.configure_input()?;
            *self.hdd_led_gpio.lock() = Some(gpio);
            debug!("HDD LED GPIO initialized on pin {}", self.pins.hdd_led);
        }

        info!("ATX control initialized");
        Ok(())
    }

    /// Press the power button briefly
    pub fn power_short_press(&self) -> Result<()> {
        self.press_button(&self.power_gpio, Duration::from_millis(POWER_PRESS_DURATION))
    }

    /// Hold the power button to force power off
    pub fn power_long_press(&self) -> Result<()> {
        self.press_button(
            &self.power_gpio,
            Duration::from_millis(FORCE_POWER_OFF_DURATION),
        )
    }

    /// Press the reset button
    pub fn reset(&self) -> Result<()> {
        self.press_button(&self.reset_gpio, Duration::from_millis(RESET_PRESS_DURATION))
    }

    /// Check if computer is powered on (by reading power LED)
    pub fn is_powered_on(&self) -> Result<bool> {
        let guard = self.power_led_gpio.lock();
        if let Some(ref gpio) = *guard {
            Ok(gpio.is_high()?)
        } else {
            Err(Error::gpio("Power LED GPIO not initialized"))
        }
    }

    /// Check if HDD is active
    pub fn is_hdd_active(&self) -> Result<bool> {
        let guard = self.hdd_led_gpio.lock();
        if let Some(ref gpio) = *guard {
            Ok(gpio.is_high()?)
        } else {
            Err(Error::gpio("HDD LED GPIO not initialized"))
        }
    }

    /// Get current power status
    pub fn status(&self) -> AtxStatus {
        AtxStatus {
            power_on: self.is_powered_on().unwrap_or(false),
            hdd_active: self.is_hdd_active().unwrap_or(false),
        }
    }

    /// Helper to press a button (pull low, wait, release)
    fn press_button(&self, gpio: &Mutex<Option<Gpio>>, duration: Duration) -> Result<()> {
        let guard = gpio.lock();
        if let Some(ref gpio) = *guard {
            gpio.set_low()?;
            thread::sleep(duration);
            gpio.set_high()?;
            Ok(())
        } else {
            Err(Error::gpio("GPIO not initialized"))
        }
    }
}

impl Default for AtxControl {
    fn default() -> Self {
        Self::new().expect("Failed to create ATX control")
    }
}

/// ATX status information
#[derive(Debug, Clone, Copy)]
pub struct AtxStatus {
    /// Whether the computer is powered on
    pub power_on: bool,
    /// Whether the HDD LED is active
    pub hdd_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pins() {
        let pins = AtxPins::default();
        assert_eq!(pins.power, 505);
        assert_eq!(pins.reset, 504);
        assert_eq!(pins.power_led, 440);
        assert_eq!(pins.hdd_led, 441);
    }

    #[test]
    fn test_atx_status() {
        let status = AtxStatus {
            power_on: true,
            hdd_active: false,
        };
        assert!(status.power_on);
        assert!(!status.hdd_active);
    }
}
