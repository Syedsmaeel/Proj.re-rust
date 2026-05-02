//! Cortex Memory Mapper
//!
//! Maps semantic objects to physical backing store (RAM or P2P-remote).

use crate::mm::cortex::MemoryContext;
use alloc::collections::BTreeMap;
use spin::Mutex;

pub type ObjectHandle = u64;

pub struct CortexMap {
    pub registry: Mutex<BTreeMap<ObjectHandle, (MemoryContext, u64)>>, // Handle -> (Context, PhysAddr)
}

impl CortexMap {
    pub fn new() -> Self {
        Self { registry: Mutex::new(BTreeMap::new()) }
    }

    pub fn map(&self, handle: ObjectHandle, ctx: MemoryContext, addr: u64) {
        let mut reg = self.registry.lock();
        reg.insert(handle, (ctx, addr));
    }

    pub fn resolve(&self, handle: ObjectHandle) -> Option<(MemoryContext, u64)> {
        self.registry.lock().get(&handle).cloned()
    }
}
