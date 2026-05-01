//! Sovereign vDisk (.tmx-disk) Header Definition
//!
//! A self-describing, capability-aware storage container optimized 
//! for the Timux Sovereign Fractal system.

#[repr(C, packed)]
pub struct TmxDiskHeader {
    pub magic: [u8; 8],           // "TMX-DISK"
    pub version: u32,
    pub encryption_salt: [u8; 16], // Derived from Sovereign Origin
    pub root_capability: u64,      // Cap required for mount
    pub extent_table_offset: u64,  // Start of data extents
    pub disk_size_gb: u32,         // Total virtual capacity (e.g., 200GB)
    pub extent_count: u32,         // Active data chunks
    pub integrity_hash: [u8; 64],  // Ed25519 signature
}

impl TmxDiskHeader {
    pub const MAGIC: [u8; 8] = *b"TMX-DISK";
    pub const HEADER_SIZE: usize = 128; // Header footprint
    
    // Size constraints:
    // Virtual capacity: 200 GB
    // Actual payload: < 500 MB (via sparse extent-based allocation)
    pub const MAX_PAYLOAD_SIZE: usize = 500 * 1024 * 1024;
}
