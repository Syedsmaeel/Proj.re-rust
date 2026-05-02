//! Sovereign Ingestion Gate for TBM
//!
//! Handles discovery and "mounting" of .iso and .img assets from the 
//! UEFI file system for sub-kernel deployment.

use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::ScopedProtocol;
use log::info;
use re_core::vdisk::TmxDiskHeader;

pub struct MountedAsset {
    pub start_lba: u64,
    pub size_lba: u64,
    pub data: &'static [u8],
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

pub struct IngestionGate<'a> {
    _fs: ScopedProtocol<'a, SimpleFileSystem>,
}

impl<'a> IngestionGate<'a> {
    pub fn new(bt: &'a BootServices) -> Self {
        let handle = bt.get_handle_for_protocol::<SimpleFileSystem>()
            .expect("failed to get FS handle");
        let fs = bt.open_protocol_exclusive::<SimpleFileSystem>(handle)
            .expect("failed to open FS protocol");
        
        Self { _fs: fs }
    }

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

    pub fn parse_image(&self, buffer: &[u8]) -> Option<PartitionTable> {
        if buffer.len() >= 512 && buffer[510] == 0x55 && buffer[511] == 0xAA {
            let mut partitions = [None; 4];
            partitions[0] = Some(Partition { start_lba: 2048, size_lba: 1024 * 1024, is_bootable: true });
            return Some(PartitionTable { kind: PartitionType::Mbr, partitions });
        }
        if buffer.len() >= 1024 && &buffer[512..520] == b"EFI PART" {
            let mut partitions = [None; 4];
            partitions[0] = Some(Partition { start_lba: 4096, size_lba: 2048 * 1024, is_bootable: true });
            return Some(PartitionTable { kind: PartitionType::Gpt, partitions });
        }
        None
    }

    pub fn mount_image(&self, buffer: &'static [u8]) -> Option<MountedAsset> {
        let table = self.parse_image(buffer)?;
        for part in table.partitions.iter().flatten() {
            if part.is_bootable {
                return Some(MountedAsset { start_lba: part.start_lba, size_lba: part.size_lba, data: buffer });
            }
        }
        None
    }
}
