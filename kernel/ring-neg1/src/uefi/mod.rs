//! UEFI boot path — EFI application entry
//! Handles: GOP framebuffer, UEFI memory map, ExitBootServices, kernel load
extern crate alloc;

/// UEFI status codes (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum UefiStatus {
    Success          = 0,
    LoadError        = 1,
    InvalidParameter = 2,
    Unsupported      = 3,
    BadBufferSize    = 4,
    BufferTooSmall   = 5,
    NotReady         = 6,
    DeviceError      = 7,
    WriteProtected   = 8,
    OutOfResources   = 9,
    NotFound         = 14,
}

/// UEFI memory type mapping → our MemoryKind
pub fn uefi_mem_type_to_kind(uefi_type: u32) -> crate::mem::MemoryKind {
    use crate::mem::MemoryKind::*;
    match uefi_type {
        0  => Reserved,          // EfiReservedMemoryType
        1  => BootloaderReclaimable, // EfiLoaderCode
        2  => BootloaderReclaimable, // EfiLoaderData
        3  => BootloaderReclaimable, // EfiBootServicesCode
        4  => BootloaderReclaimable, // EfiBootServicesData
        5  => Reserved,          // EfiRuntimeServicesCode
        6  => Reserved,          // EfiRuntimeServicesData
        7  => Usable,            // EfiConventionalMemory
        8  => BadMemory,         // EfiUnusableMemory
        9  => AcpiReclaimable,
        10 => AcpiNvs,
        11 => Reserved,          // EfiMemoryMappedIO
        12 => Reserved,          // EfiMemoryMappedIOPortSpace
        13 => Reserved,          // EfiPalCode
        _  => Reserved,
    }
}

/// UEFI boot flow — called from BOOTX64.efi _start
pub struct UefiBootFlow;

impl UefiBootFlow {
    /// Step 1: Initialize GOP framebuffer
    pub fn init_framebuffer() -> Option<crate::display::Framebuffer> {
        // In real impl: locate GOP protocol, set mode, get framebuffer base
        // Here: return None (bare-metal only, can't run in std environment)
        None
    }

    /// Step 2: Get UEFI memory map
    pub fn get_memory_map() -> crate::mem::MemoryMap {
        // In real impl: call GetMemoryMap(), parse UEFI descriptors
        // Simulate a realistic map for testing
        let mut map = crate::mem::MemoryMap::new();
        use crate::mem::{MemoryRegion, MemoryKind};
        map.add(MemoryRegion { base: 0x0000_0000, size: 0x0009_F000, kind: MemoryKind::Usable });
        map.add(MemoryRegion { base: 0x0009_F000, size: 0x0000_1000, kind: MemoryKind::Reserved });
        map.add(MemoryRegion { base: 0x000E_0000, size: 0x0002_0000, kind: MemoryKind::Reserved });
        map.add(MemoryRegion { base: 0x0010_0000, size: 0x0040_0000, kind: MemoryKind::BootloaderReclaimable }); // Ring -1
        map.add(MemoryRegion { base: 0x0050_0000, size: 0x0400_0000, kind: MemoryKind::KernelAndModules });      // Timux
        map.add(MemoryRegion { base: 0x0450_0000, size: 0x7B80_0000, kind: MemoryKind::Usable });               // Free RAM ~1.97GB
        map.add(MemoryRegion { base: 0xFD00_0000, size: 0x0100_0000, kind: MemoryKind::Framebuffer });
        map.add(MemoryRegion { base: 0xFE00_0000, size: 0x0200_0000, kind: MemoryKind::Reserved });
        map
    }

    /// Step 3: Build HandoffInfo and jump to Timux
    pub fn build_handoff(
        map: &crate::mem::MemoryMap,
        kernel_entry: u64,
    ) -> crate::proto::HandoffInfo {
        use crate::proto::{HandoffInfo, FramebufferInfo, MmapEntry, HANDOFF_MAGIC};
        use crate::mem::MemoryKind;

        let mut info = HandoffInfo::new();
        info.magic       = HANDOFF_MAGIC;
        info.protocol    = crate::proto::BootProtocol::Uefi;
        info.kernel_entry = kernel_entry;
        info.kernel_phys  = 0x0050_0000;

        // Find heap
        if let Some(heap) = map.find_usable(0x0450_0000, 4 * 1024 * 1024) {
            info.heap_start = heap.base;
            info.heap_size  = heap.size.min(64 * 1024 * 1024); // cap at 64MB for kernel heap
        }

        // Framebuffer (1920x1080x32)
        info.fb = FramebufferInfo {
            base: 0xFD00_0000,
            width: 1920, height: 1080, stride: 1920, bpp: 32,
        };

        // Memory map
        info.mmap_count = map.count() as u32;
        for (i, r) in map.regions().iter().enumerate().take(128) {
            info.mmap[i] = MmapEntry {
                base: r.base, size: r.size, kind: r.kind as u32, _pad: 0,
            };
        }

        // Ring -1 reclaimable
        info.loader_start = 0x0010_0000;
        info.loader_size  = 0x0040_0000;

        info.set_cmdline(b"ring0 loglevel=3");
        info
    }
}
