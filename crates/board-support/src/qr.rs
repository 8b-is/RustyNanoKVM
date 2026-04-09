//! QR code generation for NanoKVM
//!
//! Generates QR codes for OLED display, typically showing
//! the device IP address for easy access.

use image::{GrayImage, Luma};
use qrcode::QrCode;
use tracing::debug;

use nanokvm_core::Result;

/// Generate a QR code as a grayscale image
pub fn generate_qr(data: &str, size: u32) -> Result<GrayImage> {
    let code = QrCode::new(data.as_bytes())
        .map_err(|e| nanokvm_core::Error::internal(format!("QR generation failed: {}", e)))?;

    let qr_image = code.render::<Luma<u8>>().quiet_zone(false).build();

    // Scale to requested size
    let scaled = image::imageops::resize(
        &qr_image,
        size,
        size,
        image::imageops::FilterType::Nearest,
    );

    debug!("Generated QR code for '{}' at {}x{}", data, size, size);
    Ok(scaled)
}

/// Generate QR code as a 1-bit bitmap for OLED display
pub fn generate_qr_bitmap(data: &str, target_width: u32, target_height: u32) -> Result<Vec<u8>> {
    let size = target_width.min(target_height);
    let qr = generate_qr(data, size)?;

    // Convert to 1-bit format suitable for OLED
    // OLED format: 8 vertical pixels per byte, horizontal pages
    let pages = (target_height + 7) / 8;
    let mut bitmap = vec![0u8; (target_width * pages) as usize];

    let x_offset = (target_width - size) / 2;
    let y_offset = (target_height - size) / 2;

    for y in 0..size {
        for x in 0..size {
            let pixel = qr.get_pixel(x, y);
            if pixel[0] < 128 {
                // Dark pixel (QR code foreground)
                let display_x = x + x_offset;
                let display_y = y + y_offset;
                let page = display_y / 8;
                let bit = display_y % 8;
                let idx = (display_x + page * target_width) as usize;
                if idx < bitmap.len() {
                    bitmap[idx] |= 1 << bit;
                }
            }
        }
    }

    Ok(bitmap)
}

/// Generate a QR code showing a URL
pub fn generate_url_qr(host: &str, port: u16) -> Result<GrayImage> {
    let url = if port == 80 || port == 443 {
        format!("http://{}", host)
    } else {
        format!("http://{}:{}", host, port)
    };
    generate_qr(&url, 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr() {
        let result = generate_qr("test", 64);
        assert!(result.is_ok());
        let img = result.unwrap();
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_generate_qr_bitmap() {
        let result = generate_qr_bitmap("test", 128, 64);
        assert!(result.is_ok());
        let bitmap = result.unwrap();
        assert_eq!(bitmap.len(), 128 * 8); // 128 width * 8 pages
    }

    #[test]
    fn test_generate_url_qr() {
        let result = generate_url_qr("192.168.1.100", 80);
        assert!(result.is_ok());
    }
}
