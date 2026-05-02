//! Boot protocol — the contract between Ring -1 and Timux Ring 0
//!
//! Ring -1 fills a HandoffInfo struct and passes its address in a register
//! before jumping to Timux _start. Timux reads this to know:
//! - Where RAM is
//! - Where the framebuffer is
//! - What the kernel command line is
//! - The physical address of the Ring -1 page tables (to be reclaimed)

extern crate alloc;
use crate::mem::MemoryMap;

/// How we booted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootProtocol {
    Uefi,
    Bios,
}

/// Framebuffer info passed to Timux
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub base:   u64,
    pub width:  u32,
    pub height: u32,
    pub stride: u32,
    pub bpp:    u8,   // bits per pixel
}

/// The full handoff struct — Ring -1 → Ring 0
/// Passed as a pointer in RDI (x86_64 System V ABI first arg)
#[repr(C)]
pub struct HandoffInfo {
    /// Magic number — Timux verifies this
    pub magic:          u64,
    /// How we got here
    pub protocol:       BootProtocol,
    /// Kernel physical load address
    pub kernel_phys:    u64,
    /// Kernel virtual entry point
    pub kernel_entry:   u64,
    /// Heap start for Timux (largest usable region)
    pub heap_start:     u64,
    pub heap_size:      u64,
    /// Framebuffer
    pub fb:             FramebufferInfo,
    /// Command line (null-terminated, max 256 bytes)
    pub cmdline:        [u8; 256],
    /// Memory map entry count
    pub mmap_count:     u32,
    /// Memory map entries (inline, max 128 regions)
    pub mmap:           [MmapEntry; 128],
    /// Ring -1 reclaimable range
    pub loader_start:   u64,
    pub loader_size:    u64,
    /// RSDP physical address (ACPI)
    pub rsdp_addr:      u64,
}

/// Compact memory map entry for the handoff struct
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct MmapEntry {
    pub base: u64,
    pub size: u64,
    pub kind: u32,
    pub _pad: u32,
}

/// Magic value Timux checks to verify valid handoff
pub const HANDOFF_MAGIC: u64 = 0x54494D55585F424F; // "TIMUX_BO"

impl HandoffInfo {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed::<Self>() }
    }

    pub fn set_cmdline(&mut self, s: &[u8]) {
        let len = s.len().min(255);
        self.cmdline[..len].copy_from_slice(&s[..len]);
        self.cmdline[len] = 0;
    }

    pub fn cmdline_str(&self) -> &str {
        let end = self.cmdline.iter().position(|&b| b == 0).unwrap_or(255);
        core::str::from_utf8(&self.cmdline[..end]).unwrap_or("")
    }

    pub fn is_valid(&self) -> bool { self.magic == HANDOFF_MAGIC }
}

impl Default for HandoffInfo {
    fn default() -> Self { Self::new() }
}
