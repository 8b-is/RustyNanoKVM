//! Board Support Package for NanoKVM
//!
//! This crate provides low-level hardware access for the NanoKVM
//! platform, including GPIO, I2C, OLED display, and ATX control.

pub mod atx;
pub mod gpio;
pub mod i2c;
pub mod oled;
pub mod qr;

pub use atx::AtxControl;
pub use gpio::Gpio;
pub use i2c::I2c;
pub use oled::OledDisplay;
