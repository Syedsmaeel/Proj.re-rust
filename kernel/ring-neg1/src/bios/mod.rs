//! BIOS boot path — stage2 in protected/long mode
//! Stage1 (MBR, 512 bytes) is assembly — loads stage2 from disk
//! Stage2 (this code) runs in long mode and does the same job as UEFI path

extern crate alloc;

/// BIOS E820 memory map entry (from INT 0x15, EAX=0xE820)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct E820Entry {
    pub base: u64,
    pub size: u64,
    pub kind: u32,
    pub acpi: u32,
}

impl E820Entry {
    pub fn to_region(&self) -> crate::mem::MemoryRegion {
        use crate::mem::{MemoryRegion, MemoryKind};
        let kind = match self.kind {
            1 => MemoryKind::Usable,
            2 => MemoryKind::Reserved,
            3 => MemoryKind::AcpiReclaimable,
            4 => MemoryKind::AcpiNvs,
            5 => MemoryKind::BadMemory,
            _ => MemoryKind::Reserved,
        };
        MemoryRegion { base: self.base, size: self.size, kind }
    }
}

/// BIOS boot flow
pub struct BiosBootFlow;

impl BiosBootFlow {
    /// Parse E820 memory map entries (passed from stage1 asm via a fixed address)
    pub fn parse_e820(entries: &[E820Entry]) -> crate::mem::MemoryMap {
        let mut map = crate::mem::MemoryMap::new();
        for e in entries {
            if e.size > 0 {
                map.add(e.to_region());
            }
        }
        map
    }

    /// VESA framebuffer info from BIOS (mode 0x118 = 1024x768x24 or similar)
    pub fn vesa_framebuffer(base: u64, w: u32, h: u32) -> crate::proto::FramebufferInfo {
        crate::proto::FramebufferInfo {
            base, width: w, height: h, stride: w, bpp: 32,
        }
    }

    /// Build handoff info from BIOS boot data
    pub fn build_handoff(
        map: &crate::mem::MemoryMap,
        kernel_entry: u64,
        fb: crate::proto::FramebufferInfo,
    ) -> crate::proto::HandoffInfo {
        use crate::proto::{HandoffInfo, MmapEntry, HANDOFF_MAGIC};

        let mut info = HandoffInfo::new();
        info.magic        = HANDOFF_MAGIC;
        info.protocol     = crate::proto::BootProtocol::Bios;
        info.kernel_entry = kernel_entry;
        info.kernel_phys  = 0x0010_0000;
        info.fb           = fb;

        if let Some(heap) = map.find_usable(0x0050_0000, 4 * 1024 * 1024) {
            info.heap_start = heap.base;
            info.heap_size  = heap.size.min(32 * 1024 * 1024);
        }

        info.mmap_count = map.count() as u32;
        for (i, r) in map.regions().iter().enumerate().take(128) {
            info.mmap[i] = MmapEntry {
                base: r.base, size: r.size, kind: r.kind as u32, _pad: 0,
            };
        }

        info.loader_start = 0x0007_C000; // MBR + stage2
        info.loader_size  = 0x0000_8000;
        info.set_cmdline(b"ring0 loglevel=3");
        info
    }
}
