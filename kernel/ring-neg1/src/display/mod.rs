//! Framebuffer display driver — GOP (UEFI) or VESA (BIOS)
extern crate alloc;
use alloc::vec::Vec;

/// A 32-bit ARGB pixel
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Pixel { pub b: u8, pub g: u8, pub r: u8, pub a: u8 }

impl Pixel {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 0xff } }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
}

/// Named colors used by the boot menu
pub mod Color {
    use super::Pixel;
    pub const BLACK:      Pixel = Pixel::rgb(0x00, 0x00, 0x00);
    pub const WHITE:      Pixel = Pixel::rgb(0xff, 0xff, 0xff);
    pub const CYAN:       Pixel = Pixel::rgb(0x00, 0xff, 0xff);
    pub const DARK_CYAN:  Pixel = Pixel::rgb(0x00, 0x88, 0x88);
    pub const ORANGE:     Pixel = Pixel::rgb(0xff, 0x88, 0x00);
    pub const RED:        Pixel = Pixel::rgb(0xff, 0x44, 0x44);
    pub const GREEN:      Pixel = Pixel::rgb(0x44, 0xff, 0x88);
    pub const YELLOW:     Pixel = Pixel::rgb(0xff, 0xff, 0x00);
    pub const DARK_GRAY:  Pixel = Pixel::rgb(0x1a, 0x1a, 0x2e);
    pub const PANEL:      Pixel = Pixel::rgb(0x16, 0x21, 0x3e);
    pub const ACCENT:     Pixel = Pixel::rgb(0x0f, 0x3d, 0x66);
    pub const HIGHLIGHT:  Pixel = Pixel::rgb(0x1a, 0x53, 0x8b);
}

/// Raw framebuffer — write pixels directly to video memory
pub struct Framebuffer {
    pub base:   *mut u32,
    pub width:  u32,
    pub height: u32,
    pub stride: u32,  // pixels per row (may differ from width)
}

impl Framebuffer {
    pub unsafe fn new(base: *mut u32, width: u32, height: u32, stride: u32) -> Self {
        Self { base, width, height, stride }
    }

    /// Write a single pixel
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Pixel) {
        if x >= self.width || y >= self.height { return; }
        unsafe {
            let offset = (y * self.stride + x) as usize;
            let val = ((color.a as u32) << 24)
                    | ((color.r as u32) << 16)
                    | ((color.g as u32) << 8)
                    |  (color.b as u32);
            self.base.add(offset).write_volatile(val);
        }
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Pixel) {
        for row in y..y.saturating_add(h).min(self.height) {
            for col in x..x.saturating_add(w).min(self.width) {
                self.set_pixel(col, row, color);
            }
        }
    }

    /// Draw a horizontal line
    pub fn hline(&mut self, x: u32, y: u32, len: u32, color: Pixel) {
        self.fill_rect(x, y, len, 1, color);
    }

    /// Draw a vertical line
    pub fn vline(&mut self, x: u32, y: u32, len: u32, color: Pixel) {
        self.fill_rect(x, 1, 1, len, color);
    }

    /// Draw a border rectangle (outline only)
    pub fn draw_border(&mut self, x: u32, y: u32, w: u32, h: u32, color: Pixel) {
        self.hline(x, y, w, color);
        self.hline(x, y + h - 1, w, color);
        self.vline(x, y, h, color);
        self.vline(x + w - 1, y, h, color);
    }

    /// Clear the entire screen
    pub fn clear(&mut self, color: Pixel) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    /// Draw a psf1 font character (8x16 bitmap)
    /// `bitmap` is 16 bytes, one byte per row, MSB = leftmost pixel
    pub fn draw_char(&mut self, x: u32, y: u32, bitmap: &[u8; 16], fg: Pixel, bg: Pixel) {
        for (row, &byte) in bitmap.iter().enumerate() {
            for col in 0..8u32 {
                let set = (byte >> (7 - col)) & 1 == 1;
                self.set_pixel(x + col, y + row as u32, if set { fg } else { bg });
            }
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
}
