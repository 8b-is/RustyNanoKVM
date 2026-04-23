//! OLED display driver for NanoKVM
//!
//! Supports SSD1306-based 128x64 OLED displays over I2C.

<<<<<<< Updated upstream
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
use tracing::{debug, info};

use nanokvm_core::Result;

<<<<<<< Updated upstream
=======
use crate::i2c::{I2c, addresses};
=======
use tracing::{debug, error, info};
=======
use tracing::{debug, info};
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)

use nanokvm_core::Result;

>>>>>>> Stashed changes
use crate::i2c::{addresses, I2c};

/// OLED display width
pub const DISPLAY_WIDTH: u32 = 128;
/// OLED display height
pub const DISPLAY_HEIGHT: u32 = 64;
/// Frame buffer size (1 bit per pixel)
pub const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize) / 8;

/// SSD1306 commands
mod cmd {
    pub const SET_CONTRAST: u8 = 0x81;
    pub const DISPLAY_ALL_ON_RESUME: u8 = 0xA4;
    pub const DISPLAY_ALL_ON: u8 = 0xA5;
    pub const NORMAL_DISPLAY: u8 = 0xA6;
    pub const INVERT_DISPLAY: u8 = 0xA7;
    pub const DISPLAY_OFF: u8 = 0xAE;
    pub const DISPLAY_ON: u8 = 0xAF;
    pub const SET_DISPLAY_OFFSET: u8 = 0xD3;
    pub const SET_COM_PINS: u8 = 0xDA;
    pub const SET_VCOM_DETECT: u8 = 0xDB;
    pub const SET_DISPLAY_CLOCK_DIV: u8 = 0xD5;
    pub const SET_PRECHARGE: u8 = 0xD9;
    pub const SET_MULTIPLEX: u8 = 0xA8;
    pub const SET_LOW_COLUMN: u8 = 0x00;
    pub const SET_HIGH_COLUMN: u8 = 0x10;
    pub const SET_START_LINE: u8 = 0x40;
    pub const MEMORY_MODE: u8 = 0x20;
    pub const COLUMN_ADDR: u8 = 0x21;
    pub const PAGE_ADDR: u8 = 0x22;
    pub const COM_SCAN_INC: u8 = 0xC0;
    pub const COM_SCAN_DEC: u8 = 0xC8;
    pub const SEG_REMAP: u8 = 0xA0;
    pub const CHARGE_PUMP: u8 = 0x8D;
}

/// Control byte for commands
const CONTROL_CMD: u8 = 0x00;
/// Control byte for data
const CONTROL_DATA: u8 = 0x40;

/// OLED display driver
pub struct OledDisplay {
    i2c: I2c,
    address: u8,
    framebuffer: [u8; FRAMEBUFFER_SIZE],
    width: u32,
    height: u32,
}

impl OledDisplay {
    /// Create a new OLED display on the given I2C bus
    pub fn new(bus: u8) -> Result<Self> {
        let mut i2c = I2c::open(bus)?;
        let address = addresses::SSD1306;

        i2c.set_slave_address(address)?;

        let mut display = Self {
            i2c,
            address,
            framebuffer: [0u8; FRAMEBUFFER_SIZE],
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
        };

        display.init()?;
        info!("OLED display initialized");

        Ok(display)
    }

    /// Initialize the display
    fn init(&mut self) -> Result<()> {
        // Initialization sequence for SSD1306 128x64
        let init_cmds: &[u8] = &[
            cmd::DISPLAY_OFF,
            cmd::SET_DISPLAY_CLOCK_DIV,
            0x80,
            cmd::SET_MULTIPLEX,
            0x3F, // 64 lines
            cmd::SET_DISPLAY_OFFSET,
            0x00,
            cmd::SET_START_LINE | 0x00,
            cmd::CHARGE_PUMP,
            0x14, // Enable charge pump
            cmd::MEMORY_MODE,
            0x00, // Horizontal addressing mode
            cmd::SEG_REMAP | 0x01,
            cmd::COM_SCAN_DEC,
            cmd::SET_COM_PINS,
            0x12,
            cmd::SET_CONTRAST,
            0xCF,
            cmd::SET_PRECHARGE,
            0xF1,
            cmd::SET_VCOM_DETECT,
            0x40,
            cmd::DISPLAY_ALL_ON_RESUME,
            cmd::NORMAL_DISPLAY,
            cmd::DISPLAY_ON,
        ];

        for &cmd in init_cmds {
            self.send_command(cmd)?;
        }

        self.clear()?;
        debug!("Display initialized with {}x{}", self.width, self.height);
        Ok(())
    }

    /// Send a command to the display
    fn send_command(&mut self, cmd: u8) -> Result<()> {
        self.i2c.write(&[CONTROL_CMD, cmd])
    }

