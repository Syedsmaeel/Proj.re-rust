use anyhow::{Result, Context};
use clap::Parser;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "re-run", about = "Run Rust files like scripts", version)]
struct Cli {
    /// The .rs file to run
    file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_path = Path::new(&cli.file);
    
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", cli.file);
    }

    // 1. Create a unique hash of the file path to use as a cache key
    let mut hasher = Sha256::new();
    hasher.update(fs::canonicalize(file_path)?.to_string_lossy().as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    // 2. Setup Cache Directory
    let cache_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".sushi/cache/re-run")
        .join(&hash);
    
    fs::create_dir_all(&cache_dir)?;

    let bin_path = cache_dir.join("script_bin");
    let source_hash_path = cache_dir.join("source.hash");

    // 3. Check if we need to recompile
    let current_content = fs::read(&cli.file)?;
    let mut content_hasher = Sha256::new();
    content_hasher.update(&current_content);
    let current_hash = hex::encode(content_hasher.finalize());

    let mut needs_compile = true;
    if bin_path.exists() && source_hash_path.exists() {
        let old_hash = fs::read_to_string(&source_hash_path)?;
        if old_hash == current_hash {
            needs_compile = false;
        }
    }

    if needs_compile {
        println!("🚀 re-run — First run/Changes detected. Compiling...");
        compile_script(file_path, &cache_dir, &bin_path)?;
        fs::write(source_hash_path, current_hash)?;
    }

    // 4. Run the cached binary
    let status = Command::new(&bin_path)
        .status()
        .context("Failed to execute script")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn compile_script(source: &Path, work_dir: &Path, dest: &Path) -> Result<()> {
    // We use 'rustc' directly for single-file speed, or we can generate a mini Cargo project.
    // For ultimate "scripting" speed, rustc is faster.
    let status = Command::new("rustc")
        .arg(source)
        .arg("-o")
        .arg(dest)
        .arg("-C")
        .arg("opt-level=2") // Balanced optimization
        .status()
        .context("rustc failed to run")?;

    if !status.success() {
        anyhow::bail!("Compilation failed.");
    }
    Ok(())
}
