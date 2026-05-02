//! Sovereign vDisk Driver
//!
//! Implements extent-based I/O for .tmx-disk volumes in Ring -1.

use crate::priv_model::CapabilityToken;
use crate::priv_model::CapRight;

/// Maximum payload size: 500MB sovereignty limit
pub const MAX_PAYLOAD_SIZE: usize = 500 * 1024 * 1024;

/// TMX-DISK volume header
#[repr(C, packed)]
pub struct TmxDiskHeader {
    pub magic:               [u8; 8],   // "TMX-DISK"
    pub version:             u32,
    pub encryption_salt:     [u8; 16],  // Derived from Sovereign Origin
    pub root_capability:     u64,       // Cap token ID required for mount
    pub extent_table_offset: u64,       // Start of data extents
    pub disk_size_gb:        u32,       // Total virtual capacity
    pub extent_count:        u32,       // Active data chunks
    pub integrity_hash:      [u8; 64],  // Ed25519 signature
}

impl TmxDiskHeader {
    pub const MAGIC: [u8; 8] = *b"TMX-DISK";
    pub const HEADER_SIZE: usize = 128;
    pub const MAX_PAYLOAD_SIZE: usize = MAX_PAYLOAD_SIZE;

    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }
}

/// A single disk extent — a contiguous chunk of virtual data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Extent {
    pub lba_start: u64,
    pub lba_count: u64,
    pub flags:     u32,
    pub _pad:      u32,
}

/// vDisk mount result
#[derive(Debug)]
pub enum MountError {
    InvalidMagic,
    CapabilityDenied,
    PayloadTooLarge,
    CorruptHeader,
}

pub struct VDiskManager;

impl VDiskManager {
    /// Mount a .tmx-disk volume
    /// Requires FS_READ capability
    pub fn mount(
        header: &TmxDiskHeader,
        data:   &[u8],
        cap:    &CapabilityToken,
    ) -> Result<(), MountError> {
        // Verify capability
        if !cap.permits(CapRight::FS_READ) {
            return Err(MountError::CapabilityDenied);
        }

        // Verify magic
        if !header.is_valid() {
            return Err(MountError::InvalidMagic);
        }

        // Verify payload size
        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(MountError::PayloadTooLarge);
        }

        Ok(())
    }

    /// Format a new .tmx-disk volume
    pub fn format(
        buf:          &mut [u8],
        disk_size_gb: u32,
        cap:          &CapabilityToken,
    ) -> Result<(), MountError> {
        if !cap.permits(CapRight::FS_WRITE) {
            return Err(MountError::CapabilityDenied);
        }

        if buf.len() < TmxDiskHeader::HEADER_SIZE {
            return Err(MountError::CorruptHeader);
        }

        // Write magic
        buf[0..8].copy_from_slice(&TmxDiskHeader::MAGIC);
        // version = 1
        buf[8..12].copy_from_slice(&1u32.to_le_bytes());
        // disk_size_gb
        let offset = 8 + 4 + 16 + 8 + 8; // after magic+ver+salt+root_cap+extent_offset
        if offset + 4 <= buf.len() {
            buf[offset..offset+4].copy_from_slice(&disk_size_gb.to_le_bytes());
        }

        Ok(())
    }
}
