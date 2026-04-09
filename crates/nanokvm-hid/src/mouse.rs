//! Mouse HID emulation
//!
//! Provides mouse input emulation over USB HID gadget.
//! Supports both relative (mouse) and absolute (touchpad) modes.

use tracing::debug;

use nanokvm_core::Result;

use crate::hid::Hid;

/// Mouse report size (relative mode)
const MOUSE_REPORT_SIZE: usize = 4;

/// Touchpad report size (absolute mode)
const TOUCHPAD_REPORT_SIZE: usize = 6;

/// Mouse operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    /// Relative movement (standard mouse)
    Relative,
    /// Absolute positioning (touchpad/touchscreen)
    Absolute,
}

/// Mouse emulator
pub struct Mouse;

impl Mouse {
    /// Move mouse with relative coordinates
    pub fn move_relative(dx: i8, dy: i8) -> Result<()> {
        let report = Self::build_relative_report(0, dx, dy, 0);
        debug!("Mouse move relative: dx={}, dy={}", dx, dy);
        Hid::instance().write_mouse(&report)
    }

    /// Move mouse to absolute position
    ///
    /// Coordinates are in the range 0-32767
    pub fn move_absolute(x: u16, y: u16) -> Result<()> {
        let report = Self::build_absolute_report(0, x, y);
        debug!("Mouse move absolute: x={}, y={}", x, y);
        Hid::instance().write_touchpad(&report)
    }

    /// Click mouse button
    pub fn click(button: MouseButton) -> Result<()> {
        Self::press(button)?;
        Self::release()
    }

    /// Press mouse button (without release)
    pub fn press(button: MouseButton) -> Result<()> {
        let report = Self::build_relative_report(button.to_byte(), 0, 0, 0);
        debug!("Mouse press: {:?}", button);
        Hid::instance().write_mouse(&report)
    }

    /// Release all mouse buttons
    pub fn release() -> Result<()> {
        let report = Self::build_relative_report(0, 0, 0, 0);
        debug!("Mouse release");
        Hid::instance().write_mouse(&report)
    }

    /// Scroll wheel
    pub fn scroll(amount: i8) -> Result<()> {
        let report = Self::build_relative_report(0, 0, 0, amount);
        debug!("Mouse scroll: {}", amount);
        Hid::instance().write_mouse(&report)
    }

    /// Combined mouse action (relative mode)
    pub fn action_relative(buttons: u8, dx: i8, dy: i8, wheel: i8) -> Result<()> {
        let report = Self::build_relative_report(buttons, dx, dy, wheel);
        Hid::instance().write_mouse(&report)
    }

    /// Combined mouse action (absolute mode)
    pub fn action_absolute(buttons: u8, x: u16, y: u16) -> Result<()> {
        let report = Self::build_absolute_report(buttons, x, y);
        Hid::instance().write_touchpad(&report)
    }

    /// Build relative mouse HID report
    ///
    /// Report format:
    /// - Byte 0: Button state
    /// - Byte 1: X displacement (signed)
    /// - Byte 2: Y displacement (signed)
    /// - Byte 3: Wheel (signed)
    fn build_relative_report(buttons: u8, dx: i8, dy: i8, wheel: i8) -> [u8; MOUSE_REPORT_SIZE] {
        [buttons, dx as u8, dy as u8, wheel as u8]
    }

    /// Build absolute mouse HID report
    ///
    /// Report format:
    /// - Byte 0: Button state
    /// - Byte 1: X low byte
    /// - Byte 2: X high byte
    /// - Byte 3: Y low byte
    /// - Byte 4: Y high byte
    /// - Byte 5: Reserved
    fn build_absolute_report(buttons: u8, x: u16, y: u16) -> [u8; TOUCHPAD_REPORT_SIZE] {
        [
            buttons,
            (x & 0xFF) as u8,
            ((x >> 8) & 0xFF) as u8,
            (y & 0xFF) as u8,
            ((y >> 8) & 0xFF) as u8,
            0,
        ]
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl MouseButton {
    /// Convert to button byte value
    pub fn to_byte(self) -> u8 {
        match self {
            MouseButton::Left => 0x01,
            MouseButton::Right => 0x02,
            MouseButton::Middle => 0x04,
            MouseButton::Back => 0x08,
            MouseButton::Forward => 0x10,
        }
    }

    /// Parse from button byte value
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(MouseButton::Left),
            0x02 => Some(MouseButton::Right),
            0x04 => Some(MouseButton::Middle),
            0x08 => Some(MouseButton::Back),
            0x10 => Some(MouseButton::Forward),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_relative_report() {
        let report = Mouse::build_relative_report(0x01, 10, -5, 1);
        assert_eq!(report[0], 0x01); // Left button
        assert_eq!(report[1], 10u8); // dx
        assert_eq!(report[2], (-5i8) as u8); // dy
        assert_eq!(report[3], 1u8); // wheel
    }

    #[test]
    fn test_build_absolute_report() {
        let report = Mouse::build_absolute_report(0x00, 16384, 8192);
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], 0x00); // x low
        assert_eq!(report[2], 0x40); // x high (16384 = 0x4000)
        assert_eq!(report[3], 0x00); // y low
        assert_eq!(report[4], 0x20); // y high (8192 = 0x2000)
    }

    #[test]
    fn test_mouse_button_conversion() {
        assert_eq!(MouseButton::Left.to_byte(), 0x01);
        assert_eq!(MouseButton::from_byte(0x02), Some(MouseButton::Right));
    }
}
