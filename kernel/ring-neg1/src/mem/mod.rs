//! Memory map — UEFI or BIOS E820
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryKind {
    Usable = 1, Reserved = 2, AcpiReclaimable = 3, AcpiNvs = 4,
    BadMemory = 5, BootloaderReclaimable = 6, KernelAndModules = 7, Framebuffer = 8,
}

impl MemoryKind {
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Usable | Self::BootloaderReclaimable)
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Usable=>"Usable", Self::Reserved=>"Reserved",
            Self::AcpiReclaimable=>"ACPI Reclaimable", Self::AcpiNvs=>"ACPI NVS",
            Self::BadMemory=>"Bad Memory", Self::BootloaderReclaimable=>"Bootloader (reclaimable)",
            Self::KernelAndModules=>"Kernel + Modules", Self::Framebuffer=>"Framebuffer",
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion { pub base: u64, pub size: u64, pub kind: MemoryKind }

impl MemoryRegion {
    pub fn end(&self) -> u64 { self.base + self.size }
    pub fn contains(&self, addr: u64) -> bool { addr >= self.base && addr < self.end() }
}

pub struct MemoryMap { regions: Vec<MemoryRegion> }

impl MemoryMap {
    pub fn new() -> Self { Self { regions: Vec::new() } }
    pub fn add(&mut self, r: MemoryRegion) { self.regions.push(r); }
    pub fn regions(&self) -> &[MemoryRegion] { &self.regions }
    pub fn count(&self) -> usize { self.regions.len() }
    pub fn total_usable(&self) -> u64 {
        self.regions.iter().filter(|r| r.kind.is_usable()).map(|r| r.size).sum()
    }
    pub fn largest_usable(&self) -> Option<&MemoryRegion> {
        self.regions.iter().filter(|r| r.kind == MemoryKind::Usable).max_by_key(|r| r.size)
    }
    pub fn find_usable(&self, min_addr: u64, size: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.kind.is_usable() && r.base >= min_addr && r.size >= size)
    }
    pub fn print_map(&self) -> String {
        let mut s = String::new();
        for r in &self.regions {
            s.push_str(&format!("  {:016x}-{:016x}  {:>8}KB  {}\n",
                r.base, r.end(), r.size/1024, r.kind.name()));
        }
        s
    }
}

impl Default for MemoryMap { fn default() -> Self { Self::new() } }
