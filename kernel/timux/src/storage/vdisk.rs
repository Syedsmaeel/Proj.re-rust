//! Sovereign vDisk Driver
  //!
  //! Implements extent-based block I/O for .tmx-disk volumes.
  //!
  //! # Volume layout
  //!
  //! ```text
  //! ┌──────────────────────────────────────────┐
  //! │  TmxDiskHeader  (128 bytes, offset 0)    │
  //! ├──────────────────────────────────────────┤
  //! │  ExtentTable    (n × 24 bytes)           │
  //! │  (at header.extent_table_offset)         │
  //! ├──────────────────────────────────────────┤
  //! │  Data blocks    (512-byte sectors)       │
  //! └──────────────────────────────────────────┘
  //! ```
  //!
  //! v2 — implements mount, read, write, format, and extent management.

  #![allow(dead_code)]
  extern crate alloc;

  use alloc::vec::Vec;
  use crate::priv_model::{CapRight, CapabilityToken};

  // ─── Constants ────────────────────────────────────────────────────────────────

  pub const SECTOR_SIZE: usize = 512;
  pub const MAX_EXTENTS: usize = 1024;
  /// 500 MiB default maximum payload
  pub const MAX_PAYLOAD_SIZE: usize = 500 * 1024 * 1024;

  // ─── Header ───────────────────────────────────────────────────────────────────

  #[repr(C)]
  #[derive(Debug, Clone, Copy)]
  pub struct TmxDiskHeader {
      pub magic:               [u8; 8],   // "TMX-DISK"
      pub version:             u32,
      pub flags:               u32,       // bit 0: encrypted
      pub encryption_salt:     [u8; 16],
      pub root_capability:     u64,
      pub extent_table_offset: u64,       // byte offset of ExtentTable
      pub disk_size_sectors:   u64,       // total virtual capacity in sectors
      pub extent_count:        u32,
      pub _pad:                u32,
      pub integrity_tag:       [u8; 32],  // HMAC-SovereignHash over header[0..96]
  }

  impl TmxDiskHeader {
      pub const MAGIC: [u8; 8] = *b"TMX-DISK";
      pub const VERSION: u32   = 2;
      pub const SIZE: usize    = core::mem::size_of::<TmxDiskHeader>();

      pub fn is_valid(&self) -> bool { self.magic == Self::MAGIC }
      pub fn is_encrypted(&self) -> bool { self.flags & 1 != 0 }
      pub fn capacity_bytes(&self) -> u64 { self.disk_size_sectors * SECTOR_SIZE as u64 }
  }

  // ─── Extent ───────────────────────────────────────────────────────────────────

  /// A single data extent — contiguous run of sectors.
  #[repr(C)]
  #[derive(Debug, Clone, Copy, Default)]
  pub struct Extent {
      pub lba_start:  u64,  // first logical block address
      pub lba_count:  u64,  // number of sectors
      pub byte_offset:u64,  // byte offset into backing store for this extent
  }

  impl Extent {
      pub fn end_lba(&self) -> u64 { self.lba_start + self.lba_count }
      pub fn size_bytes(&self) -> u64 { self.lba_count * SECTOR_SIZE as u64 }

      /// True if `lba` falls within this extent.
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
      AlreadyMounted,
      WriteProtected,
  }

  // ─── MountedVolume ────────────────────────────────────────────────────────────

  /// An in-memory representation of a mounted TMX-DISK volume.
  pub struct MountedVolume {
      pub header:   TmxDiskHeader,
      extents:      Vec<Extent>,
      /// Backing byte buffer (simulates raw disk storage in-kernel).
      backing:      Vec<u8>,
      pub writable: bool,
      pub read_ops: u64,
      pub write_ops:u64,
      pub bytes_read:    u64,
      pub bytes_written: u64,
  }

  impl MountedVolume {
      fn new(header: TmxDiskHeader, extents: Vec<Extent>, backing: Vec<u8>, writable: bool) -> Self {
          Self { header, extents, backing, writable,
                 read_ops: 0, write_ops: 0, bytes_read: 0, bytes_written: 0 }
      }

      /// Resolve an LBA to a byte offset in the backing store.
      fn resolve_lba(&self, lba: u64) -> Result<usize, VdiskError> {
          for ext in &self.extents {
              if ext.contains(lba) {
                  let delta = (lba - ext.lba_start) * SECTOR_SIZE as u64;
                  return Ok((ext.byte_offset + delta) as usize);
              }
          }
          Err(VdiskError::LbaOutOfRange)
      }

      /// Read `count` sectors starting at `lba` into `buf`.
      pub fn read_sectors(&mut self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), VdiskError> {
          let needed = count * SECTOR_SIZE;
          if buf.len() < needed { return Err(VdiskError::UnalignedAccess); }
          for i in 0..count {
              let off = self.resolve_lba(lba + i as u64)?;
              let end = (off + SECTOR_SIZE).min(self.backing.len());
              if off >= self.backing.len() { return Err(VdiskError::LbaOutOfRange); }
              let src = &self.backing[off..end];
              let dst = &mut buf[i * SECTOR_SIZE..(i * SECTOR_SIZE) + src.len()];
              dst.copy_from_slice(src);
          }
          self.read_ops  += 1;
          self.bytes_read += needed as u64;
          Ok(())
      }

      /// Write `count` sectors starting at `lba` from `data`.
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
          self.write_ops   += 1;
          self.bytes_written += needed as u64;
          Ok(())
      }

      pub fn extent_count(&self) -> usize { self.extents.len() }
      pub fn capacity_bytes(&self) -> u64 { self.header.capacity_bytes() }
  }

  // ─── VDiskManager ─────────────────────────────────────────────────────────────

  /// Kernel-global vDisk manager — owns all mounted volumes.
  pub struct VDiskManager {
      volumes: Vec<(u64, MountedVolume)>,  // (volume_id, volume)
      next_id: u64,
  }

  impl VDiskManager {
      pub fn new() -> Self { Self { volumes: Vec::new(), next_id: 1 } }

      // ── Format ────────────────────────────────────────────────────────────────

      /// Allocate a fresh in-memory TMX-DISK volume with `size_sectors` capacity.
      ///
      /// Requires FS_WRITE capability.
      pub fn format(
          &mut self,
          cap:          &CapabilityToken,
          size_sectors: u64,
          root_cap_id:  u64,
          writable:     bool,
      ) -> Result<u64, VdiskError> {
          if !cap.is_valid()                { return Err(VdiskError::CapabilityDenied); }
          if !cap.permits(CapRight::FS_WRITE) { return Err(VdiskError::CapabilityDenied); }

          let backing_size = (size_sectors as usize) * SECTOR_SIZE;
          if backing_size > MAX_PAYLOAD_SIZE { return Err(VdiskError::PayloadTooLarge); }

          let header = TmxDiskHeader {
              magic:               TmxDiskHeader::MAGIC,
              version:             TmxDiskHeader::VERSION,
              flags:               0,
              encryption_salt:     [0u8; 16],
              root_capability:     root_cap_id,
              extent_table_offset: TmxDiskHeader::SIZE as u64,
              disk_size_sectors:   size_sectors,
              extent_count:        1,
              _pad:                0,
              integrity_tag:       [0u8; 32],
          };

          // One extent covering the whole disk
          let extent = Extent { lba_start: 0, lba_count: size_sectors, byte_offset: 0 };
          let backing = alloc::vec![0u8; backing_size];

          let vol = MountedVolume::new(header, alloc::vec![extent], backing, writable);
          let id  = self.next_id;
          self.next_id += 1;
          self.volumes.push((id, vol));
          Ok(id)
      }

      // ── Mount ─────────────────────────────────────────────────────────────────

      /// Parse and mount a raw byte buffer as a TMX-DISK volume.
      ///
      /// Requires FS_READ capability; FS_WRITE also required for writable mount.
      pub fn mount(
          &mut self,
          cap:      &CapabilityToken,
          raw:      Vec<u8>,
          writable: bool,
      ) -> Result<u64, VdiskError> {
          if !cap.is_valid()               { return Err(VdiskError::CapabilityDenied); }
          if !cap.permits(CapRight::FS_READ) { return Err(VdiskError::CapabilityDenied); }
          if writable && !cap.permits(CapRight::FS_WRITE) { return Err(VdiskError::CapabilityDenied); }
          if raw.len() < TmxDiskHeader::SIZE { return Err(VdiskError::CorruptHeader); }

          // Parse header
          let magic: [u8; 8] = raw[0..8].try_into().map_err(|_| VdiskError::CorruptHeader)?;
          if magic != TmxDiskHeader::MAGIC { return Err(VdiskError::InvalidMagic); }
          let version = u32::from_le_bytes(raw[8..12].try_into().unwrap());
          if version > TmxDiskHeader::VERSION { return Err(VdiskError::UnsupportedVersion); }

          let ext_offset = u64::from_le_bytes(raw[24..32].try_into().unwrap()) as usize;
          let disk_sectors = u64::from_le_bytes(raw[32..40].try_into().unwrap());
          let ext_count   = u32::from_le_bytes(raw[40..44].try_into().unwrap()) as usize;

          if ext_count > MAX_EXTENTS { return Err(VdiskError::ExtentTableFull); }

          // Parse extent table
          let mut extents = Vec::with_capacity(ext_count);
          for i in 0..ext_count {
              let base = ext_offset + i * 24;
              if base + 24 > raw.len() { return Err(VdiskError::CorruptHeader); }
              let lba_start   = u64::from_le_bytes(raw[base..base+8].try_into().unwrap());
              let lba_count   = u64::from_le_bytes(raw[base+8..base+16].try_into().unwrap());
              let byte_offset = u64::from_le_bytes(raw[base+16..base+24].try_into().unwrap());
              extents.push(Extent { lba_start, lba_count, byte_offset });
          }

          let mut header = TmxDiskHeader {
              magic, version, flags: 0, encryption_salt: [0u8;16],
              root_capability: 0, extent_table_offset: ext_offset as u64,
              disk_size_sectors: disk_sectors, extent_count: ext_count as u32,
              _pad: 0, integrity_tag: [0u8; 32],
          };

          let vol = MountedVolume::new(header, extents, raw, writable);
          let id  = self.next_id; self.next_id += 1;
          self.volumes.push((id, vol));
          Ok(id)
      }

      // ── Unmount ───────────────────────────────────────────────────────────────

      pub fn unmount(&mut self, cap: &CapabilityToken, id: u64) -> Result<(), VdiskError> {
          if !cap.is_valid()               { return Err(VdiskError::CapabilityDenied); }
          if !cap.permits(CapRight::FS_READ) { return Err(VdiskError::CapabilityDenied); }
          let pos = self.volumes.iter().position(|(vid, _)| *vid == id).ok_or(VdiskError::NotMounted)?;
          self.volumes.remove(pos);
          Ok(())
      }

      // ── Read / Write ──────────────────────────────────────────────────────────

      pub fn read(&mut self, cap: &CapabilityToken, id: u64, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_READ) { return Err(VdiskError::CapabilityDenied); }
          let vol = self.get_mut(id)?;
          vol.read_sectors(lba, count, buf)
      }

      pub fn write(&mut self, cap: &CapabilityToken, id: u64, lba: u64, count: usize, data: &[u8]) -> Result<(), VdiskError> {
          if !cap.is_valid() || !cap.permits(CapRight::FS_WRITE) { return Err(VdiskError::CapabilityDenied); }
          let vol = self.get_mut(id)?;
          vol.write_sectors(lba, count, data)
      }

      // ── Query ─────────────────────────────────────────────────────────────────

      pub fn get(&self, id: u64) -> Result<&MountedVolume, VdiskError> {
          self.volumes.iter().find(|(vid, _)| *vid == id).map(|(_, v)| v).ok_or(VdiskError::NotMounted)
      }

      fn get_mut(&mut self, id: u64) -> Result<&mut MountedVolume, VdiskError> {
          self.volumes.iter_mut().find(|(vid, _)| *vid == id).map(|(_, v)| v).ok_or(VdiskError::NotMounted)
      }

      pub fn count(&self) -> usize { self.volumes.len() }

      pub fn volume_ids(&self) -> impl Iterator<Item = u64> + '_ {
          self.volumes.iter().map(|(id, _)| *id)
      }
  }
  