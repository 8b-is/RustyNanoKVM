//! Board Support Package for NanoKVM
//!
//! This crate provides low-level hardware access for the NanoKVM
//! platform, including GPIO, I2C, OLED display, and ATX control.

pub mod gpio;
pub mod i2c;
pub mod oled;
pub mod atx;
pub mod qr;

pub use gpio::Gpio;
pub use i2c::I2c;
pub use oled::OledDisplay;
pub use atx::AtxControl;
