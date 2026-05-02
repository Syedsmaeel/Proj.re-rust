//! Sovereign IPC & Transport Bridge
//!
//! Handles IPC channels and cross-node network transport for Teleportation.
extern crate alloc;
use alloc::vec::Vec;
use crate::subkernel::instance::SubKernelId;

pub struct Bridge {
    pub id:   u64,
    pub from: SubKernelId,
    pub to:   SubKernelId,
    pub kind: BridgeKind,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum BridgeKind {
    Ipc,
    SharedMemory { size: usize, read_only: bool },
    Teleport,
}

impl Bridge {
    pub fn new(id: u64, from: SubKernelId, to: SubKernelId, kind: BridgeKind) -> Self {
        Self { id, from, to, kind, active: true }
    }
    pub fn deactivate(&mut self) { self.active = false; }
}
