use std::io::{stdout, Write};

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    current: Vec<char>,
    previous: Vec<char>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { 
            width, height, 
            current: vec![' '; width * height], 
            previous: vec!['\0'; width * height], 
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: char) {
        if x < self.width && y < self.height {
            self.current[y * self.width + x] = c;
        }
    }

    /// The "React" Logic: Only write to stdout if the character changed
    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = stdout();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if self.current[idx] != self.previous[idx] {
                    // Move cursor and print only this cell
                    print!("\x1b[{};{}H{}", y + 1, x + 1, self.current[idx]);
                    self.previous[idx] = self.current[idx];
                }
            }
        }
        out.flush()?;
        Ok(())
    }
}
