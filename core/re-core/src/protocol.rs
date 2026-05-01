//! Timux Boot Protocol (TBP)
//!
//! Shared definitions for system handoff.

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub magic: u64,             // 0x54494d55582d3121 ('TIMUX-1!')
    pub version: u32,
    
    // Memory and Hardware
    pub heap_start: usize,
    pub heap_size: usize,
    pub framebuffer: FramebufferInfo,
    pub mmap_addr: usize,
    pub mmap_len: usize,
    
    // Sovereign Configurations (The injected configs)
    pub blueprint_addr: usize,  // Pointer to the serialized Sovereign Blueprint
    pub blueprint_len: usize,
    
    // Entropy for early security
    pub entropy_seed: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
}

impl BootInfo {
    pub const MAGIC: u64 = 0x54494d55582d3121;
}
