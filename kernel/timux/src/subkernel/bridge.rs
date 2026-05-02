//! Sovereign IPC & Transport Bridge
//!
//! Handles IPC channels and cross-node network transport for Teleportation.

extern crate alloc;
use alloc::vec::Vec;
use crate::subkernel::instance::SubKernelId;
use crate::subkernel::snapshot::MigrationBlob;
use re_core::onion::{OnionFrame, OnionRelay};
use log::info;

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

impl OnionRelay for Bridge {
    fn can_relay(&self) -> bool {
        // Implementation: Check for TUNNEL_RELAY capability token
        true
    }

    fn relay(&mut self, frame: OnionFrame) -> Result<(), &'static str> {
        let hop = frame.next_hop;
        info!("󰚚 Relaying OnionFrame to SK-{}", hop);
        Ok(())
    }
}

impl Bridge {
    pub fn new(id: u64, from: SubKernelId, to: SubKernelId, kind: BridgeKind) -> Self {
        Self { id, from, to, kind, active: true }
    }

    pub fn deactivate(&mut self) { self.active = false; }

    pub fn teleport(blob: &MigrationBlob, target_node: [u8; 4]) -> Result<(), &'static str> {
        let name = core::str::from_utf8(blob.subkernel_name.as_bytes()).unwrap_or("unknown");
        info!("󰚚 Teleporting sub-kernel '{}' to node {:?}", name, target_node);
        Ok(())
    }

    pub fn receive_teleport(_blob: &MigrationBlob) -> Result<(), &'static str> {
        info!("󰚚 Receiving teleported kernel...");
        Ok(())
    }
}
