use anyhow::{Result, Context};
use clap::Parser;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Child};
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, Config};

#[derive(Parser)]
#[command(name = "re-run", about = "Elite Rust runner with Hot Reloading", version)]
struct Cli {
    file: String,
    /// Enable Hot Reload (watch for changes)
    #[arg(short, long)]
    watch: bool,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_path = fs::canonicalize(&cli.file)?;

    if cli.watch {
        println!("🔭 re-run — Hot Reloading Active. Watching {}...", cli.file);
        watch_and_run(&file_path, &cli.args)?;
    } else {
        run_once(&file_path, &cli.args)?;
    }
    Ok(())
}

fn run_once(file: &Path, args: &[String]) -> Result<()> {
    let bin = compile_if_needed(file)?;
    let mut status = Command::new(bin).args(args).status()?;
    if !status.success() { std::process::exit(status.code().unwrap_or(1)); }
    Ok(())
}

fn watch_and_run(file: &Path, args: &[String]) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(file, RecursiveMode::NonRecursive)?;

    let mut child: Option<Child> = None;

    // Initial Run
    if let Ok(bin) = compile_if_needed(file) {
        child = Some(Command::new(bin).args(args).spawn()?);
    }

    for res in rx {
        match res {
            Ok(_) => {
                println!("\n🔄 Change detected! Re-launching...");
                if let Some(mut c) = child.take() { let _ = c.kill(); }
                
                // Debounce / Wait a tiny bit for the file to finish writing
                std::thread::sleep(Duration::from_millis(200));

                if let Ok(bin) = compile_if_needed(file) {
                    child = Some(Command::new(bin).args(args).spawn()?);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}

fn compile_if_needed(file: &Path) -> Result<PathBuf> {
    let content = fs::read_to_string(file)?;
    
    // 1. Hash content and path
    let mut hasher = Sha256::new();
    hasher.update(file.to_string_lossy().as_bytes());
    hasher.update(content.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let cache_dir = dirs::home_dir().context("No home")?
        .join(".sushi/cache/re-run").join(&hash);
    
    fs::create_dir_all(&cache_dir)?;
    let bin_path = if cfg!(windows) { cache_dir.join("target/release/script.exe") } else { cache_dir.join("target/release/script") };

    if !bin_path.exists() {
        println!("🚀 re-run — Compiling...");
        setup_cargo_project(file, &cache_dir, &content)?;
        let status = Command::new("cargo").arg("build").arg("--release")
            .current_dir(&cache_dir).status()?;
        if !status.success() { anyhow::bail!("Compile failed"); }
    }

    Ok(bin_path)
}

fn setup_cargo_project(source: &Path, cache_dir: &Path, content: &str) -> Result<()> {
    let src_dir = cache_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    let mut deps = String::new();
    for line in content.lines() {
        if line.starts_with("// ") && (line.contains("=") || line.contains("{")) {
            deps.push_str(&line[3..]);
            deps.push('\n');
        } else if !line.starts_with("//") { break; }
    }

    let cargo_toml = format!(
        "[package]\nname = \"script\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{}",
        deps
    );
    fs::write(cache_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(src_dir.join("main.rs"), content)?;
    Ok(())
}
