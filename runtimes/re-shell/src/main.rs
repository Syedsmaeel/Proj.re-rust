use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use owo_colors::OwoColorize;
use std::process::Command;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🐚 re-shell — Sovereign Terminal".bold().orange());
    println!("Type 'ghost <prompt>' to ask the AI, or just run commands.");
    
    let mut rl = DefaultEditor::new()?;
    let history_path = dirs::home_dir().unwrap().join(".re_shell_history");
    let _ = rl.load_history(&history_path);

    loop {
        let readline = rl.readline("re-sh> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() { continue; }
                if line == "exit" || line == "quit" { break; }
                
                let _ = rl.add_history_entry(line);

                if line.starts_with("ghost ") {
                    let prompt = &line[6..];
                    ask_ghost(prompt).await?;
                } else {
                    execute_command(line).await?;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path)?;
    Ok(())
}

async fn execute_command(line: &str) -> Result<()> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let cmd = parts[0];
    let args = &parts[1..];

    // Handle 'cd' manually as it's a shell builtin
    if cmd == "cd" {
        let new_dir = args.get(0).map(|s| *s).unwrap_or(".");
        if let Err(e) = env::set_current_dir(new_dir) {
            eprintln!("cd error: {}", e);
        }
        return Ok(());
    }

    let status = Command::new(cmd)
        .args(args)
        .status();

    match status {
        Ok(s) if !s.success() => {
            println!("{}", "⚠ Command failed. Use 'ghost why did that fail?' to analyze.".yellow());
        }
        Err(e) => {
            println!("{} {}", "✘ Error:".red().bold(), e);
        }
        _ => {}
    }
    Ok(())
}

async fn ask_ghost(prompt: &str) -> Result<()> {
    use re_llm::{PhiBackend, PhiVariant, GenerationConfig};
    
    println!("{}", "👻 Consulting the Ghost (Phi-3)...".bright_black().italic());
    
    // In a real run, this would use the local GGUF if available.
    // For now, we load the standard variant.
    match PhiBackend::load(PhiVariant::MiniInstruct4k) {
        Ok(mut backend) => {
            let cfg = GenerationConfig::default();
            let response = backend.generate(prompt, &cfg)?;
            println!("\n{}\n", response.cyan());
        }
        Err(e) => println!("Ghost failed to materialize: {}", e),
    }
    Ok(())
}
