use std::fs::OpenOptions;
use std::ptr;
use libc::{mmap, PROT_READ, PROT_WRITE, MAP_SHARED, MAP_FAILED};

pub struct Shape {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub color: u32,
}

impl Shape {
    pub fn is_hit(&self, px: usize, py: usize) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    data: *mut u32,
}

impl Framebuffer {
    pub fn open() -> Result<Self, String> {
        let file = OpenOptions::new().read(true).write(true).open("/dev/fb0")
            .map_err(|e| format!("Must be root to access /dev/fb0: {}", e))?;
        
        let width = 1920;
        let height = 1080;
        let size = width * height * 4;

        let ptr = unsafe {
            mmap(ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_SHARED, file.as_raw_fd(), 0)
        };

        if ptr == MAP_FAILED { return Err("Failed to map framebuffer".into()); }
        Ok(Self { width, height, data: ptr as *mut u32 })
    }

    pub fn draw_rect(&mut self, shape: &Shape) {
        for y in shape.y..(shape.y + shape.h).min(self.height) {
            for x in shape.x..(shape.x + shape.w).min(self.width) {
                unsafe { *self.data.add(y * self.width + x) = shape.color; }
            }
        }
    }
}

use std::os::unix::io::AsRawFd;
