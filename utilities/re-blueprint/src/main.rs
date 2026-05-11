//! re-blueprint CLI — Sovereign Blueprint editor for Timux
  //!
  //! Commands:
  //!   new      [--json]               — create a default blueprint (binary or JSON)
  //!   validate <file>                 — validate a binary or JSON blueprint
  //!   show     <file>                 — pretty-print a blueprint
  //!   add      <file> --name <n> --profile <p> [--priority N] [--quota N]
  //!   encode   <json-file> <out.bin>  — JSON → binary
  //!   decode   <bin-file>  <out.json> — binary → JSON

  use anyhow::Context;
  use clap::{Parser, Subcommand};
  use re_blueprint::{Blueprint, BlueprintBuilder, BlueprintEntry, Profile};
  use std::{fs, path::PathBuf};

  #[derive(Parser)]
  #[command(name = "re-blueprint", about = "Sovereign Blueprint editor for the Timux kernel", version)]
  struct Cli {
      #[command(subcommand)]
      cmd: Cmd,
  }

  #[derive(Subcommand)]
  enum Cmd {
      /// Create a default blueprint
      New {
          #[arg(long, help = "Output as JSON instead of binary")]
          json: bool,
          #[arg(short, long, help = "Output file (default: stdout for json, blueprint.bin for binary)")]
          output: Option<PathBuf>,
      },
      /// Validate a blueprint file (binary or JSON)
      Validate {
          file: PathBuf,
      },
      /// Pretty-print a blueprint
      Show {
          file: PathBuf,
      },
      /// Add a sub-kernel entry to an existing blueprint file
      Add {
          file: PathBuf,
          #[arg(long)] name:     String,
          #[arg(long, default_value = "gp")] profile: String,
          #[arg(long, default_value_t = 128u8)] priority: u8,
          #[arg(long, default_value_t = 5000u64)] quota:   u64,
      },
      /// Encode a JSON blueprint to binary
      Encode {
          input:  PathBuf,
          output: PathBuf,
      },
      /// Decode a binary blueprint to JSON
      Decode {
          input:  PathBuf,
          output: PathBuf,
      },
  }

  fn load(path: &PathBuf) -> anyhow::Result<Blueprint> {
      let bytes = fs::read(path).with_context(|| format!("reading {:?}", path))?;
      // Detect JSON by first byte
      if bytes.first() == Some(&b'{') || bytes.first() == Some(&b' ') {
          let s = String::from_utf8(bytes).context("file is not valid UTF-8")?;
          Blueprint::from_json(&s).context("JSON parse failed")
      } else {
          Blueprint::decode(&bytes).context("binary decode failed")
      }
  }

  fn default_blueprint() -> anyhow::Result<Blueprint> {
      Ok(BlueprintBuilder::new()
          .gp("init",    128, 5_000)?
          .net("netd",   64,  8_000)?
          .storage("vfs", 64, 8_000)?
          .enclave("sec", 32, 3_000)?
          .build())
  }

  fn print_blueprint(bp: &Blueprint) {
      println!("Sovereign Blueprint — {} entr{}", bp.entries.len(),
          if bp.entries.len() == 1 { "y" } else { "ies" });
      println!("{:<16} {:<16} {:>8} {:>10}", "name", "profile", "priority", "quota (µs)");
      println!("{}", "-".repeat(54));
      for e in &bp.entries {
          println!("{:<16} {:<16} {:>8} {:>10}", e.name, e.profile.name(), e.priority, e.quota_us);
      }
  }

  fn main() -> anyhow::Result<()> {
      let cli = Cli::parse();
      match cli.cmd {
          Cmd::New { json, output } => {
              let bp = default_blueprint()?;
              if json {
                  let s = bp.to_json()?;
                  match output {
                      Some(p) => fs::write(&p, &s).with_context(|| format!("writing {:?}", p))?,
                      None    => println!("{s}"),
                  }
              } else {
                  let bytes = bp.encode();
                  let out = output.unwrap_or_else(|| PathBuf::from("blueprint.bin"));
                  fs::write(&out, &bytes).with_context(|| format!("writing {:?}", out))?;
                  println!("Wrote {} bytes to {:?}", bytes.len(), out);
              }
          }
          Cmd::Validate { file } => {
              let bp = load(&file)?;
              let issues = bp.validate();
              if issues.is_empty() {
                  println!("✅  Blueprint is valid ({} entries)", bp.entries.len());
              } else {
                  eprintln!("❌  {} issue(s) found:", issues.len());
                  for i in &issues { eprintln!("  • {i}"); }
                  std::process::exit(1);
              }
          }
          Cmd::Show { file } => {
              let bp = load(&file)?;
              print_blueprint(&bp);
              let issues = bp.validate();
              if !issues.is_empty() {
                  println!("\nWarnings:");
                  for i in &issues { println!("  ⚠  {i}"); }
              }
          }
          Cmd::Add { file, name, profile, priority, quota } => {
              let mut bp = load(&file).unwrap_or_else(|_| Blueprint::new());
              let p = match profile.to_lowercase().as_str() {
                  "rt" | "realtime" | "real-time" => Profile::RealTime,
                  "net" | "networking"             => Profile::Networking,
                  "storage"                        => Profile::Storage,
                  "graphics"                       => Profile::Graphics,
                  "enclave"                        => Profile::Enclave,
                  _                                => Profile::GeneralPurpose,
              };
              bp.add(BlueprintEntry::new(&name, p).priority(priority).quota_us(quota))?;
              let bytes = bp.encode();
              fs::write(&file, &bytes).with_context(|| format!("writing {:?}", file))?;
              println!("Added '{}' (profile={}, priority={}, quota={}µs) → {:?}", name, p.name(), priority, quota, file);
          }
          Cmd::Encode { input, output } => {
              let s = fs::read_to_string(&input).context("reading JSON")?;
              let bp = Blueprint::from_json(&s)?;
              let bytes = bp.encode();
              fs::write(&output, &bytes).context("writing binary")?;
              println!("Encoded {} entries → {} bytes → {:?}", bp.entries.len(), bytes.len(), output);
          }
          Cmd::Decode { input, output } => {
              let bytes = fs::read(&input).context("reading binary")?;
              let bp = Blueprint::decode(&bytes)?;
              let s = bp.to_json()?;
              fs::write(&output, &s).context("writing JSON")?;
              println!("Decoded {} entries → {:?}", bp.entries.len(), output);
          }
      }
      Ok(())
  }
  