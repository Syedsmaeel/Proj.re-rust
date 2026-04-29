//! SubKernelManager — master kernel's control plane for all sub-kernels

use super::instance::{SubKernel, SubKernelConfig, SubKernelId, SubKernelState};
use super::bridge::{Bridge, BridgeKind};
use super::snapshot::SubKernelSnapshot;
use crate::priv_model::{CapRight, CapabilityToken, PrivError, RingLevel};

pub struct SubKernelManager {
    kernels:  alloc::vec::Vec<SubKernel>,
    bridges:  alloc::vec::Vec<Bridge>,
    next_bridge_id: u64,
}

impl SubKernelManager {
    pub fn new() -> Self {
        Self {
            kernels: alloc::vec::Vec::new(),
            bridges: alloc::vec::Vec::new(),
            next_bridge_id: 1,
        }
    }

    /// Spawn a new sub-kernel — requires PROCESS_SPAWN capability
    pub fn spawn(
        &mut self,
        caller_cap: &CapabilityToken,
        config: SubKernelConfig,
        parent: Option<SubKernelId>,
    ) -> Result<SubKernelId, PrivError> {
        if !caller_cap.permits(CapRight::PROCESS_SPAWN) {
            return Err(PrivError::InsufficientRights);
        }
        let mut sk = SubKernel::spawn(config, parent);
        sk.boot();
        let id = sk.id;
        self.kernels.push(sk);
        Ok(id)
    }

    /// Pause a sub-kernel
    pub fn pause(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<(), &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_KILL) {
            return Err("insufficient capability to pause sub-kernel");
        }
        self.get_mut(id)?.pause()
    }

    /// Resume a paused sub-kernel
    pub fn resume(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<(), &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_SPAWN) {
            return Err("insufficient capability to resume sub-kernel");
        }
        self.get_mut(id)?.resume()
    }

    /// Hot-swap a running sub-kernel with a new config
    pub fn hot_swap(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
        new_config: SubKernelConfig,
    ) -> Result<(), &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_SPAWN) {
            return Err("insufficient capability to hot-swap sub-kernel");
        }
        self.get_mut(id)?.hot_swap(new_config)
    }

    /// Clone a sub-kernel — returns new ID
    pub fn clone_sk(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<SubKernelId, &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_SPAWN) {
            return Err("insufficient capability to clone sub-kernel");
        }
        let cloned = {
            let sk = self.get(id)?;
            sk.clone_sk(sk.parent)
        };
        let new_id = cloned.id;
        self.kernels.push(cloned);
        Ok(new_id)
    }

    /// Snapshot a sub-kernel — capture full state
    pub fn snapshot(
        &self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<SubKernelSnapshot, &'static str> {
        if !caller_cap.permits(CapRight::READ) {
            return Err("insufficient capability to snapshot sub-kernel");
        }
        let sk = self.get(id)?;
        Ok(SubKernelSnapshot::capture(sk))
    }

    /// Migrate a sub-kernel — begin migration
    pub fn migrate(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<(), &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_SPAWN) {
            return Err("insufficient capability to migrate sub-kernel");
        }
        self.get_mut(id)?.begin_migrate()
    }

    /// Complete migration after transport
    pub fn complete_migration(&mut self, id: SubKernelId) -> Result<(), &'static str> {
        self.get_mut(id)?.complete_migrate();
        Ok(())
    }

    /// Terminate and remove a sub-kernel
    pub fn terminate(
        &mut self,
        caller_cap: &CapabilityToken,
        id: SubKernelId,
    ) -> Result<(), &'static str> {
        if !caller_cap.permits(CapRight::PROCESS_KILL) {
            return Err("insufficient capability to terminate sub-kernel");
        }
        self.get_mut(id)?.terminate();
        self.kernels.retain(|sk| sk.id != id);
        Ok(())
    }

    /// Create an IPC bridge between two sub-kernels
    pub fn bridge_ipc(
        &mut self,
        caller_cap: &CapabilityToken,
        from: SubKernelId,
        to: SubKernelId,
    ) -> Result<u64, &'static str> {
        if !caller_cap.permits(CapRight::SEND) || !caller_cap.permits(CapRight::RECV) {
            return Err("insufficient capability to create IPC bridge");
        }
        let bridge_id = self.next_bridge_id;
        self.next_bridge_id += 1;
        self.bridges.push(Bridge::new(bridge_id, from, to, BridgeKind::Ipc));
        Ok(bridge_id)
    }

    /// Create a shared memory bridge between two sub-kernels
    pub fn bridge_shm(
        &mut self,
        caller_cap: &CapabilityToken,
        from: SubKernelId,
        to: SubKernelId,
        size: usize,
        read_only: bool,
    ) -> Result<u64, &'static str> {
        if !caller_cap.permits(CapRight::MAP) {
            return Err("insufficient capability to create shared memory bridge");
        }
        let bridge_id = self.next_bridge_id;
        self.next_bridge_id += 1;
        self.bridges.push(Bridge::new(
            bridge_id, from, to,
            BridgeKind::SharedMemory { size, read_only },
        ));
        Ok(bridge_id)
    }

    /// Tick all running sub-kernels
    pub fn tick_all(&mut self) {
        for sk in self.kernels.iter_mut() {
            if sk.state == SubKernelState::Running {
                sk.tick();
            }
        }
    }

    // ─── Query ───────────────────────────────────────────────────────────────

    pub fn get(&self, id: SubKernelId) -> Result<&SubKernel, &'static str> {
        self.kernels.iter().find(|sk| sk.id == id)
            .ok_or("sub-kernel not found")
    }

    pub fn get_mut(&mut self, id: SubKernelId) -> Result<&mut SubKernel, &'static str> {
        self.kernels.iter_mut().find(|sk| sk.id == id)
            .ok_or("sub-kernel not found")
    }

    pub fn count(&self) -> usize { self.kernels.len() }
    pub fn running(&self) -> usize {
        self.kernels.iter().filter(|sk| sk.state == SubKernelState::Running).count()
    }
    pub fn bridge_count(&self) -> usize { self.bridges.len() }

    pub fn list(&self) -> impl Iterator<Item = &SubKernel> {
        self.kernels.iter()
    }
}

impl Default for SubKernelManager {
    fn default() -> Self { Self::new() }
}

extern crate alloc;
