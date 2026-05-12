//! re-blueprint — Sovereign Blueprint library
  //!
  //! A Sovereign Blueprint is the boot-time declaration that tells the Timux
  //! kernel which sub-kernels to spawn and in what configuration.
  //!
  //! # Binary format (v1, little-endian)
  //! Header (8 bytes):  magic(4) + entry_count(4)
  //! Per entry (32 B):  profile_id(1) priority(1) quota_us(8) flags(4) name[16](null-padded) reserved(2)

  use serde::{Deserialize, Serialize};

  pub const MAGIC: u32 = 0x424C_5052;  // "BLPR"
  pub const ENTRY_SIZE: usize = 32;
  pub const MAX_ENTRIES: usize = 64;
  pub const DEFAULT_QUOTA_US: u64 = 5_000;

  // ─── Profile ─────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "lowercase")]
  pub enum Profile {
      GeneralPurpose = 0,
      RealTime       = 1,
      Networking     = 2,
      Storage        = 3,
      Graphics       = 4,
      Enclave        = 5,
  }

  impl Profile {
      pub fn from_id(id: u8) -> Self {
          match id { 1=>Self::RealTime, 2=>Self::Networking, 3=>Self::Storage,
                     4=>Self::Graphics, 5=>Self::Enclave, _=>Self::GeneralPurpose }
      }
      pub fn id(self) -> u8 { self as u8 }
      pub fn name(self) -> &'static str {
          match self {
              Self::GeneralPurpose => "general-purpose",
              Self::RealTime       => "real-time",
              Self::Networking     => "networking",
              Self::Storage        => "storage",
              Self::Graphics       => "graphics",
              Self::Enclave        => "enclave",
          }
      }
  }

  // ─── BlueprintEntry ──────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct BlueprintEntry {
      pub name:     String,
      pub profile:  Profile,
      pub priority: u8,
      pub quota_us: u64,
  }

  impl BlueprintEntry {
      pub fn new(name: impl Into<String>, profile: Profile) -> Self {
          Self { name: name.into(), profile, priority: 128, quota_us: DEFAULT_QUOTA_US }
      }
      pub fn priority(mut self, p: u8)   -> Self { self.priority = p; self }
      pub fn quota_us(mut self, q: u64)  -> Self { self.quota_us = q; self }
  }

  // ─── Blueprint ───────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  pub struct Blueprint {
      pub entries: Vec<BlueprintEntry>,
  }

  impl Blueprint {
      pub fn new() -> Self { Self::default() }

      pub fn add(&mut self, entry: BlueprintEntry) -> anyhow::Result<()> {
          self.validate_entry(&entry)?;
          if self.entries.len() >= MAX_ENTRIES {
              anyhow::bail!("blueprint already has {MAX_ENTRIES} entries (maximum)");
          }
          self.entries.push(entry);
          Ok(())
      }

      fn validate_entry(&self, e: &BlueprintEntry) -> anyhow::Result<()> {
          anyhow::ensure!(!e.name.is_empty(), "entry name cannot be empty");
          anyhow::ensure!(e.name.len() <= 15, "entry name '{}' exceeds 15 characters", e.name);
          anyhow::ensure!(
              e.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
              "entry name '{}' contains invalid characters", e.name
          );
          anyhow::ensure!(
              !self.entries.iter().any(|x| x.name == e.name),
              "duplicate sub-kernel name '{}'", e.name
          );
          Ok(())
      }

      /// Encode to binary wire format.
      pub fn encode(&self) -> Vec<u8> {
          let mut out = Vec::with_capacity(8 + self.entries.len() * ENTRY_SIZE);
          out.extend_from_slice(&MAGIC.to_le_bytes());
          out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
          for e in &self.entries {
              let mut b = [0u8; ENTRY_SIZE];
              b[0] = e.profile.id();
              b[1] = e.priority;
              b[2..10].copy_from_slice(&e.quota_us.to_le_bytes());
              let nb = e.name.as_bytes();
              let nl = nb.len().min(15);
              b[14..14 + nl].copy_from_slice(&nb[..nl]);
              out.extend_from_slice(&b);
          }
          out
      }

      /// Decode from binary wire format.
      pub fn decode(data: &[u8]) -> anyhow::Result<Self> {
          anyhow::ensure!(data.len() >= 8, "too short: {} bytes", data.len());
          let magic = u32::from_le_bytes([data[0],data[1],data[2],data[3]]);
          anyhow::ensure!(magic == MAGIC, "bad magic 0x{:08X}", magic);
          let count = u32::from_le_bytes([data[4],data[5],data[6],data[7]]) as usize;
          anyhow::ensure!(count <= MAX_ENTRIES, "entry_count {count} > max {MAX_ENTRIES}");
          anyhow::ensure!(data.len() >= 8 + count * ENTRY_SIZE, "truncated data");
          let mut entries = Vec::with_capacity(count);
          for i in 0..count {
              let o = 8 + i * ENTRY_SIZE;
              let b = &data[o..o + ENTRY_SIZE];
              let profile  = Profile::from_id(b[0]);
              let priority = b[1];
              let quota_us = u64::from_le_bytes([b[2],b[3],b[4],b[5],b[6],b[7],b[8],b[9]]);
              let nr = &b[14..30];
              let nend = nr.iter().position(|&x| x == 0).unwrap_or(16);
              let name = std::str::from_utf8(&nr[..nend])?.to_owned();
              entries.push(BlueprintEntry { name, profile, priority, quota_us });
          }
          Ok(Self { entries })
      }

      pub fn to_json(&self)       -> anyhow::Result<String>    { Ok(serde_json::to_string_pretty(self)?) }
      pub fn from_json(s: &str)   -> anyhow::Result<Self>      { Ok(serde_json::from_str(s)?) }

      /// Return a list of warning/error strings; empty means valid.
      pub fn validate(&self) -> Vec<String> {
          let mut w = Vec::new();
          let mut seen = std::collections::HashSet::new();
          if self.entries.is_empty() { w.push("blueprint has no entries".into()); }
          for (i, e) in self.entries.iter().enumerate() {
              if e.name.is_empty()   { w.push(format!("entry {i}: name is empty")); }
              if e.name.len() > 15   { w.push(format!("entry {i}: name too long")); }
              if !seen.insert(&e.name) { w.push(format!("entry {i}: duplicate name '{}'", e.name)); }
              if e.quota_us == 0     { w.push(format!("entry {i} ({}): quota_us is 0", e.name)); }
              if e.quota_us > 50_000 { w.push(format!("entry {i} ({}): quota_us {} is very high", e.name, e.quota_us)); }
          }
          w
      }
  }

  // ─── Builder ─────────────────────────────────────────────────────────────────

  #[derive(Default)]
  pub struct BlueprintBuilder { bp: Blueprint }

  impl BlueprintBuilder {
      pub fn new() -> Self { Self::default() }
      pub fn add(mut self, e: BlueprintEntry) -> anyhow::Result<Self> { self.bp.add(e)?; Ok(self) }
      pub fn gp(self, n: impl Into<String>, pri: u8, q: u64)  -> anyhow::Result<Self> { self.add(BlueprintEntry::new(n, Profile::GeneralPurpose).priority(pri).quota_us(q)) }
      pub fn rt(self, n: impl Into<String>, pri: u8, q: u64)  -> anyhow::Result<Self> { self.add(BlueprintEntry::new(n, Profile::RealTime).priority(pri).quota_us(q)) }
      pub fn net(self, n: impl Into<String>, pri: u8, q: u64) -> anyhow::Result<Self> { self.add(BlueprintEntry::new(n, Profile::Networking).priority(pri).quota_us(q)) }
      pub fn storage(self, n: impl Into<String>, pri: u8, q: u64) -> anyhow::Result<Self> { self.add(BlueprintEntry::new(n, Profile::Storage).priority(pri).quota_us(q)) }
      pub fn enclave(self, n: impl Into<String>, pri: u8, q: u64) -> anyhow::Result<Self> { self.add(BlueprintEntry::new(n, Profile::Enclave).priority(pri).quota_us(q)) }
      pub fn build(self) -> Blueprint { self.bp }
  }
  