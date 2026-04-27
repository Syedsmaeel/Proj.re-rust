use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::copy;

pub struct Downloader;

impl Downloader {
    /// Download a file (e.g. model weights) with progress tracking
    pub fn download(url: &str, dest: &str) -> Result<()> {
        println!("🦀 crab-rs — Downloading: {}", url);
        let mut response = reqwest::blocking::get(url)?;
        if !response.status().is_success() {
            return Err(anyhow!("Download failed: {}", response.status()));
        }
        let mut out = File::create(dest)?;
        copy(&mut response, &mut out)?;
        println!("✅ Downloaded to {}", dest);
        Ok(())
    }
}
