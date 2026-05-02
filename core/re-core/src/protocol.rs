//! Timux Boot Protocol (TBP)
//!
//! Shared definitions for system handoff.

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct StaticStr {
    ptr: *const u8,
    len: usize,
}

impl StaticStr {
    pub const fn new(s: &'static str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr, self.len);
            core::str::from_utf8_unchecked(slice)
        }
    }
}

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
    
pub fn minimal(heap_start: usize) -> Self {
        Self {
            magic: Self::MAGIC,
            version: 1,
            heap_start,
            heap_size: 0x40_0000,
            framebuffer: FramebufferInfo { addr: 0, size: 0, width: 0, height: 0, pitch: 0 },
            mmap_addr: 0,
            mmap_len: 0,
            blueprint_addr: 0,
            blueprint_len: 0,
            entropy_seed: [0; 32],
        }
    }
}
