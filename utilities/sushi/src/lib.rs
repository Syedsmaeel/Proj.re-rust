use std::fs::{OpenOptions, File};
use std::os::unix::io::AsRawFd;
use std::ptr;
use libc::{mmap, PROT_READ, PROT_WRITE, MAP_SHARED, MAP_FAILED};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    data: *mut u32,
    size: usize,
}

impl Framebuffer {
    pub fn open() -> Result<Self, String> {
        let file = OpenOptions::new().read(true).write(true).open("/dev/fb0")
            .map_err(|e| format!("Failed to open /dev/fb0: {}. Are you root?", e))?;
        
        // This is a simplified hardware resolution (1920x1080)
        let width = 1920;
        let height = 1080;
        let size = width * height * 4;

        let ptr = unsafe {
            mmap(ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_SHARED, file.as_raw_fd(), 0)
        };

        if ptr == MAP_FAILED {
            return Err("Failed to map framebuffer".into());
        }

        Ok(Self { width, height, data: ptr as *mut u32, size })
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            unsafe {
                *self.data.add(y * self.width + x) = color;
            }
        }
    }
}
