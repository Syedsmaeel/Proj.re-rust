//! Sovereign vDisk Driver — extent-based block I/O for .tmx-disk volumes
  //!
  //! # Volume layout
  //! ```text
  //! ┌──────────────────────────────────────────┐
  //! │  TmxDiskHeader  (128 bytes, offset 0)    │
  //! ├──────────────────────────────────────────┤
  //! │  ExtentTable    (n × 24 bytes)           │
  //! ├──────────────────────────────────────────┤
  //! │  Data blocks    (512-byte sectors)       │
  //! └──────────────────────────────────────────┘
  //! ```
  //! v3 — all .unwrap() removed; uses safe le_u32/le_u64 helpers.

  extern crate alloc;
  use alloc::vec::Vec;
  use crate::priv_model::{CapRight, CapabilityToken};

  // ─── Safe byte helpers (no .unwrap()) ────────────────────────────────────────

  #[inline(always)]
  fn le_u32(raw: &[u8], off: usize) -> u32 {
      u32::from_le_bytes([raw[off], raw[off+1], raw[off+2], raw[off+3]])
  }

  #[inline(always)]
  fn le_u64(raw: &[u8], off: usize) -> u64 {
      u64::from_le_bytes([
          raw[off],raw[off+1],raw[off+2],raw[off+3],
          raw[off+4],raw[off+5],raw[off+6],raw[off+7],
      ])
  }

  // ─── Constants ────────────────────────────────────────────────────────────────

  pub const SECTOR_SIZE:    usize = 512;
  pub const MAX_EXTENTS:    usize = 1024;
  pub const MAX_PAYLOAD_SIZE: usize = 500 * 1024 * 1024; // 500 MiB

  // ─── Header ───────────────────────────────────────────────────────────────────

  #[repr(C)]
  #[derive(Debug, Clone, Copy)]
  pub struct TmxDiskHeader {
      pub magic:               [u8; 8],
      pub version:             u32,
      pub flags:               u32,
      pub encryption_salt:     [u8; 16],
      pub root_capability:     u64,
      pub extent_table_offset: u64,
      pub disk_size_sectors:   u64,
      pub extent_count:        u32,
      pub _pad:                u32,
      pub integrity_tag:       [u8; 32],
  }

  impl TmxDiskHeader {
      pub const MAGIC: [u8; 8]  = *b"TMX-DISK";
      pub const VERSION: u32    = 2;
      pub const SIZE: usize     = core::mem::size_of::<TmxDiskHeader>();

      pub fn is_valid(&self)      -> bool { self.magic == Self::MAGIC }
      pub fn is_encrypted(&self)  -> bool { self.flags & 1 != 0 }
      pub fn capacity_bytes(&self)-> u64  { self.disk_size_sectors * SECTOR_SIZE as u64 }
  }

  // ─── Extent ───────────────────────────────────────────────────────────────────

  #[repr(C)]
  #[derive(Debug, Clone, Copy, Default)]
  pub struct Extent {
      pub lba_start:   u64,
      pub lba_count:   u64,
      pub byte_offset: u64,
  }

  impl Extent {
      pub fn end_lba(&self)     -> u64  { self.lba_start + self.lba_count }
      pub fn size_bytes(&self)  -> u64  { self.lba_count * SECTOR_SIZE as u64 }
      pub fn contains(&self, lba: u64) -> bool {
          lba >= self.lba_start && lba < self.end_lba()
      }
  }

  // ─── Error ────────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum VdiskError {
      InvalidMagic,
      UnsupportedVersion,
      CapabilityDenied,
      PayloadTooLarge,
      CorruptHeader,
      ExtentTableFull,
      LbaOutOfRange,
      UnalignedAccess,
      NotMounted,
      WriteProtected,
  }

  // ─── MountedVolume ────────────────────────────────────────────────────────────

  pub struct MountedVolume {
      pub header:       TmxDiskHeader,
      extents:          Vec<Extent>,
      backing:          Vec<u8>,
      pub writable:     bool,
      pub read_ops:     u64,
      pub write_ops:    u64,
      pub bytes_read:   u64,
      pub bytes_written:u64,
  }

  impl MountedVolume {
      fn new(header: TmxDiskHeader, extents: Vec<Extent>, backing: Vec<u8>, writable: bool) -> Self {
          Self { header, extents, backing, writable,
                 read_ops: 0, write_ops: 0, bytes_read: 0, bytes_written: 0 }
      }

      fn resolve_lba(&self, lba: u64) -> Result<usize, VdiskError> {
          for ext in &self.extents {
              if ext.contains(lba) {
                  let delta = (lba - ext.lba_start) * SECTOR_SIZE as u64;
                  return Ok((ext.byte_offset + delta) as usize);
              }
          }
          Err(VdiskError::LbaOutOfRange)
      }

      pub fn read_sectors(&mut self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), VdiskError> {
          let needed = count * SECTOR_SIZE;
          if buf.len() < needed { return Err(VdiskError::UnalignedAccess); }
          for i in 0..count {
              let off = self.resolve_lba(lba + i as u64)?;
              let end = (off + SECTOR_SIZE).min(self.backing.len());
              if off >= self.backing.len() { return Err(VdiskError::LbaOutOfRange); }
              buf[i * SECTOR_SIZE..(i * SECTOR_SIZE) + (end - off)]
                  .copy_from_slice(&self.backing[off..end]);
          }
          self.read_ops  += 1;
          self.bytes_read += needed as u64;
          Ok(())
      }

      pub fn write_sectors(&mut self, lba: u64, count: usize, data: &[u8]) -> Result<(), VdiskError> {
          if !self.writable { return Err(VdiskError::WriteProtected); }
          let needed = count * SECTOR_SIZE;
          if data.len() < needed { return Err(VdiskError::UnalignedAccess); }
          for i in 0..count {
              let off = self.resolve_lba(lba + i as u64)?;
              let end = off + SECTOR_SIZE;
              if end > self.backing.len() { return Err(VdiskError::LbaOutOfRange); }
              self.backing[off..end].copy_from_slice(&data[i * SECTOR_SIZE..(i+1) * SECTOR_SIZE]);
          }
          self.write_ops    += 1;
          self.bytes_written += needed as u64;
          Ok(())
      }

      pub fn extent_count(&self) -> usize { self.extents.len() }
      pub fn capacity_bytes(&self) -> u64 { self.header.capacity_bytes() }
  }

  // ─── VDiskManager ─────────────────────────────────────────────────────────────

  pub struct VDiskManager {
      volumes: Vec<(u64, MountedVolume)>,
      next_id: u64,
  }

  impl VDiskManager {
      pub fn new() -> Self { Self { volumes: Vec::new(), next_id: 1 } }

      pub fn format(
          &mut self, cap: &CapabilityToken,
          size_sectors: u64, root_cap_id: u64, writable: bool,
      ) -> Result<u64, VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_WRITE) {
              return Err(VdiskError::CapabilityDenied);
          }
          let backing_size = (size_sectors as usize) * SECTOR_SIZE;
          if backing_size > MAX_PAYLOAD_SIZE { return Err(VdiskError::PayloadTooLarge); }

          let header = TmxDiskHeader {
              magic: TmxDiskHeader::MAGIC, version: TmxDiskHeader::VERSION,
              flags: 0, encryption_salt: [0u8; 16], root_capability: root_cap_id,
              extent_table_offset: TmxDiskHeader::SIZE as u64,
              disk_size_sectors: size_sectors, extent_count: 1,
              _pad: 0, integrity_tag: [0u8; 32],
          };
          let extent  = Extent { lba_start: 0, lba_count: size_sectors, byte_offset: 0 };
          let backing = alloc::vec![0u8; backing_size];
          let id      = self.next_id; self.next_id += 1;
          self.volumes.push((id, MountedVolume::new(header, alloc::vec![extent], backing, writable)));
          Ok(id)
      }

      /// Parse and mount a raw byte buffer as a TMX-DISK volume.
      /// Uses le_u32/le_u64 helpers — no .unwrap() anywhere.
      pub fn mount(
          &mut self, cap: &CapabilityToken, raw: Vec<u8>, writable: bool,
      ) -> Result<u64, VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_READ) {
              return Err(VdiskError::CapabilityDenied);
          }
          if writable && !cap.permits(CapRight::FS_WRITE) {
              return Err(VdiskError::CapabilityDenied);
          }
          if raw.len() < TmxDiskHeader::SIZE { return Err(VdiskError::CorruptHeader); }

          // Validate magic (direct slice, no try_into)
          if raw[0..8] != TmxDiskHeader::MAGIC { return Err(VdiskError::InvalidMagic); }

          let version      = le_u32(&raw, 8);
          if version > TmxDiskHeader::VERSION { return Err(VdiskError::UnsupportedVersion); }

          let ext_offset   = le_u64(&raw, 24) as usize;
          let disk_sectors = le_u64(&raw, 32);
          let ext_count    = le_u32(&raw, 40) as usize;

          if ext_count > MAX_EXTENTS { return Err(VdiskError::ExtentTableFull); }

          // Parse extent table — 24 bytes per entry, no try_into
          let mut extents = Vec::with_capacity(ext_count);
          for i in 0..ext_count {
              let base = ext_offset + i * 24;
              if base + 24 > raw.len() { return Err(VdiskError::CorruptHeader); }
              extents.push(Extent {
                  lba_start:   le_u64(&raw, base),
                  lba_count:   le_u64(&raw, base + 8),
                  byte_offset: le_u64(&raw, base + 16),
              });
          }

          let header = TmxDiskHeader {
              magic: TmxDiskHeader::MAGIC, version, flags: 0,
              encryption_salt: [0u8; 16], root_capability: 0,
              extent_table_offset: ext_offset as u64,
              disk_size_sectors: disk_sectors, extent_count: ext_count as u32,
              _pad: 0, integrity_tag: [0u8; 32],
          };
          let id = self.next_id; self.next_id += 1;
          self.volumes.push((id, MountedVolume::new(header, extents, raw, writable)));
          Ok(id)
      }

      pub fn unmount(&mut self, cap: &CapabilityToken, id: u64) -> Result<(), VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_READ) {
              return Err(VdiskError::CapabilityDenied);
          }
          let pos = self.volumes.iter().position(|(vid,_)| *vid == id)
              .ok_or(VdiskError::NotMounted)?;
          self.volumes.remove(pos);
          Ok(())
      }

      pub fn read(&mut self, cap: &CapabilityToken, id: u64, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_READ) {
              return Err(VdiskError::CapabilityDenied);
          }
          self.get_mut(id)?.read_sectors(lba, count, buf)
      }

      pub fn write(&mut self, cap: &CapabilityToken, id: u64, lba: u64, count: usize, data: &[u8]) -> Result<(), VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_WRITE) {
              return Err(VdiskError::CapabilityDenied);
          }
          self.get_mut(id)?.write_sectors(lba, count, data)
      }

      pub fn get(&self, id: u64) -> Result<&MountedVolume, VdiskError> {
          self.volumes.iter().find(|(vid,_)| *vid == id).map(|(_,v)| v)
              .ok_or(VdiskError::NotMounted)
      }
      fn get_mut(&mut self, id: u64) -> Result<&mut MountedVolume, VdiskError> {
          self.volumes.iter_mut().find(|(vid,_)| *vid == id).map(|(_,v)| v)
              .ok_or(VdiskError::NotMounted)
      }
      pub fn count(&self) -> usize { self.volumes.len() }
      pub fn volume_ids(&self) -> impl Iterator<Item = u64> + '_ {
          self.volumes.iter().map(|(id,_)| *id)
      }
  }

  // ─── Tests ───────────────────────────────────────────────────────────────────

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::priv_model::{CapabilityToken, CapRight};

      fn root_cap() -> CapabilityToken { CapabilityToken::root() }

      #[test]
      fn format_creates_volume() {
          let mut mgr = VDiskManager::new();
          let id = mgr.format(&root_cap(), 16, 0, true).expect("format failed");
          assert_eq!(mgr.count(), 1);
          let vol = mgr.get(id).unwrap();
          assert_eq!(vol.capacity_bytes(), 16 * SECTOR_SIZE as u64);
      }

      #[test]
      fn write_read_roundtrip() {
          let mut mgr = VDiskManager::new();
          let cap = root_cap();
          let id  = mgr.format(&cap, 4, 0, true).unwrap();

          let mut data = [0u8; SECTOR_SIZE];
          data[0] = 0xDE; data[1] = 0xAD; data[SECTOR_SIZE-1] = 0xFF;
          mgr.write(&cap, id, 0, 1, &data).expect("write failed");

          let mut buf = [0u8; SECTOR_SIZE];
          mgr.read(&cap, id, 0, 1, &mut buf).expect("read failed");
          assert_eq!(buf[0], 0xDE);
          assert_eq!(buf[1], 0xAD);
          assert_eq!(buf[SECTOR_SIZE-1], 0xFF);
      }

      #[test]
      fn write_read_multiple_sectors() {
          let mut mgr = VDiskManager::new();
          let cap = root_cap();
          let id  = mgr.format(&cap, 8, 0, true).unwrap();

          let data = (0u8..=255).cycle().take(SECTOR_SIZE * 4).collect::<alloc::vec::Vec<_>>();
          mgr.write(&cap, id, 0, 4, &data).unwrap();

          let mut buf = alloc::vec![0u8; SECTOR_SIZE * 4];
          mgr.read(&cap, id, 0, 4, &mut buf).unwrap();
          assert_eq!(buf, data);
      }

      #[test]
      fn lba_out_of_range_returns_error() {
          let mut mgr = VDiskManager::new();
          let cap = root_cap();
          let id  = mgr.format(&cap, 2, 0, true).unwrap();
          let mut buf = [0u8; SECTOR_SIZE];
          assert_eq!(mgr.read(&cap, id, 99, 1, &mut buf), Err(VdiskError::LbaOutOfRange));
      }

      #[test]
      fn write_protect_rejects_write() {
          let mut mgr = VDiskManager::new();
          let cap = root_cap();
          let id  = mgr.format(&cap, 2, 0, false).unwrap(); // read-only
          let data = [0u8; SECTOR_SIZE];
          assert_eq!(mgr.write(&cap, id, 0, 1, &data), Err(VdiskError::WriteProtected));
      }

      #[test]
      fn unmount_removes_volume() {
          let mut mgr = VDiskManager::new();
          let cap = root_cap();
          let id  = mgr.format(&cap, 2, 0, true).unwrap();
          mgr.unmount(&cap, id).unwrap();
          assert_eq!(mgr.count(), 0);
          assert_eq!(mgr.get(id).unwrap_err(), VdiskError::NotMounted);
      }
  }
  