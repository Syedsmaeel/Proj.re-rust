//! Timux Memory Manager

pub mod allocator;
pub mod cortex;
pub mod cortex_map;

pub use allocator::{LinkedListAllocator, align_up, align_down};

use crate::priv_model::{CapRight, CapabilityToken, PrivError};

/// A virtual memory region
#[derive(Debug, Clone)]
pub struct VmRegion {
    pub vaddr: usize,
    pub size:  usize,
    pub flags: RegionFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct RegionFlags: u32 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXEC    = 1 << 2;
        const USER    = 1 << 3;
        const SHARED  = 1 << 4;
        const MMIO    = 1 << 5;
    }
}

/// Virtual address space — one per task / per sub-kernel
pub struct AddressSpace {
    regions:   alloc::vec::Vec<VmRegion>,
    arch_root: usize,
}

impl AddressSpace {
    pub fn new(arch_root: usize) -> Self {
        Self { regions: alloc::vec::Vec::new(), arch_root }
    }

    pub fn map(
        &mut self,
        cap: &CapabilityToken,
        vaddr: usize,
        size: usize,
        flags: RegionFlags,
    ) -> Result<(), PrivError> {
        if !cap.permits(CapRight::MAP) {
            return Err(PrivError::InsufficientRights);
        }
        self.regions.push(VmRegion { vaddr, size, flags });
        Ok(())
    }

    pub fn map_mmio(
        &mut self,
        cap: &CapabilityToken,
        _phys: usize,
        vaddr: usize,
        size: usize,
    ) -> Result<(), PrivError> {
        if !cap.permits(CapRight::MMIO_ACCESS) {
            return Err(PrivError::InsufficientRights);
        }
        self.regions.push(VmRegion {
            vaddr,
            size,
            flags: RegionFlags::READ | RegionFlags::WRITE | RegionFlags::MMIO,
        });
        Ok(())
    }

    pub fn arch_root(&self) -> usize { self.arch_root }
    pub fn regions(&self) -> &[VmRegion] { &self.regions }
}

extern crate alloc;
