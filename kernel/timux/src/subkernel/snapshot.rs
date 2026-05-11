//! Sovereign Teleportation — Sub-kernel snapshot and P2P transport
  //!
  //! A `MigrationBlob` is the serialised state of a sub-kernel that can be
  //! sent to another sovereign node over an Onion-routed channel.
  //!
  //! # Wire format (little-endian)
  //!
  //! ```text
  //! ┌────────────────────────────────────────────────────────────┐
  //! │  magic        u64  0x54454c45504f5254  "TELEPORT"          │
  //! │  version      u32  currently 2                             │
  //! │  flags        u32  bit 0: encrypted, bit 1: compressed     │
  //! │  name_len     u32  length of subkernel name                │
  //! │  name_data    [u8; name_len]                               │
  //! │  mem_size     u64  serialised memory region size           │
  //! │  cap_count    u32  number of capability entries            │
  //! │  cap_data     [CapEntry; cap_count]                        │
  //! │  payload_len  u64  length of following payload             │
  //! │  payload      [u8; payload_len]                            │
  //! │  tag          [u8; 32]  HMAC-SovereignHash of all above    │
  //! └────────────────────────────────────────────────────────────┘
  //! ```
  //!
  //! v2 — replaces the static 1 MiB placeholder array with a dynamic Vec.

  #![allow(dead_code)]
  extern crate alloc;

  use alloc::vec::Vec;
  use alloc::string::String;
  use crate::subkernel::instance::{SubKernel, SubKernelId};

  // ─── Constants ────────────────────────────────────────────────────────────────

  pub const TELEPORT_MAGIC:   u64 = 0x54454c45504f5254; // "TELEPORT"
  pub const TELEPORT_VERSION: u32 = 2;
  pub const MAX_NAME:         usize = 64;
  pub const TAG_SIZE:         usize = 32; // HMAC-SovereignHash output

  // ─── CapEntry — a single capability record in the blob ───────────────────────

  #[derive(Debug, Clone, Copy, Default)]
  #[repr(C)]
  pub struct CapEntry {
      pub id:     u64,
      pub rights: u64,
      pub expiry: u64,
      pub _pad:   u64,
  }

  // ─── MigrationBlob ───────────────────────────────────────────────────────────

  /// Serialised, optionally encrypted snapshot of a sub-kernel ready to
  /// teleport across a sovereign P2P channel.
  ///
  /// Unlike the previous version this uses heap-allocated `Vec<u8>` for
  /// the payload — no static array means no 1 MiB BSS waste per blob.
  #[derive(Debug)]
  pub struct MigrationBlob {
      pub magic:       u64,
      pub version:     u32,
      pub flags:       u32,
      pub name:        String,
      pub memory_size: u64,
      pub caps:        Vec<CapEntry>,
      pub payload:     Vec<u8>,
      pub tag:         [u8; TAG_SIZE],  // filled by sign(), zeroed before verify
  }

  impl MigrationBlob {
      // ── Construction ─────────────────────────────────────────────────────────

      /// Serialise a sub-kernel into a migration blob.
      ///
      /// In a real implementation the payload would contain:
      ///   - register state
      ///   - dirty page frames
      ///   - IPC message queues
      ///   - IKFS mount table
      ///
      /// Here we capture the metadata exactly and set an empty payload so the
      /// type system and wire format are both correct.
      pub fn serialize(sk: &SubKernel) -> Self {
          let name   = String::from(sk.config.name);
          let range  = sk.memory_range();

          // Snapshot capability table entries
          let caps: Vec<CapEntry> = sk.cap_table.entries()
              .map(|e| CapEntry { id: e.id, rights: e.rights.bits(), expiry: e.expiry, _pad: 0 })
              .collect();

          // Build the raw payload: capture memory region header (no live ptr copy here;
          // in a real implementation an MMU-assisted freeze + copy-on-write snapshot
          // would fill this buffer).
          let mut payload = Vec::new();
          // Header marker: [start_phys: u64, size: u64]
          payload.extend_from_slice(&range.start.to_le_bytes());
          payload.extend_from_slice(&range.size.to_le_bytes());
          // Metadata: sk state u8
          payload.push(sk.state as u8);

          Self {
              magic: TELEPORT_MAGIC,
              version: TELEPORT_VERSION,
              flags: 0,
              name,
              memory_size: range.size as u64,
              caps,
              payload,
              tag: [0u8; TAG_SIZE],
          }
      }

      // ── Wire encoding ─────────────────────────────────────────────────────────

      /// Encode the blob into a flat byte vector ready for transmission.
      pub fn encode(&self) -> Vec<u8> {
          let name_bytes = self.name.as_bytes();
          let cap_bytes  = self.caps.len() * core::mem::size_of::<CapEntry>();
          let capacity   = 8+4+4+4 + name_bytes.len()
                         + 8+4 + cap_bytes
                         + 8   + self.payload.len()
                         + TAG_SIZE;

          let mut buf = Vec::with_capacity(capacity);

          buf.extend_from_slice(&self.magic.to_le_bytes());
          buf.extend_from_slice(&self.version.to_le_bytes());
          buf.extend_from_slice(&self.flags.to_le_bytes());
          buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
          buf.extend_from_slice(name_bytes);
          buf.extend_from_slice(&self.memory_size.to_le_bytes());
          buf.extend_from_slice(&(self.caps.len() as u32).to_le_bytes());
          for cap in &self.caps {
              buf.extend_from_slice(&cap.id.to_le_bytes());
              buf.extend_from_slice(&cap.rights.to_le_bytes());
              buf.extend_from_slice(&cap.expiry.to_le_bytes());
              buf.extend_from_slice(&cap._pad.to_le_bytes());
          }
          buf.extend_from_slice(&(self.payload.len() as u64).to_le_bytes());
          buf.extend_from_slice(&self.payload);
          buf.extend_from_slice(&self.tag);
          buf
      }

      // ── Wire decoding ─────────────────────────────────────────────────────────

      /// Decode a flat byte vector back into a `MigrationBlob`.
      pub fn decode(raw: &[u8]) -> Result<Self, BlobError> {
          let mut cur = 0usize;
          macro_rules! take {
              ($n:expr) => {{
                  if cur + $n > raw.len() { return Err(BlobError::Truncated); }
                  let s = &raw[cur..cur+$n]; cur += $n; s
              }}
          }
          let magic   = u64::from_le_bytes(take!(8).try_into().unwrap());
          if magic != TELEPORT_MAGIC { return Err(BlobError::BadMagic); }
          let version = u32::from_le_bytes(take!(4).try_into().unwrap());
          if version > TELEPORT_VERSION { return Err(BlobError::UnsupportedVersion); }
          let flags   = u32::from_le_bytes(take!(4).try_into().unwrap());
          let name_len= u32::from_le_bytes(take!(4).try_into().unwrap()) as usize;
          if name_len > MAX_NAME { return Err(BlobError::NameTooLong); }
          let name_raw = take!(name_len);
          let name = String::from_utf8(name_raw.to_vec()).map_err(|_| BlobError::InvalidUtf8)?;
          let memory_size = u64::from_le_bytes(take!(8).try_into().unwrap());
          let cap_count   = u32::from_le_bytes(take!(4).try_into().unwrap()) as usize;
          let mut caps = Vec::with_capacity(cap_count);
          for _ in 0..cap_count {
              let id     = u64::from_le_bytes(take!(8).try_into().unwrap());
              let rights = u64::from_le_bytes(take!(8).try_into().unwrap());
              let expiry = u64::from_le_bytes(take!(8).try_into().unwrap());
              let _pad   = u64::from_le_bytes(take!(8).try_into().unwrap());
              caps.push(CapEntry { id, rights, expiry, _pad });
          }
          let payload_len = u64::from_le_bytes(take!(8).try_into().unwrap()) as usize;
          let payload = take!(payload_len).to_vec();
          let tag: [u8; TAG_SIZE] = take!(TAG_SIZE).try_into().map_err(|_| BlobError::Truncated)?;
          Ok(Self { magic, version, flags, name, memory_size, caps, payload, tag })
      }

      // ── Integrity ─────────────────────────────────────────────────────────────

      /// Sign the blob by computing its HMAC tag.
      /// `key` should be the node's long-term signing key.
      pub fn sign(&mut self, key: &[u8]) {
          use crate::subkernel::snapshot::hmac_sign;
          let body = self.encode_without_tag();
          self.tag = hmac_sign(key, &body);
      }

      /// Verify the HMAC tag.
      pub fn verify(&self, key: &[u8]) -> bool {
          use crate::subkernel::snapshot::hmac_verify;
          let body = self.encode_without_tag();
          hmac_verify(key, &body, &self.tag)
      }

      fn encode_without_tag(&self) -> Vec<u8> {
          let mut e = self.encode();
          let len   = e.len();
          e.truncate(len - TAG_SIZE);
          e
      }

      pub fn name(&self) -> &str { &self.name }
      pub fn cap_count(&self) -> usize { self.caps.len() }
      pub fn payload_len(&self) -> usize { self.payload.len() }
      pub fn is_encrypted(&self) -> bool { self.flags & 1 != 0 }
  }

  // ─── Error ────────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum BlobError {
      Truncated,
      BadMagic,
      UnsupportedVersion,
      NameTooLong,
      InvalidUtf8,
      BadTag,
  }

  // ─── Helpers (thin wrappers so snapshot.rs compiles standalone) ───────────────

  /// Compute HMAC-SovereignHash — delegates to tbm::crypto in a real build.
  /// Here we provide a stand-alone pure-Rust implementation.
  pub fn hmac_sign(key: &[u8], data: &[u8]) -> [u8; TAG_SIZE] {
      // XOR-based lightweight MAC for no_std (full HMAC-SH in tbm/crypto.rs).
      let mut h = [0u8; 32];
      let klen = key.len().min(32);
      h[..klen].copy_from_slice(&key[..klen]);
      for (i, b) in data.iter().enumerate() {
          h[i % 32] ^= b.wrapping_add(i as u8);
          h[(i + 1) % 32] = h[(i+1)%32].rotate_left(3);
      }
      h
  }

  pub fn hmac_verify(key: &[u8], data: &[u8], tag: &[u8; TAG_SIZE]) -> bool {
      let expected = hmac_sign(key, data);
      let mut diff = 0u8;
      for (a, b) in expected.iter().zip(tag.iter()) { diff |= a ^ b; }
      diff == 0
  }
  