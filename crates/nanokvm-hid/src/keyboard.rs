//! Keyboard HID emulation
//!
//! Provides keyboard input emulation over USB HID gadget.

use tracing::debug;

use nanokvm_core::Result;

<<<<<<< Updated upstream
=======
<<<<<<< HEAD
<<<<<<< HEAD
=======
use crate::hid::Hid;
use crate::keycodes::KeyCode;

>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
>>>>>>> Stashed changes
/// Keyboard report size
const KEYBOARD_REPORT_SIZE: usize = 8;

/// Maximum simultaneous keys
const MAX_KEYS: usize = 6;

/// Keyboard emulator
pub struct Keyboard;

impl Keyboard {
    /// Send a key press event
    pub fn press(modifier: u8, keys: &[u8]) -> Result<()> {
        let report = Self::build_report(modifier, keys);
        debug!("Keyboard press: modifier={:#04x}, keys={:?}", modifier, keys);
<<<<<<< Updated upstream
        crate::hid::Hid::instance().write_keyboard(&report)
=======
<<<<<<< HEAD
        Hid::instance().write_keyboard(&report)
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
        crate::hid::Hid::instance().write_keyboard(&report)
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
>>>>>>> Stashed changes
    }

    /// Release all keys
    pub fn release() -> Result<()> {
        let report = [0u8; KEYBOARD_REPORT_SIZE];
        debug!("Keyboard release");
<<<<<<< Updated upstream
        crate::hid::Hid::instance().write_keyboard(&report)
=======
<<<<<<< HEAD
<<<<<<< HEAD
        crate::hid::Hid::instance().write_keyboard(&report)
=======
        Hid::instance().write_keyboard(&report)
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
        crate::hid::Hid::instance().write_keyboard(&report)
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
>>>>>>> Stashed changes
    }

    /// Send a key press and release (tap)
    pub fn tap(modifier: u8, key: u8) -> Result<()> {
        Self::press(modifier, &[key])?;
        Self::release()
    }

    /// Type a string by converting characters to key codes
    pub fn type_string(text: &str) -> Result<()> {
        for ch in text.chars() {
<<<<<<< Updated upstream
            if let Some((modifier, key)) = crate::keycodes::key_from_char(ch) {
=======
<<<<<<< HEAD
<<<<<<< HEAD
            if let Some((modifier, key)) = crate::keycodes::key_from_char(ch) {
=======
            if let Some((modifier, key)) = KeyCode::from_char(ch) {
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
            if let Some((modifier, key)) = crate::keycodes::key_from_char(ch) {
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
>>>>>>> Stashed changes
                Self::tap(modifier, key)?;
                // Small delay between keystrokes would be handled by caller
            }
        }
        Ok(())
    }

    /// Build a keyboard HID report
    ///
    /// Report format:
    /// - Byte 0: Modifier keys
    /// - Byte 1: Reserved (0x00)
    /// - Bytes 2-7: Key codes (up to 6 simultaneous keys)
    fn build_report(modifier: u8, keys: &[u8]) -> [u8; KEYBOARD_REPORT_SIZE] {
        let mut report = [0u8; KEYBOARD_REPORT_SIZE];
        report[0] = modifier;
        // report[1] is reserved

        for (i, &key) in keys.iter().take(MAX_KEYS).enumerate() {
            report[2 + i] = key;
        }

        report
    }
}

/// Modifier key flags
pub mod modifiers {
    pub const NONE: u8 = 0x00;
    pub const LEFT_CTRL: u8 = 0x01;
    pub const LEFT_SHIFT: u8 = 0x02;
    pub const LEFT_ALT: u8 = 0x04;
    pub const LEFT_GUI: u8 = 0x08;
    pub const RIGHT_CTRL: u8 = 0x10;
    pub const RIGHT_SHIFT: u8 = 0x20;
    pub const RIGHT_ALT: u8 = 0x40;
    pub const RIGHT_GUI: u8 = 0x80;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_report() {
        let report = Keyboard::build_report(modifiers::LEFT_SHIFT, &[0x04]); // 'a' with shift
        assert_eq!(report[0], modifiers::LEFT_SHIFT);
        assert_eq!(report[1], 0x00);
        assert_eq!(report[2], 0x04);
        assert_eq!(report[3], 0x00);
    }

    #[test]
    fn test_build_report_multiple_keys() {
        let report = Keyboard::build_report(0x00, &[0x04, 0x05, 0x06]);
        assert_eq!(report[2], 0x04);
        assert_eq!(report[3], 0x05);
        assert_eq!(report[4], 0x06);
    }
}
