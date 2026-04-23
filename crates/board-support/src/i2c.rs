//! I2C interface for NanoKVM
//!
//! Provides I2C communication for OLED displays and other peripherals.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

<<<<<<< Updated upstream
use nix::libc;
use tracing::debug;
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
use nix::libc;
use tracing::debug;
=======
use tracing::{debug, error};
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
use nix::libc;
use tracing::debug;
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes

use nanokvm_core::{Error, Result};

/// I2C device path format
const I2C_DEVICE_PATH: &str = "/dev/i2c-";

/// I2C ioctl commands
const I2C_SLAVE: libc::c_ulong = 0x0703;
const I2C_SLAVE_FORCE: libc::c_ulong = 0x0706;

/// I2C bus controller
pub struct I2c {
    file: File,
    #[allow(dead_code)]
    bus: u8,
    current_addr: Option<u8>,
}

impl I2c {
    /// Open an I2C bus
    pub fn open(bus: u8) -> Result<Self> {
        let path = format!("{}{}", I2C_DEVICE_PATH, bus);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| Error::Hardware(format!("Failed to open I2C bus {}: {}", bus, e)))?;

        debug!("Opened I2C bus {}", bus);

        Ok(Self {
            file,
            bus,
            current_addr: None,
        })
    }

    /// Set the slave address for subsequent operations
    pub fn set_slave_address(&mut self, addr: u8) -> Result<()> {
        if self.current_addr == Some(addr) {
            return Ok(());
        }

        let fd = self.file.as_raw_fd();
        let result = unsafe { libc::ioctl(fd, I2C_SLAVE, addr as libc::c_int) };

        if result < 0 {
            // Try force mode if regular mode fails (device might be in use by kernel)
            let result = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_int) };
            if result < 0 {
                return Err(Error::Hardware(format!(
                    "Failed to set I2C slave address {:#04x}",
                    addr
                )));
            }
        }

        self.current_addr = Some(addr);
        debug!("Set I2C slave address to {:#04x}", addr);
        Ok(())
    }

    /// Write data to the current slave
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file
            .write_all(data)
            .map_err(|e| Error::Hardware(format!("I2C write failed: {}", e)))?;
        Ok(())
    }

    /// Read data from the current slave
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self
            .file
            .read(buf)
            .map_err(|e| Error::Hardware(format!("I2C read failed: {}", e)))?;
        Ok(n)
    }

    /// Write data to a specific register
    pub fn write_register(&mut self, reg: u8, data: &[u8]) -> Result<()> {
        let mut buf = Vec::with_capacity(1 + data.len());
        buf.push(reg);
        buf.extend_from_slice(data);
        self.write(&buf)
    }

    /// Read data from a specific register
    pub fn read_register(&mut self, reg: u8, buf: &mut [u8]) -> Result<usize> {
        self.write(&[reg])?;
        self.read(buf)
    }

    /// Write a single byte to a register
    pub fn write_byte(&mut self, reg: u8, value: u8) -> Result<()> {
        self.write(&[reg, value])
    }

    /// Read a single byte from a register
    pub fn read_byte(&mut self, reg: u8) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_register(reg, &mut buf)?;
        Ok(buf[0])
    }

    /// Probe for a device at the given address
    pub fn probe(&mut self, addr: u8) -> bool {
        if self.set_slave_address(addr).is_err() {
            return false;
        }

        // Try to read a byte - if it succeeds, device is present
        let mut buf = [0u8; 1];
        self.read(&mut buf).is_ok()
    }

    /// Scan the bus for devices
    pub fn scan(&mut self) -> Vec<u8> {
        let mut devices = Vec::new();
        for addr in 0x03..=0x77 {
            if self.probe(addr) {
                devices.push(addr);
            }
        }
        devices
    }
}

/// Common I2C addresses
pub mod addresses {
    /// SSD1306 OLED display (128x64)
    pub const SSD1306: u8 = 0x3C;
    /// SSD1306 OLED display (alternate address)
    pub const SSD1306_ALT: u8 = 0x3D;
    /// PCF8574 I/O expander
    pub const PCF8574: u8 = 0x20;
    /// LM75 temperature sensor
    pub const LM75: u8 = 0x48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addresses() {
        assert_eq!(addresses::SSD1306, 0x3C);
        assert_eq!(addresses::SSD1306_ALT, 0x3D);
    }
}
