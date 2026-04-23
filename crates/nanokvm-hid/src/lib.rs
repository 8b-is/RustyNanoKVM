//! NanoKVM HID - Human Interface Device Emulation
//!
//! This crate provides HID gadget functionality for emulating
//! keyboard, mouse, and touchpad devices.

pub mod hid;
pub mod keyboard;
pub mod keycodes;
pub mod mouse;

pub use hid::Hid;
pub use keyboard::Keyboard;
pub use mouse::{Mouse, MouseMode};
