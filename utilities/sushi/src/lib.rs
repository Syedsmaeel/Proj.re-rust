pub mod diff_renderer;
pub struct SubPixelCanvas {
    pub width: usize,
    pub height: usize,
    buffer: Vec<u8>, // Each byte represents 8 dots (2x4 Braille)
}

impl SubPixelCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, buffer: vec![0; (width / 2) * (height / 4)] }
    }

    /// Set a sub-pixel (high-res) at (x, y)
    pub fn set_pixel(&mut self, x: usize, y: usize) {
        let char_x = x / 2;
        let char_y = y / 4;
        let bit = (1 << ((y % 4) * 2 + (x % 2)));
        let idx = char_y * (self.width / 2) + char_x;
        self.buffer[idx] |= bit;
    }

    /// Render the canvas to the terminal using Braille Unicode (0x2800)
    pub fn render(&self) {
        for y in 0..(self.height / 4) {
            for x in 0..(self.width / 2) {
                let idx = y * (self.width / 2) + x;
                let c = std::char::from_u32(0x2800 + self.buffer[idx] as u32).unwrap();
                print!("{}", c);
            }
            println!();
        }
    }
}
