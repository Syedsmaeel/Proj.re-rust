//! Sovereign Shadow Manager
//!
//! Manages full memory state mirroring and automatic fail-over for sub-kernels.
//! Each primary sub-kernel has a shadow replica kept in sync.
//! On health failure, shadow is promoted to primary instantly.

use crate::subkernel::instance::{SubKernel, SubKernelConfig, SubKernelId, MemoryRange};


/// A paired primary + shadow sub-kernel instance
pub struct ShadowInstance {
    pub id:         SubKernelId,
    pub primary:    Box<SubKernel>,
    pub shadow:     Box<SubKernel>,
    pub is_healthy: bool,
    pub sync_tick:  u64,
}

impl ShadowInstance {
    pub fn new(primary: SubKernel, shadow: SubKernel) -> Self {
        let id = primary.id;
        Self {
            id,
            primary: Box::new(primary),
            shadow:  Box::new(shadow),
            is_healthy: true,
            sync_tick: 0,
        }
    }

    /// Mirror full memory state from primary to shadow
    pub fn sync(&mut self) {
        let primary_range = self.primary.memory_range();
        let shadow_range  = self.shadow.memory_range_mut();

        // Only copy if both ranges are valid and same size
        if primary_range.size == 0 || primary_range.size != shadow_range.size {
            return;
        }

        // SAFETY: Both ranges point to valid, non-overlapping physical memory
        // allocated at sub-kernel spawn time. Size equality is verified above.
        unsafe {
            core::ptr::copy_nonoverlapping(
                primary_range.start as *const u8,
                shadow_range.start  as *mut u8,
                primary_range.size,
            );
        }

        self.sync_tick += 1;
    }

    /// Check if primary is alive; if not, promote shadow
    pub fn check_and_failover(&mut self) -> bool {
        if !self.is_healthy {
            // Promote shadow to primary
            core::mem::swap(&mut self.primary, &mut self.shadow);
            self.primary.state = crate::subkernel::instance::SubKernelState::Running;
            self.is_healthy = true;
            true // failover occurred
        } else {
            false
        }
    }

    /// Mark primary as unhealthy — triggers failover on next check
    pub fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
    }
}

/// Manages all shadow pairs in the system
pub struct ShadowManager {
    instances: Vec<ShadowInstance>,
}

impl ShadowManager {
    pub fn new() -> Self {
        Self { instances: Vec::new() }
    }

    /// Register a primary+shadow pair
    pub fn register(&mut self, primary: SubKernel, shadow: SubKernel) -> SubKernelId {
        let id = primary.id;
        self.instances.push(ShadowInstance::new(primary, shadow));
        id
    }

    /// Sync all shadow instances (call on every kernel tick)
    pub fn sync_all(&mut self) {
        for inst in &mut self.instances {
            if inst.is_healthy {
                inst.sync();
            }
        }
    }

    /// Health check + automatic failover for all instances
    /// Returns IDs of sub-kernels that failed over
    pub fn check_health(&mut self) -> Vec<SubKernelId> {
        let mut failed_over = Vec::new();
        for inst in &mut self.instances {
            if inst.check_and_failover() {
                failed_over.push(inst.id);
            }
        }
        failed_over
    }

    /// Mark a sub-kernel as unhealthy
    pub fn mark_unhealthy(&mut self, id: SubKernelId) {
        if let Some(inst) = self.instances.iter_mut().find(|i| i.id == id) {
            inst.mark_unhealthy();
        }
    }

    /// Get reference to primary sub-kernel
    pub fn get_primary(&self, id: SubKernelId) -> Option<&SubKernel> {
        self.instances.iter().find(|i| i.id == id).map(|i| i.primary.as_ref())
    }

    pub fn count(&self) -> usize { self.instances.len() }
    pub fn healthy_count(&self) -> usize { self.instances.iter().filter(|i| i.is_healthy).count() }
}

impl Default for ShadowManager {
    fn default() -> Self { Self::new() }
}

extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;
