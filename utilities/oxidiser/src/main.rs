//! Oxidiser — Sovereign OS Generator
  //!
  //! Run with: oxidiser new
  //!       or: oxidiser new --name MyOS --arch x86_64 --features net,fs,crypto
  //!       or: oxidiser preset minimal|full|server|embedded
  //!
  //! Oxidiser interviews you with a short script and generates a complete,
  //! compilable Re-Rust sovereign OS workspace on disk.

  mod config;
  mod scaffold;
  mod templates;

  use std::io::{self, BufRead, Write};
  use std::path::PathBuf;
  use anyhow::Result;
  use clap::{Parser, Subcommand};
  use config::{OsConfig, OsFeatures, TargetArch};

  // ─── CLI definition ───────────────────────────────────────────────────────────

  #[derive(Parser)]
  #[command(
      name    = "oxidiser",
      about   = "Sovereign OS generator — create a complete Re-Rust OS from a script",
      version = env!("CARGO_PKG_VERSION"),
      long_about = "
  Oxidiser scaffolds a brand-new sovereign OS workspace in seconds.
  Answer a handful of prompts and get a fully compilable Rust workspace
  with a capability kernel, bootloader, optional networking, AI, and more.

  EXAMPLES:
    oxidiser new                        # interactive wizard
    oxidiser new --name Nexus           # pre-fill the OS name
    oxidiser preset full                # generate a full-featured OS
    oxidiser preset minimal             # generate a minimal OS
    oxidiser list-features              # show all available features
  "
  )]
  struct Cli {
      #[command(subcommand)]
      command: Command,
  }

  #[derive(Subcommand)]
  enum Command {
      /// Create a new sovereign OS (interactive wizard)
      New {
          /// OS name (e.g. Nexus)
          #[arg(long)]
          name: Option<String>,

          /// Slug — lowercase, no spaces (auto-derived from name if omitted)
          #[arg(long)]
          slug: Option<String>,

          /// Target architecture: x86_64 | arm64 | riscv | all
          #[arg(long)]
          arch: Option<String>,

          /// Comma-separated feature list: net,fs,ai,shadow,hotswap,blueprint,tui,crypto
          #[arg(long)]
          features: Option<String>,

          /// Output directory (default: ./<slug>)
          #[arg(long, short = 'o')]
          output: Option<PathBuf>,

          /// Skip all prompts and use defaults (non-interactive)
          #[arg(long, short = 'y')]
          yes: bool,
      },

      /// Generate an OS from a named preset (minimal | full | server | embedded)
      Preset {
          /// Preset name
          name: String,
          /// OS name override
          #[arg(long)]
          os_name: Option<String>,
          /// Output directory
          #[arg(long, short = 'o')]
          output: Option<PathBuf>,
      },

      /// List all available optional features
      ListFeatures,
  }

  // ─── Main ────────────────────────────────────────────────────────────────────

  fn main() -> Result<()> {
      let cli = Cli::parse();

      match cli.command {
          Command::ListFeatures => {
              print_features();
              return Ok(());
          }

          Command::Preset { name, os_name, output } => {
              let cfg = build_preset(&name, os_name, output)?;
              scaffold::generate(&cfg)?;
              return Ok(());
          }

          Command::New { name, slug, arch, features, output, yes } => {
              let cfg = if yes {
                  build_defaults(name, slug, arch, features, output)
              } else {
                  run_wizard(name, slug, arch, features, output)?
              };
              scaffold::generate(&cfg)?;
          }
      }

      Ok(())
  }

  // ─── Interactive wizard ───────────────────────────────────────────────────────

  fn run_wizard(
      pre_name:     Option<String>,
      pre_slug:     Option<String>,
      pre_arch:     Option<String>,
      pre_features: Option<String>,
      pre_output:   Option<PathBuf>,
  ) -> Result<OsConfig> {
      let stdin = io::stdin();
      let mut lines = stdin.lock().lines();

      print_banner();

      // ── 1. OS name ────────────────────────────────────────────────────────────
      let name = match pre_name {
          Some(n) => { println!("  OS name       : {n}"); n }
          None => {
              prompt("  Enter your OS name (e.g. Nexus, Aether, Kronos): ");
              let raw = lines.next().unwrap_or(Ok(String::new()))?;
              let n = raw.trim().to_string();
              if n.is_empty() { anyhow::bail!("OS name cannot be empty."); }
              n
          }
      };

      // ── 2. Slug ───────────────────────────────────────────────────────────────
      let default_slug = slugify(&name);
      let slug = match pre_slug {
          Some(s) => s,
          None => {
              prompt(&format!("  Package slug   [{default_slug}]: "));
              let raw = lines.next().unwrap_or(Ok(String::new()))?;
              let s = raw.trim().to_string();
              if s.is_empty() { default_slug } else { slugify(&s) }
          }
      };

      // ── 3. One-line description ───────────────────────────────────────────────
      prompt("  One-line description: ");
      let desc_raw = lines.next().unwrap_or(Ok(String::new()))?;
      let description = {
          let d = desc_raw.trim().to_string();
          if d.is_empty() { format!("{name} — a sovereign OS built with Re-Rust") } else { d }
      };

      // ── 4. Author ─────────────────────────────────────────────────────────────
      prompt("  Author / org name: ");
      let author_raw = lines.next().unwrap_or(Ok(String::new()))?;
      let author = {
          let a = author_raw.trim().to_string();
          if a.is_empty() { String::from("Oxidiser User") } else { a }
      };

      // ── 5. Architecture ───────────────────────────────────────────────────────
      let arch = match pre_arch {
          Some(ref a) => TargetArch::from_str(a).unwrap_or(TargetArch::X86_64),
          None => {
              println!();
              println!("  Target architecture:");
              println!("    1) x86_64   — standard PC / server");
              println!("    2) arm64    — Raspberry Pi, Apple Silicon, AWS Graviton");
              println!("    3) riscv    — RISC-V boards (QEMU virt, HiFive)");
              println!("    4) all      — generate stubs for all three");
              prompt("  Choose [1]: ");
              let raw = lines.next().unwrap_or(Ok(String::new()))?;
              TargetArch::from_str(raw.trim()).unwrap_or(TargetArch::X86_64)
          }
      };
      println!("  Architecture   : {}", arch.display());

      // ── 6. Features ───────────────────────────────────────────────────────────
      let features = match pre_features {
          Some(ref f) => {
              let flags: Vec<&str> = f.split(',').collect();
              OsFeatures::from_flags(&flags)
          }
          None => {
              println!();
              println!("  Optional feature modules (space or comma-separated, or 'all', or leave blank):");
              println!("    net       — capability-gated TCP/UDP network stack");
              println!("    fs        — Inter-Kernel Filesystem (IKFS)");
              println!("    ai        — LLM inference sub-kernel stub");
              println!("    shadow    — shadow manager + automatic failover");
              println!("    hotswap   — live sub-kernel hot-swap coordinator");
              println!("    blueprint — boot blueprint editor");
              println!("    tui       — Ring -1 TUI boot menu");
              println!("    crypto    — sovereign crypto (ChaCha20, HMAC, HKDF)");
              prompt("  Features [none]: ");
              let raw = lines.next().unwrap_or(Ok(String::new()))?;
              let input = raw.trim().to_lowercase();
              if input == "all" {
                  OsFeatures::all()
              } else {
                  let flags: Vec<&str> = input.split(|c| c == ',' || c == ' ').collect();
                  OsFeatures::from_flags(&flags)
              }
          }
      };

      // ── 7. Output directory ───────────────────────────────────────────────────
      let output_dir = match pre_output {
          Some(p) => p,
          None => {
              let default = format!("./{slug}");
              prompt(&format!("  Output directory [{default}]: "));
              let raw = lines.next().unwrap_or(Ok(String::new()))?;
              let s = raw.trim().to_string();
              PathBuf::from(if s.is_empty() { default } else { s })
          }
      };

      // ── 8. Confirm ────────────────────────────────────────────────────────────
      println!();
      println!("  ┌─ Summary ─────────────────────────────────────────┐");
      println!("  │  Name   : {:<43}│", name);
      println!("  │  Slug   : {:<43}│", slug);
      println!("  │  Arch   : {:<43}│", arch.display());
      println!("  │  Output : {:<43}│", output_dir.display());
      let feat_list = features.enabled_list();
      if feat_list.is_empty() {
          println!("  │  Feats  : minimal                                 │");
      } else {
          for f in &feat_list {
              println!("  │  ✓      : {:<43}│", f);
          }
      }
      println!("  └───────────────────────────────────────────────────┘");
      prompt("  Generate? [Y/n]: ");
      let confirm = lines.next().unwrap_or(Ok(String::new()))?;
      let c = confirm.trim().to_lowercase();
      if c == "n" || c == "no" {
          anyhow::bail!("Aborted by user.");
      }

      Ok(OsConfig {
          name, slug, description, author, arch, features,
          output_dir, edition: String::from("2021"), version: String::from("0.1.0"),
      })
  }

  // ─── Non-interactive defaults ─────────────────────────────────────────────────

  fn build_defaults(
      name:     Option<String>,
      slug:     Option<String>,
      arch:     Option<String>,
      features: Option<String>,
      output:   Option<PathBuf>,
  ) -> OsConfig {
      let name = name.unwrap_or_else(|| String::from("SovereignOS"));
      let slug = slug.unwrap_or_else(|| slugify(&name));
      let arch = arch.as_deref().and_then(TargetArch::from_str).unwrap_or(TargetArch::X86_64);
      let features = features
          .as_deref()
          .map(|f| OsFeatures::from_flags(&f.split(',').collect::<Vec<_>>()))
          .unwrap_or_default();
      let output_dir = output.unwrap_or_else(|| PathBuf::from(format!("./{slug}")));
      OsConfig {
          name, slug, description: String::from("A sovereign OS"),
          author: String::from("Oxidiser"), arch, features,
          output_dir, edition: String::from("2021"), version: String::from("0.1.0"),
      }
  }

  // ─── Presets ──────────────────────────────────────────────────────────────────

  fn build_preset(preset: &str, os_name: Option<String>, output: Option<PathBuf>) -> Result<OsConfig> {
      let (name, features) = match preset.to_lowercase().as_str() {
          "minimal" => (
              os_name.unwrap_or_else(|| String::from("MinimalOS")),
              OsFeatures::minimal(),
          ),
          "full" => (
              os_name.unwrap_or_else(|| String::from("SovereignOS")),
              OsFeatures::all(),
          ),
          "server" => (
              os_name.unwrap_or_else(|| String::from("ServerOS")),
              OsFeatures {
                  networking: true, filesystem: true,
                  shadow: true, hotswap: true, crypto: true,
                  ..Default::default()
              },
          ),
          "embedded" => (
              os_name.unwrap_or_else(|| String::from("EmbeddedOS")),
              OsFeatures { crypto: true, ..Default::default() },
          ),
          "ai" => (
              os_name.unwrap_or_else(|| String::from("NeuralOS")),
              OsFeatures { ai: true, networking: true, filesystem: true, crypto: true, ..Default::default() },
          ),
          _ => anyhow::bail!(
              "Unknown preset '{}'. Available: minimal, full, server, embedded, ai", preset
          ),
      };

      let slug      = slugify(&name);
      let output_dir = output.unwrap_or_else(|| PathBuf::from(format!("./{slug}")));

      println!("Using preset: {preset}");
      Ok(OsConfig {
          name: name.clone(), slug, description: format!("{name} — {preset} sovereign OS"),
          author: String::from("Oxidiser"), arch: TargetArch::X86_64,
          features, output_dir,
          edition: String::from("2021"), version: String::from("0.1.0"),
      })
  }

  // ─── Helpers ──────────────────────────────────────────────────────────────────

  /// Convert an OS name to a valid Rust crate slug.
  fn slugify(s: &str) -> String {
      s.to_lowercase()
          .chars()
          .map(|c| if c.is_alphanumeric() { c } else { '-' })
          .collect::<String>()
          .trim_matches('-')
          .to_string()
  }

  fn prompt(msg: &str) {
      print!("{msg}");
      io::stdout().flush().unwrap();
  }

  fn print_banner() {
      println!();
      println!(" ██████╗ ██╗  ██╗██╗██████╗ ██╗███████╗███████╗██████╗ ");
      println!("██╔═══██╗╚██╗██╔╝██║██╔══██╗██║██╔════╝██╔════╝██╔══██╗");
      println!("██║   ██║ ╚███╔╝ ██║██║  ██║██║███████╗█████╗  ██████╔╝");
      println!("██║   ██║ ██╔██╗ ██║██║  ██║██║╚════██║██╔══╝  ██╔══██╗");
      println!("╚██████╔╝██╔╝ ██╗██║██████╔╝██║███████║███████╗██║  ██║");
      println!(" ╚═════╝ ╚═╝  ╚═╝╚═╝╚═════╝ ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝");
      println!();
      println!("  Sovereign OS Generator — part of the Re-Rust / Timux ecosystem");
      println!("  Answer the prompts below to forge your new OS.");
      println!();
  }

  fn print_features() {
      println!();
      println!("Available Oxidiser features:");
      println!("  net       — Capability-gated TCP/UDP network stack sub-kernel");
      println!("  fs        — Inter-Kernel Filesystem (IKFS) with vnode layer");
      println!("  ai        — LLM inference sub-kernel stub (candle-core ready)");
      println!("  shadow    — Shadow manager + automatic failover on heartbeat loss");
      println!("  hotswap   — Live sub-kernel replacement without reboot");
      println!("  blueprint — Boot blueprint editor (create/validate/encode .tmxb)");
      println!("  tui       — Ring -1 TUI boot menu with pixel framebuffer");
      println!("  crypto    — Sovereign crypto: ChaCha20 stream cipher, HMAC, HKDF");
      println!();
      println!("Presets:");
      println!("  minimal   — cap + sched + ipc + sub-kernel manager only");
      println!("  full      — everything enabled");
      println!("  server    — net + fs + shadow + hotswap + crypto");
      println!("  embedded  — minimal + crypto");
      println!("  ai        — net + fs + ai + crypto");
      println!();
  }
  