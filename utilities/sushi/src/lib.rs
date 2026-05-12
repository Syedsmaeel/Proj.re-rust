use nix::pty::{openpty, Winsize};
use std::os::unix::io::AsRawFd;

pub struct SovereignTerminal {
    pub master: std::fs::File,
    pub slave: std::fs::File,
}

impl SovereignTerminal {
    pub fn new() -> Result<Self, anyhow::Error> {
        let pty = openpty(None, None)?;
        // We now have total control over the terminal's input/output
        Ok(Self {
            master: unsafe { std::fs::File::from_raw_fd(pty.master) },
            slave: unsafe { std::fs::File::from_raw_fd(pty.slave) },
        })
    }

    /// Direct 24-bit color injection
    pub fn paint_pixel(&self, r: u8, g: u8, b: u8) {
        // Send raw RGB bytes directly to the terminal's display buffer
        use std::io::Write;
        let mut master = &self.master;
        let _ = write!(master, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
}
