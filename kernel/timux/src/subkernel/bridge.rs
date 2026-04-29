//! Bridge — communication channel between two sub-kernels
//! Either IPC (message passing) or shared memory

use super::instance::SubKernelId;

#[derive(Debug, Clone)]
pub enum BridgeKind {
    /// Capability-gated message passing
    Ipc,
    /// Shared physical memory region
    SharedMemory { size: usize, read_only: bool },
    /// Both IPC + shared memory
    Hybrid { shm_size: usize },
}

#[derive(Debug)]
pub struct Bridge {
    pub id:       u64,
    pub from:     SubKernelId,
    pub to:       SubKernelId,
    pub kind:     BridgeKind,
    pub active:   bool,
    pub messages: u64,  // total messages sent through this bridge
    pub bytes:    u64,  // total bytes through shared memory
}

impl Bridge {
    pub fn new(id: u64, from: SubKernelId, to: SubKernelId, kind: BridgeKind) -> Self {
        Self { id, from, to, kind, active: true, messages: 0, bytes: 0 }
    }

    pub fn record_message(&mut self, size: usize) {
        self.messages += 1;
        self.bytes += size as u64;
    }

    pub fn deactivate(&mut self) { self.active = false; }
}
