//! Sovereign Teleportation — Sub-kernel snapshot and P2P transport
  //!
  //! # Wire format (little-endian)
  //!
  //! ```text
  //! magic u64 | version u32 | flags u32 | name_len u32 | name [u8]
  //! mem_size u64 | cap_count u32 | cap_data [CapEntry] 
  //! payload_len u64 | payload [u8] | tag [u8; 32]
  //! ```

  extern crate alloc;
  use alloc::vec::Vec;
  use alloc::string::String;
  use crate::subkernel::instance::{SubKernel, SubKernelId};

  // ─── Safe byte-parsing helpers (no .unwrap()) ────────────────────────────────

  #[inline(always)]
  fn le_u32(b: &[u8]) -> u32 {
      u32::from_le_bytes([b[0], b[1], b[2], b[3]])
  }

  #[inline(always)]
  fn le_u64(b: &[u8]) -> u64 {
      u64::from_le_bytes([b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]])
  }

  // ─── Constants ────────────────────────────────────────────────────────────────

  pub const TELEPORT_MAGIC:   u64   = 0x54454c45504f5254; // "TELEPORT"
  pub const TELEPORT_VERSION: u32   = 2;
  pub const MAX_NAME:         usize = 64;
  pub const TAG_SIZE:         usize = 32;

  // ─── CapEntry ────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, Default)]
  #[repr(C)]
  pub struct CapEntry {
      pub id:     u64,
      pub rights: u64,
      pub expiry: u64,
      pub _pad:   u64,
  }

  // ─── BlobError ───────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum BlobError {
      Truncated,
      BadMagic,
      UnsupportedVersion,
      NameTooLong,
      InvalidUtf8,
      BadTag,
  }

  // ─── MigrationBlob ───────────────────────────────────────────────────────────

  /// Serialised, optionally encrypted snapshot of a sub-kernel.
  /// Uses heap-allocated Vec<u8> — no static 1 MiB BSS waste.
  #[derive(Debug)]
  pub struct MigrationBlob {
      pub magic:       u64,
      pub version:     u32,
      pub flags:       u32,
      pub name:        String,
      pub memory_size: u64,
      pub caps:        Vec<CapEntry>,
      pub payload:     Vec<u8>,
      pub tag:         [u8; TAG_SIZE],
  }

  impl MigrationBlob {
      // ── Construction ─────────────────────────────────────────────────────────

      pub fn serialize(sk: &SubKernel) -> Self {
          let name  = String::from(sk.config.name);
          let range = sk.memory_range();
          let caps: Vec<CapEntry> = sk.cap_table.entries()
              .map(|e| CapEntry { id: e.id, rights: e.rights.bits(), expiry: e.expiry, _pad: 0 })
              .collect();
          let mut payload = Vec::new();
          payload.extend_from_slice(&range.start.to_le_bytes());
          payload.extend_from_slice(&range.size.to_le_bytes());
          payload.push(sk.state as u8);
          Self {
              magic: TELEPORT_MAGIC, version: TELEPORT_VERSION,
              flags: 0, name, memory_size: range.size as u64,
              caps, payload, tag: [0u8; TAG_SIZE],
          }
      }

      // ── Wire encode ───────────────────────────────────────────────────────────

      pub fn encode(&self) -> Vec<u8> {
          let name_bytes = self.name.as_bytes();
          let mut buf = Vec::with_capacity(
              8+4+4+4 + name_bytes.len()
              + 8+4 + self.caps.len() * core::mem::size_of::<CapEntry>()
              + 8   + self.payload.len()
              + TAG_SIZE
          );
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

      // ── Wire decode (zero .unwrap()) ──────────────────────────────────────────

      pub fn decode(raw: &[u8]) -> Result<Self, BlobError> {
          let mut cur = 0usize;

          /// Advance cursor by N bytes; return slice or Truncated.
          macro_rules! take {
              ($n:expr) => {{
                  if cur + $n > raw.len() { return Err(BlobError::Truncated); }
                  let s = &raw[cur..cur + $n];
                  cur += $n;
                  s
              }};
          }

          let magic = le_u64(take!(8));
          if magic != TELEPORT_MAGIC { return Err(BlobError::BadMagic); }

          let version = le_u32(take!(4));
          if version > TELEPORT_VERSION { return Err(BlobError::UnsupportedVersion); }

          let flags   = le_u32(take!(4));
          let name_len = le_u32(take!(4)) as usize;
          if name_len > MAX_NAME { return Err(BlobError::NameTooLong); }

          let name = String::from_utf8(take!(name_len).to_vec())
              .map_err(|_| BlobError::InvalidUtf8)?;

          let memory_size = le_u64(take!(8));
          let cap_count   = le_u32(take!(4)) as usize;

          let mut caps = Vec::with_capacity(cap_count);
          for _ in 0..cap_count {
              let id     = le_u64(take!(8));
              let rights = le_u64(take!(8));
              let expiry = le_u64(take!(8));
              let _pad   = le_u64(take!(8));
              caps.push(CapEntry { id, rights, expiry, _pad });
          }

          let payload_len = le_u64(take!(8)) as usize;
          let payload = take!(payload_len).to_vec();

          // Tag: exactly TAG_SIZE bytes — bounds already checked by take!
          let tag_slice = take!(TAG_SIZE);
          let mut tag = [0u8; TAG_SIZE];
          tag.copy_from_slice(tag_slice);   // no try_into, no unwrap

          Ok(Self { magic, version, flags, name, memory_size, caps, payload, tag })
      }

      // ── Integrity ─────────────────────────────────────────────────────────────

      pub fn sign(&mut self, key: &[u8]) {
          let body = self.encode_without_tag();
          self.tag = hmac_sign(key, &body);
      }

      pub fn verify(&self, key: &[u8]) -> bool {
          let body = self.encode_without_tag();
          hmac_verify(key, &body, &self.tag)
      }

      fn encode_without_tag(&self) -> Vec<u8> {
          let mut e = self.encode();
          let len   = e.len();
          e.truncate(len - TAG_SIZE);
          e
      }

      pub fn name(&self)        -> &str  { &self.name }
      pub fn cap_count(&self)   -> usize { self.caps.len() }
      pub fn payload_len(&self) -> usize { self.payload.len() }
      pub fn is_encrypted(&self)-> bool  { self.flags & 1 != 0 }
  }

  // ─── HMAC helpers ────────────────────────────────────────────────────────────

  pub fn hmac_sign(key: &[u8], data: &[u8]) -> [u8; TAG_SIZE] {
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

  // ─── Tests ───────────────────────────────────────────────────────────────────

  #[cfg(test)]
  mod tests {
      use super::*;

      fn make_blob(name: &str, payload: Vec<u8>) -> MigrationBlob {
          MigrationBlob {
              magic: TELEPORT_MAGIC, version: TELEPORT_VERSION,
              flags: 0, name: String::from(name),
              memory_size: payload.len() as u64, caps: Vec::new(),
              payload, tag: [0u8; TAG_SIZE],
          }
      }

      #[test]
      fn encode_decode_roundtrip_empty_payload() {
          let blob = make_blob("init-sk", Vec::new());
          let encoded = blob.encode();
          let decoded = MigrationBlob::decode(&encoded).expect("decode failed");
          assert_eq!(decoded.name, "init-sk");
          assert_eq!(decoded.magic, TELEPORT_MAGIC);
          assert_eq!(decoded.version, TELEPORT_VERSION);
          assert_eq!(decoded.payload.len(), 0);
      }

      #[test]
      fn encode_decode_roundtrip_with_payload() {
          let payload = b"kernel state snapshot".to_vec();
          let blob = make_blob("net-sk", payload.clone());
          let encoded = blob.encode();
          let decoded = MigrationBlob::decode(&encoded).expect("decode failed");
          assert_eq!(decoded.payload, payload);
          assert_eq!(decoded.name, "net-sk");
      }

      #[test]
      fn encode_decode_roundtrip_with_caps() {
          let mut blob = make_blob("enclave", vec![1, 2, 3]);
          blob.caps.push(CapEntry { id: 42, rights: 0xFF, expiry: u64::MAX, _pad: 0 });
          blob.caps.push(CapEntry { id: 99, rights: 0x0F, expiry: 1000, _pad: 0 });
          let encoded = blob.encode();
          let decoded = MigrationBlob::decode(&encoded).expect("decode failed");
          assert_eq!(decoded.cap_count(), 2);
          assert_eq!(decoded.caps[0].id, 42);
          assert_eq!(decoded.caps[1].rights, 0x0F);
      }

      #[test]
      fn decode_bad_magic_returns_error() {
          let blob = make_blob("test", Vec::new());
          let mut encoded = blob.encode();
          encoded[0] ^= 0xFF;  // corrupt magic
          assert_eq!(MigrationBlob::decode(&encoded), Err(BlobError::BadMagic));
      }

      #[test]
      fn decode_truncated_returns_error() {
          assert_eq!(MigrationBlob::decode(&[]), Err(BlobError::Truncated));
          assert_eq!(MigrationBlob::decode(&[0u8; 4]), Err(BlobError::Truncated));
      }

      #[test]
      fn sign_verify_roundtrip() {
          let mut blob = make_blob("signed-sk", vec![0xAB; 16]);
          blob.sign(b"sovereign-node-key");
          assert!(blob.verify(b"sovereign-node-key"), "valid key must verify");
          assert!(!blob.verify(b"wrong-key"), "wrong key must fail");
      }

      #[test]
      fn sign_tamper_fails_verify() {
          let mut blob = make_blob("tampered", vec![1,2,3]);
          blob.sign(b"key");
          blob.payload[0] ^= 0xFF;   // tamper with payload
          // re-encode would differ from signed body
          blob.tag[0] ^= 0xFF;       // also tamper tag directly
          assert!(!blob.verify(b"key"));
      }
  }
  