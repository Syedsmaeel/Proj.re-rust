//! OsConfig — the structured answer to every Oxidiser prompt.
  //!
  //! Collected interactively by `main.rs` and then handed to `scaffold.rs`.

  use serde::{Deserialize, Serialize};

  // ─── Architecture ─────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub enum TargetArch {
      X86_64,
      Arm64,
      RiscV,
      All,
  }

  impl TargetArch {
      pub fn from_str(s: &str) -> Option<Self> {
          match s.trim().to_lowercase().as_str() {
              "x86_64" | "x86" | "1" => Some(Self::X86_64),
              "arm64" | "aarch64" | "2" => Some(Self::Arm64),
              "riscv" | "riscv64" | "3" => Some(Self::RiscV),
              "all" | "4" => Some(Self::All),
              _ => None,
          }
      }

      pub fn rust_target(&self) -> &'static str {
          match self {
              Self::X86_64 => "x86_64-unknown-none",
              Self::Arm64  => "aarch64-unknown-none-softfloat",
              Self::RiscV  => "riscv64gc-unknown-none-elf",
              Self::All    => "x86_64-unknown-none",
          }
      }

      pub fn display(&self) -> &'static str {
          match self {
              Self::X86_64 => "x86_64",
              Self::Arm64  => "ARM64 (AArch64)",
              Self::RiscV  => "RISC-V 64",
              Self::All    => "All (x86_64 + ARM64 + RISC-V)",
          }
      }
  }

  // ─── Optional features ────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  pub struct OsFeatures {
      pub networking:  bool,   // include re-net network stack sub-kernel
      pub filesystem:  bool,   // include IKFS inter-kernel filesystem
      pub ai:          bool,   // include re-llm inference sub-kernel
      pub shadow:      bool,   // include shadow manager + failover
      pub hotswap:     bool,   // include hot-swap sub-kernel coordinator
      pub blueprint:   bool,   // include re-blueprint editor
      pub tui_menu:    bool,   // include Ring -1 TUI boot menu
      pub crypto:      bool,   // include sovereign crypto primitives
  }

  impl OsFeatures {
      pub fn all() -> Self {
          Self {
              networking: true, filesystem: true, ai: true,
              shadow: true, hotswap: true, blueprint: true,
              tui_menu: true, crypto: true,
          }
      }

      pub fn minimal() -> Self { Self::default() }

      pub fn from_flags(flags: &[&str]) -> Self {
          let mut f = Self::default();
          for flag in flags {
              match flag.trim().to_lowercase().as_str() {
                  "net" | "networking"  => f.networking  = true,
                  "fs"  | "filesystem"  => f.filesystem  = true,
                  "ai"  | "llm"         => f.ai           = true,
                  "shadow"              => f.shadow       = true,
                  "hotswap"             => f.hotswap      = true,
                  "blueprint"           => f.blueprint    = true,
                  "tui" | "menu"        => f.tui_menu     = true,
                  "crypto"              => f.crypto       = true,
                  _ => {}
              }
          }
          f
      }

      /// Human-readable list of enabled features.
      pub fn enabled_list(&self) -> Vec<&'static str> {
          let mut v = Vec::new();
          if self.networking  { v.push("Networking (re-net TCP/UDP)"); }
          if self.filesystem  { v.push("Inter-Kernel Filesystem (IKFS)"); }
          if self.ai          { v.push("AI Inference (re-llm)"); }
          if self.shadow      { v.push("Shadow Manager + Failover"); }
          if self.hotswap     { v.push("Hot-Swap Sub-kernel"); }
          if self.blueprint   { v.push("Blueprint Editor"); }
          if self.tui_menu    { v.push("TUI Boot Menu (Ring -1)"); }
          if self.crypto      { v.push("Sovereign Crypto (ChaCha20 / HMAC)"); }
          v
      }
  }

  // ─── OsConfig — the complete spec for a new sovereign OS ─────────────────────

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct OsConfig {
      /// Human-readable OS name, e.g. "Nexus"
      pub name: String,
      /// Lowercase slug used in Cargo package names, e.g. "nexus"
      pub slug: String,
      /// One-line description
      pub description: String,
      /// Author / organisation name
      pub author: String,
      /// Target hardware architecture(s)
      pub arch: TargetArch,
      /// Optional feature modules
      pub features: OsFeatures,
      /// Output directory for the generated workspace
      pub output_dir: std::path::PathBuf,
      /// Rust edition
      pub edition: String,
      /// Workspace version
      pub version: String,
  }

  impl OsConfig {
      pub fn kernel_crate(&self) -> String { format!("{}-kernel", self.slug) }
      pub fn core_crate(&self)   -> String { format!("{}-core",   self.slug) }
      pub fn boot_crate(&self)   -> String { format!("{}-boot",   self.slug) }
  }
  