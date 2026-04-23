//! Core HID device management
//!
//! Manages the Linux USB Gadget HID devices for keyboard and mouse emulation.

use std::fs::{File, OpenOptions};
use std::io::Write;
<<<<<<< Updated upstream
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
=======
use std::os::unix::io::AsRawFd;
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
use std::sync::OnceLock;
use std::time::Duration;

use parking_lot::Mutex;
use tracing::{debug, error, warn};

use nanokvm_core::{Error, Result};

/// HID device paths
const HID0_PATH: &str = "/dev/hidg0"; // Keyboard
const HID1_PATH: &str = "/dev/hidg1"; // Mouse (Relative Mode)
const HID2_PATH: &str = "/dev/hidg2"; // Touchpad (Absolute Mode)

/// Write timeout for HID devices
const WRITE_TIMEOUT: Duration = Duration::from_millis(8);

/// Global HID instance
static HID_INSTANCE: OnceLock<Hid> = OnceLock::new();

/// HID device manager
pub struct Hid {
    keyboard: Mutex<Option<File>>,
    mouse: Mutex<Option<File>>,
    touchpad: Mutex<Option<File>>,
}

impl Hid {
    /// Get the global HID instance
    pub fn instance() -> &'static Hid {
        HID_INSTANCE.get_or_init(|| {
            let hid = Hid::new();
            if let Err(e) = hid.open() {
                warn!("Failed to open HID devices: {}", e);
            }
            hid
        })
    }

    /// Create a new HID manager
    pub fn new() -> Self {
        Self {
            keyboard: Mutex::new(None),
            mouse: Mutex::new(None),
            touchpad: Mutex::new(None),
        }
    }

    /// Open all HID devices
    pub fn open(&self) -> Result<()> {
        self.close();

        // Open keyboard device
        match Self::open_device(HID0_PATH) {
            Ok(file) => *self.keyboard.lock() = Some(file),
            Err(e) => error!("Failed to open keyboard device: {}", e),
        }

        // Open mouse device
        match Self::open_device(HID1_PATH) {
            Ok(file) => *self.mouse.lock() = Some(file),
            Err(e) => error!("Failed to open mouse device: {}", e),
        }

        // Open touchpad device
        match Self::open_device(HID2_PATH) {
            Ok(file) => *self.touchpad.lock() = Some(file),
            Err(e) => error!("Failed to open touchpad device: {}", e),
        }

        debug!("HID devices opened");
        Ok(())
    }

    /// Close all HID devices
    pub fn close(&self) {
        if let Some(file) = self.keyboard.lock().take() {
            let _ = file.sync_all();
        }
        if let Some(file) = self.mouse.lock().take() {
            let _ = file.sync_all();
        }
        if let Some(file) = self.touchpad.lock().take() {
            let _ = file.sync_all();
        }
        debug!("HID devices closed");
    }

    /// Write to keyboard device (HID0)
    pub fn write_keyboard(&self, data: &[u8]) -> Result<()> {
        self.write_to_device(&self.keyboard, HID0_PATH, data)
    }

    /// Write to mouse device (HID1 - relative mode)
    pub fn write_mouse(&self, data: &[u8]) -> Result<()> {
        self.write_to_device(&self.mouse, HID1_PATH, data)
    }

    /// Write to touchpad device (HID2 - absolute mode)
    pub fn write_touchpad(&self, data: &[u8]) -> Result<()> {
        self.write_to_device(&self.touchpad, HID2_PATH, data)
    }

    fn open_device(path: &str) -> Result<File> {
        OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| Error::hid(format!("Failed to open {}: {}", path, e)))
    }

    fn write_to_device(&self, device: &Mutex<Option<File>>, path: &str, data: &[u8]) -> Result<()> {
        let mut guard = device.lock();

        // Re-open if device is closed
        if guard.is_none() {
            match Self::open_device(path) {
                Ok(file) => *guard = Some(file),
                Err(e) => return Err(e),
            }
        }

        if let Some(ref mut file) = *guard {
            // Set write deadline using raw fd operations
            Self::set_write_timeout(file, WRITE_TIMEOUT)?;

            match file.write_all(data) {
                Ok(_) => {
                    debug!("Write to {}: {:?}", path, data);
                    Ok(())
                }
                Err(e) => {
                    error!("Write to {} failed: {}", path, e);

                    // Reopen on error
                    *guard = None;

                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        warn!("Write to {} timed out", path);
                        return Ok(()); // Timeout is not fatal
                    }

                    Err(Error::hid(format!("Write failed: {}", e)))
                }
            }
        } else {
            Err(Error::hid("Device not open"))
        }
    }

    #[cfg(unix)]
<<<<<<< Updated upstream
    fn set_write_timeout(_file: &File, _timeout: Duration) -> Result<()> {
        // Note: Setting socket options on device files may not work
        // The timeout is handled at the application level
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    fn set_write_timeout(_file: &File, _timeout: Duration) -> Result<()> {
        // Note: Setting socket options on device files may not work
        // The timeout is handled at the application level
=======
    fn set_write_timeout(file: &File, timeout: Duration) -> Result<()> {
        use std::os::unix::io::AsRawFd;

        let fd = file.as_raw_fd();
        let tv = libc::timeval {
            tv_sec: timeout.as_secs() as libc::time_t,
            tv_usec: timeout.subsec_micros() as libc::suseconds_t,
        };

        let result = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_SNDTIMEO,
                &tv as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as libc::socklen_t,
            )
        };

        if result < 0 {
            // Socket options may not work on device files, that's okay
            debug!("Could not set write timeout on HID device");
        }

>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
    fn set_write_timeout(_file: &File, _timeout: Duration) -> Result<()> {
        // Note: Setting socket options on device files may not work
        // The timeout is handled at the application level
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_write_timeout(_file: &File, _timeout: Duration) -> Result<()> {
        Ok(())
    }
}

impl Default for Hid {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Hid {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_creation() {
        let hid = Hid::new();
        assert!(hid.keyboard.lock().is_none());
        assert!(hid.mouse.lock().is_none());
        assert!(hid.touchpad.lock().is_none());
    }
}