    /// Send multiple commands
    fn send_commands(&mut self, cmds: &[u8]) -> Result<()> {
        for &cmd in cmds {
            self.send_command(cmd)?;
        }
        Ok(())
    }

    /// Send data to the display
    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        // Send in chunks to avoid I2C buffer limitations
        const CHUNK_SIZE: usize = 16;

        for chunk in data.chunks(CHUNK_SIZE) {
            let mut buf = Vec::with_capacity(chunk.len() + 1);
            buf.push(CONTROL_DATA);
            buf.extend_from_slice(chunk);
            self.i2c.write(&buf)?;
        }
        Ok(())
    }

    /// Clear the display
    pub fn clear(&mut self) -> Result<()> {
        self.framebuffer.fill(0);
        self.flush()
    }

    /// Fill the display with a solid color
    pub fn fill(&mut self, on: bool) -> Result<()> {
        self.framebuffer.fill(if on { 0xFF } else { 0x00 });
        self.flush()
    }

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: u32, y: u32, on: bool) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = (x + (y / 8) * self.width) as usize;
        let bit = y % 8;

        if on {
            self.framebuffer[index] |= 1 << bit;
        } else {
            self.framebuffer[index] &= !(1 << bit);
        }
    }

    /// Get a pixel from the framebuffer
    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        let index = (x + (y / 8) * self.width) as usize;
        let bit = y % 8;

        (self.framebuffer[index] & (1 << bit)) != 0
    }

    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x: u32, y: u32, width: u32, on: bool) {
        for i in 0..width {
            self.set_pixel(x + i, y, on);
        }
    }

    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: u32, y: u32, height: u32, on: bool) {
        for i in 0..height {
            self.set_pixel(x, y + i, on);
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, on: bool) {
        self.draw_hline(x, y, width, on);
        self.draw_hline(x, y + height - 1, width, on);
        self.draw_vline(x, y, height, on);
        self.draw_vline(x + width - 1, y, height, on);
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, on: bool) {
        for dy in 0..height {
            self.draw_hline(x, y + dy, width, on);
        }
    }

    /// Display text at position (basic 6x8 font)
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if cursor_x + 6 > self.width {
                break;
            }
            self.draw_char(cursor_x, y, ch);
            cursor_x += 6;
        }
    }

    /// Draw a single character (basic 6x8 font)
    fn draw_char(&mut self, x: u32, y: u32, ch: char) {
        // Very basic bitmap font - just draw a placeholder box for now
        // A real implementation would use a proper font table
        let code = ch as u8;
        if code >= 32 && code < 127 {
            // Draw character box
            self.draw_rect(x, y, 5, 7, true);
        }
    }

    /// Flush the framebuffer to the display
    pub fn flush(&mut self) -> Result<()> {
        // Set column and page addresses
        self.send_commands(&[
            cmd::COLUMN_ADDR,
            0,
            (self.width - 1) as u8,
            cmd::PAGE_ADDR,
            0,
            (self.height / 8 - 1) as u8,
        ])?;

<<<<<<< Updated upstream
        // Copy framebuffer to send (to avoid borrowing issues)
        let data = self.framebuffer;
        self.send_data(&data)?;
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
        // Copy framebuffer to send (to avoid borrowing issues)
        let data = self.framebuffer;
        self.send_data(&data)?;
=======
        // Send framebuffer data
        self.send_data(&self.framebuffer)?;
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
        // Copy framebuffer to send (to avoid borrowing issues)
        let data = self.framebuffer;
        self.send_data(&data)?;
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
        Ok(())
    }

    /// Set display contrast (0-255)
    pub fn set_contrast(&mut self, contrast: u8) -> Result<()> {
        self.send_commands(&[cmd::SET_CONTRAST, contrast])
    }

    /// Turn display on
    pub fn display_on(&mut self) -> Result<()> {
        self.send_command(cmd::DISPLAY_ON)
    }

    /// Turn display off
    pub fn display_off(&mut self) -> Result<()> {
        self.send_command(cmd::DISPLAY_OFF)
    }

    /// Invert display colors
    pub fn invert(&mut self, invert: bool) -> Result<()> {
        self.send_command(if invert {
            cmd::INVERT_DISPLAY
        } else {
            cmd::NORMAL_DISPLAY
        })
    }

    /// Get display dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get raw framebuffer reference
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Set raw framebuffer data
    pub fn set_framebuffer(&mut self, data: &[u8]) {
        let len = data.len().min(FRAMEBUFFER_SIZE);
        self.framebuffer[..len].copy_from_slice(&data[..len]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_size() {
        assert_eq!(FRAMEBUFFER_SIZE, 1024); // 128 * 64 / 8
    }
}
