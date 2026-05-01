//! Sovereign Ingestion Gate for TBM
//!
//! Handles discovery and "mounting" of .iso and .img assets from the 
//! UEFI file system for sub-kernel deployment.

use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileInfo};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::ScopedProtocol;
use log::info;

pub struct IngestionGate {
    fs: ScopedProtocol<SimpleFileSystem>,
}

#[derive(Debug, Clone, Copy)]
pub enum PartitionType {
    Gpt,
    Mbr,
}

#[derive(Debug, Clone, Copy)]
pub struct Partition {
    pub start_lba: u64,
    pub size_lba: u64,
    pub is_bootable: bool,
}

pub struct PartitionTable {
    pub kind: PartitionType,
    pub partitions: [Option<Partition>; 4],
}

impl IngestionGate {
    pub fn new(bt: &BootServices) -> Self {
        let handle = bt.get_handle_for_protocol::<SimpleFileSystem>()
            .expect("failed to get FS handle");
        let fs = bt.open_protocol_exclusive::<SimpleFileSystem>(handle)
            .expect("failed to open FS protocol");
        
        Self { fs }
    }

    /// Parses the partition table of an image file
    pub fn parse_image(&self, buffer: &[u8]) -> Option<PartitionTable> {
        // Simple MBR check (Magic 0x55AA at end of first sector)
        if buffer.len() >= 512 && buffer[510] == 0x55 && buffer[511] == 0xAA {
            info!("󰒋 Detected MBR Partition Table");
            let mut partitions = [None; 4];
            
            // Simplified MBR parsing (extracting first partition for demo)
            partitions[0] = Some(Partition {
                start_lba: 2048, // Standard alignment
                size_lba: 1024 * 1024, // 512MB placeholder
                is_bootable: true,
            });

            return Some(PartitionTable { kind: PartitionType::Mbr, partitions });
        }

        // Simple GPT check (Signature 'EFI PART' at sector 1)
        if buffer.len() >= 1024 && &buffer[512..520] == b"EFI PART" {
            info!("󰒋 Detected GPT Partition Table");
            let mut partitions = [None; 4];
            partitions[0] = Some(Partition {
                start_lba: 4096,
                size_lba: 2048 * 1024, // 1GB placeholder
                is_bootable: true,
            });
            return Some(PartitionTable { kind: PartitionType::Gpt, partitions });
        }

        None
    }

use re_core::vdisk::TmxDiskHeader;

pub struct MountedAsset {
    pub start_lba: u64,
    pub size_lba: u64,
    pub data: &'static [u8],
}

impl IngestionGate {
    pub fn format_tmx_disk(size_gb: u32, root_cap: u64) -> TmxDiskHeader {
        info!("󰒋 Formatting new TMX-DISK: {}GB", size_gb);
        TmxDiskHeader {
            magic: TmxDiskHeader::MAGIC,
            version: 1,
            encryption_salt: [0; 16],
            root_capability: root_cap,
            extent_table_offset: TmxDiskHeader::HEADER_SIZE as u64,
            disk_size_gb: size_gb,
            extent_count: 0,
            integrity_hash: [0; 64],
        }
    }
    
    /// Mounts an image file by locating its bootable partition
    pub fn mount_image(&self, buffer: &'static [u8]) -> Option<MountedAsset> {
        let table = self.parse_image(buffer)?;
        
        // Locate the bootable partition
        for part in table.partitions.iter().flatten() {
            if part.is_bootable {
                info!("󰒋 Mounting bootable partition at LBA {}", part.start_lba);
                return Some(MountedAsset {
                    start_lba: part.start_lba,
                    size_lba: part.size_lba,
                    data: buffer,
                });
            }
        }
        None
    }
