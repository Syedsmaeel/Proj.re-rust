use anyhow::{Result, Context};
use clap::Parser;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "re-run", about = "Instant Rust script runner with dependencies", version)]
struct Cli {
    file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_path = Path::new(&cli.file);
    if !file_path.exists() { anyhow::bail!("File not found"); }

    let content = fs::read_to_string(file_path)?;
    
    // 1. Parse Dependencies from comments
    let mut deps = String::new();
    for line in content.lines() {
        if line.starts_with("// [dependencies]") { continue; }
        if line.starts_with("// ") && (line.contains("=") || line.contains("{")) {
            deps.push_str(&line[3..]);
            deps.push('\n');
        } else if !line.starts_with("//") {
            break; // Stop looking for deps after first non-comment line
        }
    }

    // 2. Setup Cache Key
    let mut hasher = Sha256::new();
    hasher.update(fs::canonicalize(file_path)?.to_string_lossy().as_bytes());
    hasher.update(deps.as_bytes()); // Hash deps too so changes trigger rebuild
    let hash = hex::encode(hasher.finalize());
    
    let cache_dir = dirs::home_dir()
        .context("Home dir not found")?
        .join(".sushi/cache/re-run").join(&hash);
    
    fs::create_dir_all(&cache_dir)?;
    let bin_path = if cfg!(windows) { cache_dir.join("target/release/script.exe") } else { cache_dir.join("target/release/script") };
    let source_hash_path = cache_dir.join("source.hash");

    // 3. Check Cache
    let mut content_hasher = Sha256::new();
    content_hasher.update(&content);
    let current_hash = hex::encode(content_hasher.finalize());

    let mut needs_compile = true;
    if bin_path.exists() && source_hash_path.exists() {
        if fs::read_to_string(&source_hash_path)? == current_hash {
            needs_compile = false;
        }
    }

    if needs_compile {
        println!("🚀 re-run — Setting up dependencies & compiling...");
        setup_and_compile(file_path, &cache_dir, &deps)?;
        fs::write(source_hash_path, current_hash)?;
    }

    // 4. Run
    let status = Command::new(&bin_path).status()?;
    if !status.success() { std::process::exit(status.code().unwrap_or(1)); }

    Ok(())
}

fn setup_and_compile(source: &Path, cache_dir: &Path, deps: &str) -> Result<()> {
    // Create a mini Cargo project
    let src_dir = cache_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    let cargo_toml = format!(
        "[package]\nname = \"script\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{}",
        deps
    );
    fs::write(cache_dir.join("Cargo.toml"), cargo_toml)?;
    fs::copy(source, src_dir.join("main.rs"))?;

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(cache_dir)
        .status()?;

    if !status.success() { anyhow::bail!("Cargo build failed"); }
    Ok(())
}
