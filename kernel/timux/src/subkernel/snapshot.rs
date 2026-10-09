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
  use hmac::{Hmac, Mac};
  use sha2::Sha256;
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

  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
  #[derive(Debug, PartialEq, Eq)]
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
          // Capability tokens carry no expiry, so they are exported as
          // non-expiring (u64::MAX).
          let caps: Vec<CapEntry> = sk.caps.entries()
              .map(|t| CapEntry { id: t.id(), rights: t.rights().bits(), expiry: u64::MAX, _pad: 0 })
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
                  // checked_add: lengths come from untrusted input and must not
                  // be able to wrap around the bounds check.
                  let end = match cur.checked_add($n) {
                      Some(e) if e <= raw.len() => e,
                      _ => return Err(BlobError::Truncated),
                  };
                  let s = &raw[cur..end];
                  cur = end;
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

          // Each entry is 32 bytes on the wire; never pre-allocate more than
          // the input could possibly contain.
          let mut caps = Vec::with_capacity(cap_count.min(raw.len() / 32));
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

  type HmacSha256 = Hmac<Sha256>;

  /// HMAC-SHA256 over `data` (RFC 2104). Keys of any length are accepted.
  pub fn hmac_sign(key: &[u8], data: &[u8]) -> [u8; TAG_SIZE] {
      let mut mac = HmacSha256::new_from_slice(key)
          .expect("HMAC accepts keys of any length");
      mac.update(data);
      mac.finalize().into_bytes().into()
  }

  /// Constant-time verification of an HMAC-SHA256 tag.
  pub fn hmac_verify(key: &[u8], data: &[u8], tag: &[u8; TAG_SIZE]) -> bool {
      let mut mac = HmacSha256::new_from_slice(key)
          .expect("HMAC accepts keys of any length");
      mac.update(data);
      mac.verify_slice(tag).is_ok()
  }

  // ─── Tests ───────────────────────────────────────────────────────────────────

  #[cfg(test)]
  mod tests {
      use super::*;
      use alloc::vec;

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

      #[test]
      fn decode_huge_lengths_do_not_overflow_or_allocate() {
          let blob = make_blob("x", vec![1, 2, 3]);
          let mut raw = blob.encode();
          // payload_len sits right after: 8+4+4+4 + name(1) + 8 + 4 (cap_count=0)
          let off = 8 + 4 + 4 + 4 + 1 + 8 + 4;
          raw[off..off + 8].copy_from_slice(&u64::MAX.to_le_bytes());
          assert_eq!(MigrationBlob::decode(&raw), Err(BlobError::Truncated));

          // absurd cap_count must fail cleanly, not try to reserve gigabytes
          let mut raw2 = make_blob("x", Vec::new()).encode();
          let cc = 8 + 4 + 4 + 4 + 1 + 8;
          raw2[cc..cc + 4].copy_from_slice(&u32::MAX.to_le_bytes());
          assert_eq!(MigrationBlob::decode(&raw2), Err(BlobError::Truncated));
      }

      fn hex(b: &[u8]) -> String {
          let mut s = String::new();
          for x in b { s.push_str(&alloc::format!("{:02x}", x)); }
          s
      }

      #[test]
      fn hmac_matches_rfc4231_vectors() {
          // RFC 4231 test case 1
          let t1 = hmac_sign(&[0x0b; 20], b"Hi There");
          assert_eq!(hex(&t1), "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
          // RFC 4231 test case 2
          let t2 = hmac_sign(b"Jefe", b"what do ya want for nothing?");
          assert_eq!(hex(&t2), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
      }

      #[test]
      fn hmac_uses_the_whole_key_and_rejects_forgeries() {
          let mut k1 = [7u8; 64];
          let mut k2 = [7u8; 64];
          k2[63] = 8; // differs only after byte 32 (the old code ignored it)
          assert_ne!(hmac_sign(&k1, b"data"), hmac_sign(&k2, b"data"));

          let tag = hmac_sign(&k1, b"data");
          assert!(hmac_verify(&k1, b"data", &tag));
          assert!(!hmac_verify(&k1, b"datb", &tag));
          k1[0] ^= 1;
          assert!(!hmac_verify(&k1, b"data", &tag));
      }
  }
  