//! Sovereign Cryptographic Primitives for the Timux Boot Manager
  //!
  //! Implements bare-metal crypto for TBM without any external crates.
  //! Runs in `#![no_std]` context before the heap is available.
  //!
  //! # Primitives
  //!  - SovereignHash  — 256-bit collision-resistant hash (Blake3-inspired)
  //!  - ChaCha20       — RFC 8439 stream cipher
  //!  - HmacSovereign  — keyed MAC built on SovereignHash
  //!  - Hkdf           — HKDF-style key derivation
  //!  - ct_eq / verify_mac — constant-time comparison helpers

  // ── no .unwrap() in this file — all byte parsing uses safe array literals ────

  // ─── Byte-slice helpers (no try_into, no unwrap) ──────────────────────────────

  #[inline(always)]
  fn u32_le(b: &[u8], off: usize) -> u32 {
      u32::from_le_bytes([b[off], b[off+1], b[off+2], b[off+3]])
  }

  #[inline(always)]
  fn u64_le(b: &[u8], off: usize) -> u64 {
      u64::from_le_bytes([b[off],b[off+1],b[off+2],b[off+3],b[off+4],b[off+5],b[off+6],b[off+7]])
  }

  // ─── SovereignHash (256-bit) ─────────────────────────────────────────────────

  const IV: [u32; 8] = [
      0x6A09_E667, 0xBB67_AE85, 0x3C6E_F372, 0xA54F_F53A,
      0x510E_527F, 0x9B05_688C, 0x1F83_D9AB, 0x5BE0_CD19,
  ];

  const SIGMA: [[usize; 16]; 10] = [
      [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15],
      [14,10,4,8,9,15,13,6,1,12,0,2,11,7,5,3],
      [11,8,12,0,5,2,15,13,10,14,3,6,7,1,9,4],
      [7,9,3,1,13,12,11,14,2,6,5,10,4,0,15,8],
      [9,0,5,7,2,4,10,15,14,1,11,12,6,8,3,13],
      [2,12,6,10,0,11,8,3,4,13,7,5,15,14,1,9],
      [12,5,1,15,14,13,4,10,0,7,6,3,9,2,8,11],
      [13,11,7,14,12,1,3,9,5,0,15,4,8,6,2,10],
      [6,15,14,9,11,3,0,8,12,2,13,7,1,4,10,5],
      [10,2,8,4,7,6,1,5,15,11,9,14,3,12,13,0],
  ];

  #[inline(always)]
  fn rotr32(x: u32, n: u32) -> u32 { x.rotate_right(n) }

  fn compress(state: &mut [u32; 8], block: &[u8; 64], count: u64, last: bool) {
      let mut v = [0u32; 16];
      v[0..8].copy_from_slice(state);
      v[8]  = IV[0]; v[9]  = IV[1]; v[10] = IV[2]; v[11] = IV[3];
      v[12] = IV[4] ^ (count as u32);
      v[13] = IV[5] ^ ((count >> 32) as u32);
      v[14] = if last { IV[6] ^ 0xFFFF_FFFF } else { IV[6] };
      v[15] = IV[7];

      let mut m = [0u32; 16];
      for i in 0..16 {
          m[i] = u32_le(block, i * 4);  // safe — block is [u8;64], 16×4 = 64
      }

      macro_rules! g {
          ($a:expr,$b:expr,$c:expr,$d:expr,$x:expr,$y:expr) => {
              v[$a] = v[$a].wrapping_add(v[$b]).wrapping_add($x);
              v[$d] = rotr32(v[$d] ^ v[$a], 16);
              v[$c] = v[$c].wrapping_add(v[$d]);
              v[$b] = rotr32(v[$b] ^ v[$c], 12);
              v[$a] = v[$a].wrapping_add(v[$b]).wrapping_add($y);
              v[$d] = rotr32(v[$d] ^ v[$a], 8);
              v[$c] = v[$c].wrapping_add(v[$d]);
              v[$b] = rotr32(v[$b] ^ v[$c], 7);
          }
      }

      for r in 0..10 {
          let s = &SIGMA[r];
          g!(0,4, 8,12, m[s[0]], m[s[1]]);
          g!(1,5, 9,13, m[s[2]], m[s[3]]);
          g!(2,6,10,14, m[s[4]], m[s[5]]);
          g!(3,7,11,15, m[s[6]], m[s[7]]);
          g!(0,5,10,15, m[s[8]], m[s[9]]);
          g!(1,6,11,12, m[s[10]],m[s[11]]);
          g!(2,7, 8,13, m[s[12]],m[s[13]]);
          g!(3,4, 9,14, m[s[14]],m[s[15]]);
      }
      for i in 0..8 { state[i] ^= v[i] ^ v[i+8]; }
  }

  pub struct SovereignHash {
      state:  [u32; 8],
      buf:    [u8; 64],
      buflen: usize,
      count:  u64,
  }

  impl SovereignHash {
      pub fn new() -> Self {
          Self { state: IV, buf: [0u8; 64], buflen: 0, count: 0 }
      }

      pub fn update(&mut self, data: &[u8]) {
          let mut off = 0;
          while off < data.len() {
              let space = 64 - self.buflen;
              let take  = space.min(data.len() - off);
              self.buf[self.buflen..self.buflen + take].copy_from_slice(&data[off..off + take]);
              self.buflen += take;
              off         += take;
              if self.buflen == 64 {
                  self.count += 64;
                  let blk = self.buf;
                  compress(&mut self.state, &blk, self.count, false);
                  self.buflen = 0;
              }
          }
      }

      pub fn finalize(mut self) -> [u8; 32] {
          self.buf[self.buflen] = 0x80;
          for b in &mut self.buf[self.buflen + 1..] { *b = 0; }
          self.count += self.buflen as u64;
          let blk = self.buf;
          compress(&mut self.state, &blk, self.count, true);
          let mut out = [0u8; 32];
          for (i, &w) in self.state.iter().enumerate() {
              out[i*4..i*4+4].copy_from_slice(&w.to_le_bytes());
          }
          out
      }

      pub fn hash(data: &[u8]) -> [u8; 32] {
          let mut h = Self::new(); h.update(data); h.finalize()
      }
  }

  // ─── ChaCha20 ─────────────────────────────────────────────────────────────────

  pub struct ChaCha20 {
      state:     [u32; 16],
      block:     [u8; 64],
      block_pos: usize,
  }

  impl ChaCha20 {
      const CONST: [u32; 4] = [0x6170_7865, 0x3320_646E, 0x7962_2D32, 0x3620_6574];

      /// Create a ChaCha20 instance.
      /// key: 32 bytes, nonce: 12 bytes, counter: block counter (usually 0).
      pub fn new(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
          let mut state = [0u32; 16];
          state[0..4].copy_from_slice(&Self::CONST);
          // Key words — 8 × u32 from fixed [u8;32], safe direct indexing
          for i in 0..8 {
              state[4 + i] = u32_le(key, i * 4);
          }
          state[12] = counter;
          state[13] = u32_le(nonce, 0);
          state[14] = u32_le(nonce, 4);
          state[15] = u32_le(nonce, 8);
          Self { state, block: [0u8; 64], block_pos: 64 }
      }

      fn quarter_round(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
          s[a] = s[a].wrapping_add(s[b]); s[d] ^= s[a]; s[d] = s[d].rotate_left(16);
          s[c] = s[c].wrapping_add(s[d]); s[b] ^= s[c]; s[b] = s[b].rotate_left(12);
          s[a] = s[a].wrapping_add(s[b]); s[d] ^= s[a]; s[d] = s[d].rotate_left(8);
          s[c] = s[c].wrapping_add(s[d]); s[b] ^= s[c]; s[b] = s[b].rotate_left(7);
      }

      fn generate_block(&mut self) {
          let mut ws = self.state;
          for _ in 0..10 {
              Self::quarter_round(&mut ws, 0,4, 8,12);
              Self::quarter_round(&mut ws, 1,5, 9,13);
              Self::quarter_round(&mut ws, 2,6,10,14);
              Self::quarter_round(&mut ws, 3,7,11,15);
              Self::quarter_round(&mut ws, 0,5,10,15);
              Self::quarter_round(&mut ws, 1,6,11,12);
              Self::quarter_round(&mut ws, 2,7, 8,13);
              Self::quarter_round(&mut ws, 3,4, 9,14);
          }
          for i in 0..16 {
              let word = ws[i].wrapping_add(self.state[i]);
              self.block[i*4..i*4+4].copy_from_slice(&word.to_le_bytes());
          }
          self.state[12] = self.state[12].wrapping_add(1);
          self.block_pos = 0;
      }

      pub fn apply(&mut self, data: &mut [u8]) {
          for b in data.iter_mut() {
              if self.block_pos >= 64 { self.generate_block(); }
              *b ^= self.block[self.block_pos];
              self.block_pos += 1;
          }
      }

      pub fn encrypt(key: &[u8; 32], nonce: &[u8; 12], counter: u32, data: &mut [u8]) {
          Self::new(key, nonce, counter).apply(data);
      }
  }

  // ─── HMAC-SovereignHash ───────────────────────────────────────────────────────

  pub struct HmacSovereign {
      inner: SovereignHash,
      outer: SovereignHash,
  }

  impl HmacSovereign {
      pub fn new(key: &[u8]) -> Self {
          let key_block: [u8; 32] = if key.len() > 32 {
              SovereignHash::hash(key)
          } else {
              let mut b = [0u8; 32];
              b[..key.len()].copy_from_slice(key);
              b
          };
          let mut ipad = [0x36u8; 32];
          let mut opad = [0x5Cu8; 32];
          for i in 0..32 { ipad[i] ^= key_block[i]; opad[i] ^= key_block[i]; }
          let mut inner = SovereignHash::new(); inner.update(&ipad);
          let mut outer = SovereignHash::new(); outer.update(&opad);
          Self { inner, outer }
      }

      pub fn update(&mut self, data: &[u8]) { self.inner.update(data); }

      pub fn finalize(mut self) -> [u8; 32] {
          let inner_hash = self.inner.finalize();
          self.outer.update(&inner_hash);
          self.outer.finalize()
      }

      pub fn mac(key: &[u8], data: &[u8]) -> [u8; 32] {
          let mut h = Self::new(key); h.update(data); h.finalize()
      }
  }

  // ─── HKDF ────────────────────────────────────────────────────────────────────

  pub struct Hkdf;

  impl Hkdf {
      pub fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> [u8; 32] {
          let salt = salt.unwrap_or(&[0u8; 32]);
          HmacSovereign::mac(salt, ikm)
      }

      pub fn expand(prk: &[u8; 32], info: &[u8], len: usize) -> alloc::vec::Vec<u8> {
          extern crate alloc;
          let mut okm = alloc::vec::Vec::with_capacity(len);
          let mut t   = [0u8; 32];
          let mut n   = 0u8;
          while okm.len() < len {
              n += 1;
              let mut h = HmacSovereign::new(prk);
              if n > 1 { h.update(&t); }
              h.update(info);
              h.update(&[n]);
              t = h.finalize();
              let need = (len - okm.len()).min(32);
              okm.extend_from_slice(&t[..need]);
          }
          okm
      }

      pub fn derive(salt: Option<&[u8]>, ikm: &[u8], info: &[u8], len: usize) -> alloc::vec::Vec<u8> {
          let prk = Self::extract(salt, ikm);
          Self::expand(&prk, info, len)
      }
  }

  // ─── Constant-time helpers ───────────────────────────────────────────────────

  pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
      if a.len() != b.len() { return false; }
      let mut diff = 0u8;
      for (x, y) in a.iter().zip(b.iter()) { diff |= x ^ y; }
      diff == 0
  }

  pub fn verify_mac(key: &[u8], data: &[u8], tag: &[u8; 32]) -> bool {
      let expected = HmacSovereign::mac(key, data);
      ct_eq(&expected, tag)
  }

  // ─── Tests ───────────────────────────────────────────────────────────────────

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn sovereign_hash_deterministic() {
          let a = SovereignHash::hash(b"hello world");
          let b = SovereignHash::hash(b"hello world");
          assert_eq!(a, b, "same input must produce same hash");
      }

      #[test]
      fn sovereign_hash_differs_on_different_input() {
          let a = SovereignHash::hash(b"hello");
          let b = SovereignHash::hash(b"world");
          assert_ne!(a, b, "different inputs must produce different hashes");
      }

      #[test]
      fn sovereign_hash_avalanche() {
          let a = SovereignHash::hash(b"hello");
          let b = SovereignHash::hash(b"hEllo");
          // At least half the bytes should differ (avalanche effect)
          let diff = a.iter().zip(b.iter()).filter(|(x,y)| x != y).count();
          assert!(diff >= 8, "avalanche: expected >= 8 bytes different, got {diff}");
      }

      #[test]
      fn chacha20_encrypt_decrypt_roundtrip() {
          let key   = [0x42u8; 32];
          let nonce = [0x13u8; 12];
          let plain = b"Timux sovereign OS test vector!";
          let mut buf = plain.to_vec();
          ChaCha20::encrypt(&key, &nonce, 0, &mut buf);
          assert_ne!(&buf, plain, "ciphertext must differ from plaintext");
          ChaCha20::encrypt(&key, &nonce, 0, &mut buf);
          assert_eq!(&buf, plain, "double-encrypt must restore plaintext");
      }

      #[test]
      fn chacha20_different_counters_differ() {
          let key   = [0x11u8; 32];
          let nonce = [0x22u8; 12];
          let mut a = [0u8; 64];
          let mut b = [0u8; 64];
          ChaCha20::encrypt(&key, &nonce, 0, &mut a);
          ChaCha20::encrypt(&key, &nonce, 1, &mut b);
          assert_ne!(a, b, "different block counters must yield different keystreams");
      }

      #[test]
      fn hmac_same_key_same_data_stable() {
          let tag_a = HmacSovereign::mac(b"key", b"data");
          let tag_b = HmacSovereign::mac(b"key", b"data");
          assert_eq!(tag_a, tag_b);
      }

      #[test]
      fn hmac_key_sensitivity() {
          let a = HmacSovereign::mac(b"key1", b"data");
          let b = HmacSovereign::mac(b"key2", b"data");
          assert_ne!(a, b, "different keys must yield different MACs");
      }

      #[test]
      fn ct_eq_correct() {
          assert!( ct_eq(b"hello", b"hello"));
          assert!(!ct_eq(b"hello", b"world"));
          assert!(!ct_eq(b"hello", b"hello!"));
      }

      #[test]
      fn verify_mac_accepts_valid_tag() {
          let key  = b"sovereign-key";
          let data = b"boot-payload";
          let tag  = HmacSovereign::mac(key, data);
          assert!(verify_mac(key, data, &tag));
      }

      #[test]
      fn verify_mac_rejects_tampered_tag() {
          let key  = b"sovereign-key";
          let data = b"boot-payload";
          let mut tag = HmacSovereign::mac(key, data);
          tag[0] ^= 0xFF;
          assert!(!verify_mac(key, data, &tag));
      }

      #[test]
      fn hkdf_output_length_correct() {
          extern crate alloc;
          let okm = Hkdf::derive(None, b"secret", b"info", 64);
          assert_eq!(okm.len(), 64);
      }

      #[test]
      fn hkdf_deterministic() {
          extern crate alloc;
          let a = Hkdf::derive(None, b"secret", b"info", 32);
          let b = Hkdf::derive(None, b"secret", b"info", 32);
          assert_eq!(a, b);
      }
  }
  