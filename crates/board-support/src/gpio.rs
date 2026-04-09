//! GPIO control for NanoKVM
//!
//! Provides GPIO access using the sysfs interface and memory-mapped I/O
//! for high-performance operations.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use tracing::{debug, warn};

use nanokvm_core::Result;

/// GPIO sysfs base path
const GPIO_SYSFS_PATH: &str = "/sys/class/gpio";

/// GPIO pin direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::Input => "in",
            Direction::Output => "out",
        }
    }
}

/// GPIO pin value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Low,
    High,
}

impl Value {
    fn as_str(&self) -> &'static str {
        match self {
            Value::Low => "0",
            Value::High => "1",
        }
    }

    fn from_str(s: &str) -> Self {
        if s.trim().starts_with('1') {
            Value::High
        } else {
            Value::Low
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Value::High
        } else {
            Value::Low
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> bool {
        matches!(value, Value::High)
    }
}

/// GPIO pin controller
pub struct Gpio {
    pin: u32,
    exported: bool,
}

impl Gpio {
    /// Create a new GPIO pin controller
    pub fn new(pin: u32) -> Result<Self> {
        let gpio = Self {
            pin,
            exported: false,
        };
        Ok(gpio)
    }

    /// Export the GPIO pin for userspace access
    pub fn export(&mut self) -> Result<()> {
        if self.is_exported() {
            debug!("GPIO {} already exported", self.pin);
            return Ok(());
        }

        let export_path = format!("{}/export", GPIO_SYSFS_PATH);
        let mut file = OpenOptions::new().write(true).open(&export_path)?;

        file.write_all(self.pin.to_string().as_bytes())?;
        self.exported = true;
        debug!("GPIO {} exported", self.pin);
        Ok(())
    }

    /// Unexport the GPIO pin
    pub fn unexport(&mut self) -> Result<()> {
        if !self.is_exported() {
            return Ok(());
        }

        let unexport_path = format!("{}/unexport", GPIO_SYSFS_PATH);
        let mut file = OpenOptions::new().write(true).open(&unexport_path)?;

        file.write_all(self.pin.to_string().as_bytes())?;
        self.exported = false;
        debug!("GPIO {} unexported", self.pin);
        Ok(())
    }

    /// Check if GPIO is exported
    pub fn is_exported(&self) -> bool {
        let gpio_path = self.gpio_path();
        gpio_path.exists()
    }

    /// Set pin direction
    pub fn set_direction(&self, direction: Direction) -> Result<()> {
        let direction_path = self.gpio_path().join("direction");
        let mut file = OpenOptions::new().write(true).open(&direction_path)?;

        file.write_all(direction.as_str().as_bytes())?;
        debug!("GPIO {} direction set to {:?}", self.pin, direction);
        Ok(())
    }

    /// Get pin direction
    pub fn get_direction(&self) -> Result<Direction> {
        let direction_path = self.gpio_path().join("direction");
        let mut content = String::new();
        let mut file = File::open(&direction_path)?;
        file.read_to_string(&mut content)?;

        if content.trim() == "in" {
            Ok(Direction::Input)
        } else {
            Ok(Direction::Output)
        }
    }

    /// Set pin value
    pub fn set_value(&self, value: Value) -> Result<()> {
        let value_path = self.gpio_path().join("value");
        let mut file = OpenOptions::new().write(true).open(&value_path)?;

        file.write_all(value.as_str().as_bytes())?;
        debug!("GPIO {} value set to {:?}", self.pin, value);
        Ok(())
    }

    /// Get pin value
    pub fn get_value(&self) -> Result<Value> {
        let value_path = self.gpio_path().join("value");
        let mut content = String::new();
        let mut file = File::open(&value_path)?;
        file.read_to_string(&mut content)?;

        Ok(Value::from_str(&content))
    }

    /// Set pin high
    pub fn set_high(&self) -> Result<()> {
        self.set_value(Value::High)
    }

    /// Set pin low
    pub fn set_low(&self) -> Result<()> {
        self.set_value(Value::Low)
    }

    /// Check if pin is high
    pub fn is_high(&self) -> Result<bool> {
        Ok(self.get_value()? == Value::High)
    }

    /// Check if pin is low
    pub fn is_low(&self) -> Result<bool> {
        Ok(self.get_value()? == Value::Low)
    }

    /// Toggle pin value
    pub fn toggle(&self) -> Result<()> {
        let current = self.get_value()?;
        match current {
            Value::Low => self.set_high(),
            Value::High => self.set_low(),
        }
    }

    /// Configure as output and set initial value
    pub fn configure_output(&mut self, initial: Value) -> Result<()> {
        self.export()?;
        self.set_direction(Direction::Output)?;
        self.set_value(initial)?;
        Ok(())
    }

    /// Configure as input
    pub fn configure_input(&mut self) -> Result<()> {
        self.export()?;
        self.set_direction(Direction::Input)?;
        Ok(())
    }

    fn gpio_path(&self) -> PathBuf {
        PathBuf::from(format!("{}/gpio{}", GPIO_SYSFS_PATH, self.pin))
    }
}

impl Drop for Gpio {
    fn drop(&mut self) {
        if self.exported {
            if let Err(e) = self.unexport() {
                warn!("Failed to unexport GPIO {}: {}", self.pin, e);
            }
        }
    }
}

/// Batch GPIO operations helper
pub struct GpioBatch {
    pins: Vec<Gpio>,
}

impl GpioBatch {
    /// Create a new batch with the given pins
    pub fn new(pin_numbers: &[u32]) -> Result<Self> {
        let mut pins = Vec::with_capacity(pin_numbers.len());
        for &pin in pin_numbers {
            pins.push(Gpio::new(pin)?);
        }
        Ok(Self { pins })
    }

    /// Export all pins
    pub fn export_all(&mut self) -> Result<()> {
        for pin in &mut self.pins {
            pin.export()?;
        }
        Ok(())
    }

    /// Set all pins as outputs with initial low value
    pub fn configure_all_output(&mut self) -> Result<()> {
        for pin in &mut self.pins {
            pin.configure_output(Value::Low)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_str() {
        assert_eq!(Direction::Input.as_str(), "in");
        assert_eq!(Direction::Output.as_str(), "out");
    }

    #[test]
    fn test_value_conversion() {
        assert_eq!(Value::from(true), Value::High);
        assert_eq!(Value::from(false), Value::Low);
        assert!(bool::from(Value::High));
        assert!(!bool::from(Value::Low));
    }

    #[test]
    fn test_value_from_str() {
        assert_eq!(Value::from_str("1"), Value::High);
        assert_eq!(Value::from_str("0"), Value::Low);
        assert_eq!(Value::from_str("1\n"), Value::High);
    }
}
