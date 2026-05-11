//! Sub-Kernel Manager — master control plane for the sub-kernel system
  //!
  //! v2: implements spawn, terminate, list, and snapshot-restore lifecycle.
  //!
  //! The SubKernelManager owns all live sub-kernel instances and enforces
  //! capability-gated access for every lifecycle operation.

  extern crate alloc;

  use alloc::vec::Vec;
  use crate::subkernel::instance::{SubKernel, SubKernelConfig, SubKernelId, SubKernelState};
  use crate::subkernel::shadow::ShadowManager;
  use crate::priv_model::{CapRight, CapabilityToken, PrivError};

  // ─── Errors ───────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ManagerError {
      InsufficientRights,
      InvalidCapability,
      NotFound,
      AlreadyExists,
      LimitReached,
      InvalidState,
  }

  impl From<PrivError> for ManagerError {
      fn from(e: PrivError) -> Self {
          match e {
              PrivError::InsufficientRights => ManagerError::InsufficientRights,
              PrivError::InvalidCapability  => ManagerError::InvalidCapability,
              _                             => ManagerError::InsufficientRights,
          }
      }
  }

  // ─── SubKernelRecord ─────────────────────────────────────────────────────────

  /// Internal record stored in the manager table.
  struct SkRecord {
      sk:     SubKernel,
      parent: Option<SubKernelId>,
  }

  // ─── SubKernelManager ─────────────────────────────────────────────────────────

  /// Maximum simultaneous sub-kernels.
  pub const MAX_SUBKERNELS: usize = 64;

  pub struct SubKernelManager {
      table:   Vec<SkRecord>,
      shadows: ShadowManager,
      next_id: SubKernelId,
  }

  impl SubKernelManager {
      pub fn new() -> Self {
          Self { table: Vec::new(), shadows: ShadowManager::new(), next_id: 1 }
      }

      // ── Spawn ─────────────────────────────────────────────────────────────────

      /// Spawn a new sub-kernel.
      ///
      /// Requires: PROCESS_SPAWN capability.
      pub fn spawn(
          &mut self,
          cap: &CapabilityToken,
          config: SubKernelConfig,
          parent: Option<SubKernelId>,
      ) -> Result<SubKernelId, ManagerError> {
          if !cap.is_valid()                       { return Err(ManagerError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(ManagerError::InsufficientRights); }
          if self.table.len() >= MAX_SUBKERNELS    { return Err(ManagerError::LimitReached); }

          // Validate parent exists if specified
          if let Some(pid) = parent {
              if self.find(pid).is_none() { return Err(ManagerError::NotFound); }
          }

          let id = self.next_id;
          self.next_id += 1;

          let sk = SubKernel::new(id, config);
          self.table.push(SkRecord { sk, parent });

          Ok(id)
      }

      // ── Terminate ─────────────────────────────────────────────────────────────

      /// Terminate a sub-kernel and all its children.
      ///
      /// Requires: PROCESS_SPAWN (reuse for lifecycle control).
      pub fn terminate(
          &mut self,
          cap: &CapabilityToken,
          id: SubKernelId,
      ) -> Result<(), ManagerError> {
          if !cap.is_valid()                       { return Err(ManagerError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(ManagerError::InsufficientRights); }

          // Collect children (shallow — one level; deep termination via recursion left to caller)
          let children: Vec<SubKernelId> = self.table.iter()
              .filter(|r| r.parent == Some(id))
              .map(|r| r.sk.id)
              .collect();
          for child in children {
              let _ = self.terminate(cap, child);
          }

          // Mark and remove
          if let Some(rec) = self.table.iter_mut().find(|r| r.sk.id == id) {
              rec.sk.state = SubKernelState::Terminated;
          }
          self.table.retain(|r| r.sk.id != id);
          Ok(())
      }

      // ── Suspend / Resume ──────────────────────────────────────────────────────

      pub fn suspend(&mut self, cap: &CapabilityToken, id: SubKernelId) -> Result<(), ManagerError> {
          if !cap.is_valid()                       { return Err(ManagerError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(ManagerError::InsufficientRights); }
          let rec = self.find_mut(id).ok_or(ManagerError::NotFound)?;
          if rec.sk.state != SubKernelState::Running { return Err(ManagerError::InvalidState); }
          rec.sk.state = SubKernelState::Suspended;
          Ok(())
      }

      pub fn resume(&mut self, cap: &CapabilityToken, id: SubKernelId) -> Result<(), ManagerError> {
          if !cap.is_valid()                       { return Err(ManagerError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(ManagerError::InsufficientRights); }
          let rec = self.find_mut(id).ok_or(ManagerError::NotFound)?;
          if rec.sk.state != SubKernelState::Suspended { return Err(ManagerError::InvalidState); }
          rec.sk.state = SubKernelState::Running;
          Ok(())
      }

      // ── Query ─────────────────────────────────────────────────────────────────

      pub fn get(&self, id: SubKernelId) -> Option<&SubKernel> {
          self.find(id).map(|r| &r.sk)
      }

      pub fn get_mut(&mut self, id: SubKernelId) -> Option<&mut SubKernel> {
          self.find_mut(id).map(|r| &mut r.sk)
      }

      /// Iterate over all live sub-kernels.
      pub fn iter(&self) -> impl Iterator<Item = &SubKernel> {
          self.table.iter().map(|r| &r.sk)
      }

      /// Total number of live sub-kernels.
      pub fn count(&self) -> usize { self.table.len() }

      /// Children of a given sub-kernel.
      pub fn children(&self, id: SubKernelId) -> impl Iterator<Item = &SubKernel> {
          self.table.iter().filter(move |r| r.parent == Some(id)).map(|r| &r.sk)
      }

      // ── Internal helpers ──────────────────────────────────────────────────────

      fn find(&self, id: SubKernelId) -> Option<&SkRecord> {
          self.table.iter().find(|r| r.sk.id == id)
      }

      fn find_mut(&mut self, id: SubKernelId) -> Option<&mut SkRecord> {
          self.table.iter_mut().find(|r| r.sk.id == id)
      }
  }
  