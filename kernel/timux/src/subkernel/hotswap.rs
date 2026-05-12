//! Hot-swap Sub-kernel — live replacement without reboot
  //!
  //! The HotSwapCoordinator allows the master kernel to replace a running
  //! sub-kernel with a new version at runtime while preserving:
  //!
  //!  • Open IPC channels (drained before cut-over)
  //!  • Capability tokens (transferred to the replacement)
  //!  • IKFS mount points (re-mounted on the new instance)
  //!  • Memory snapshot (copied to replacement's address space)
  //!
  //! # Swap lifecycle
  //!
  //! ```text
  //! RUNNING ──prepare──▶ DRAINING ──drain_complete──▶ FROZEN
  //!                                                      │
  //!                                              swap_in(new_sk)
  //!                                                      │
  //!                                                      ▼
  //!                                                   RUNNING  (new instance)
  //!                                  old instance ──▶ TERMINATED
  //! ```
  //!
  //! All steps require PROCESS_SPAWN capability.
  extern crate alloc;

  use alloc::vec::Vec;
  use alloc::string::String;
  use crate::priv_model::{CapRight, CapabilityToken, PrivError};
  use crate::subkernel::instance::{SubKernelId, SubKernelState};

  // ─── Error ───────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum HotSwapError {
      InsufficientRights,
      InvalidCapability,
      SwapAlreadyInProgress,
      InvalidState,
      DrainTimeout,
      SnapshotFailed,
      TransferFailed,
  }

  impl From<PrivError> for HotSwapError {
      fn from(e: PrivError) -> Self {
          match e {
              PrivError::InsufficientRights => HotSwapError::InsufficientRights,
              PrivError::InvalidCapability  => HotSwapError::InvalidCapability,
              _                             => HotSwapError::InsufficientRights,
          }
      }
  }

  // ─── Swap phase ──────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum SwapPhase {
      /// No swap in progress.
      Idle,
      /// Old sub-kernel is draining its IPC queues.
      Draining,
      /// Old sub-kernel is frozen; state being transferred.
      Frozen,
      /// New sub-kernel is live; old one pending cleanup.
      Complete,
  }

  // ─── Transferred state ───────────────────────────────────────────────────────

  /// Capabilities handed from old sub-kernel to new.
  #[derive(Debug, Clone)]
  pub struct TransferredCaps {
      pub tokens: Vec<CapabilityToken>,
  }

  /// Mount points re-attached to the new sub-kernel.
  #[derive(Debug, Clone)]
  pub struct TransferredMount {
      pub local:    String,
      pub vnode_id: u64,
      pub writable: bool,
  }

  /// Memory snapshot — a byte-level copy of the old sub-kernel's heap.
  #[derive(Debug, Clone)]
  pub struct MemorySnapshot {
      pub data: Vec<u8>,
      pub base_addr: usize,
  }

  // ─── SwapRecord ──────────────────────────────────────────────────────────────

  #[derive(Debug)]
  pub struct SwapRecord {
      pub old_id:        SubKernelId,
      pub new_id:        SubKernelId,
      pub phase:         SwapPhase,
      pub caps:          TransferredCaps,
      pub mounts:        Vec<TransferredMount>,
      pub snapshot:      Option<MemorySnapshot>,
      pub drain_timeout: u64,  // ticks before drain is forced
      pub drain_elapsed: u64,
  }

  // ─── HotSwapCoordinator ──────────────────────────────────────────────────────

  pub struct HotSwapCoordinator {
      active: Option<SwapRecord>,
      history: Vec<(SubKernelId, SubKernelId)>, // (old, new) completed swaps
  }

  impl HotSwapCoordinator {
      pub fn new() -> Self {
          Self { active: None, history: Vec::new() }
      }

      // ── Phase 1: prepare ─────────────────────────────────────────────────────

      /// Begin a hot-swap: freeze inbound traffic to `old_id` and start draining.
      ///
      /// Requires PROCESS_SPAWN capability.
      pub fn prepare(
          &mut self,
          cap:         &CapabilityToken,
          old_id:      SubKernelId,
          new_id:      SubKernelId,
          drain_ticks: u64,
      ) -> Result<(), HotSwapError> {
          if !cap.is_valid()                       { return Err(HotSwapError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(HotSwapError::InsufficientRights); }
          if self.active.is_some()                 { return Err(HotSwapError::SwapAlreadyInProgress); }

          self.active = Some(SwapRecord {
              old_id,
              new_id,
              phase:         SwapPhase::Draining,
              caps:          TransferredCaps { tokens: Vec::new() },
              mounts:        Vec::new(),
              snapshot:      None,
              drain_timeout: drain_ticks,
              drain_elapsed: 0,
          });
          Ok(())
      }

      // ── Phase 2: drain tick ───────────────────────────────────────────────────

      /// Advance the drain counter by `ticks`.  Returns true when drain is done.
      ///
      /// In a real system, the caller checks whether all IPC queues for old_id
      /// are empty; here we model it as a timeout that fires after drain_timeout
      /// ticks (forced drain).
      pub fn tick_drain(&mut self, ticks: u64, queues_empty: bool) -> Result<bool, HotSwapError> {
          let rec = self.active.as_mut().ok_or(HotSwapError::InvalidState)?;
          if rec.phase != SwapPhase::Draining { return Err(HotSwapError::InvalidState); }
          rec.drain_elapsed += ticks;
          let done = queues_empty || rec.drain_elapsed >= rec.drain_timeout;
          if done { rec.phase = SwapPhase::Frozen; }
          Ok(done)
      }

      // ── Phase 3: transfer state ───────────────────────────────────────────────

      /// Add a capability token to be transferred to the new sub-kernel.
      pub fn transfer_cap(&mut self, token: CapabilityToken) -> Result<(), HotSwapError> {
          let rec = self.active.as_mut().ok_or(HotSwapError::InvalidState)?;
          if rec.phase != SwapPhase::Frozen { return Err(HotSwapError::InvalidState); }
          rec.caps.tokens.push(token);
          Ok(())
      }

      /// Record an IKFS mount to be re-attached to the new sub-kernel.
      pub fn transfer_mount(
          &mut self,
          local: impl Into<String>,
          vnode_id: u64,
          writable: bool,
      ) -> Result<(), HotSwapError> {
          let rec = self.active.as_mut().ok_or(HotSwapError::InvalidState)?;
          if rec.phase != SwapPhase::Frozen { return Err(HotSwapError::InvalidState); }
          rec.mounts.push(TransferredMount { local: local.into(), vnode_id, writable });
          Ok(())
      }

      /// Attach a memory snapshot from the old sub-kernel's heap.
      pub fn attach_snapshot(&mut self, data: Vec<u8>, base_addr: usize) -> Result<(), HotSwapError> {
          let rec = self.active.as_mut().ok_or(HotSwapError::InvalidState)?;
          if rec.phase != SwapPhase::Frozen { return Err(HotSwapError::InvalidState); }
          rec.snapshot = Some(MemorySnapshot { data, base_addr });
          Ok(())
      }

      // ── Phase 4: complete ────────────────────────────────────────────────────

      /// Finalise the swap: mark the new sub-kernel as the replacement and
      /// return the completed record for the caller to apply (mount, cap-inject,
      /// snapshot-restore, schedule new sk).
      pub fn complete(
          &mut self,
          cap: &CapabilityToken,
      ) -> Result<SwapRecord, HotSwapError> {
          if !cap.is_valid()                       { return Err(HotSwapError::InvalidCapability); }
          if !cap.permits(CapRight::PROCESS_SPAWN) { return Err(HotSwapError::InsufficientRights); }

          let mut rec = self.active.take().ok_or(HotSwapError::InvalidState)?;
          if rec.phase != SwapPhase::Frozen { return Err(HotSwapError::InvalidState); }

          rec.phase = SwapPhase::Complete;
          self.history.push((rec.old_id, rec.new_id));
          Ok(rec)
      }

      // ── Query ────────────────────────────────────────────────────────────────

      pub fn phase(&self) -> SwapPhase {
          self.active.as_ref().map_or(SwapPhase::Idle, |r| r.phase)
      }

      pub fn in_progress(&self) -> bool { self.active.is_some() }

      /// Total completed swaps.
      pub fn swap_count(&self) -> usize { self.history.len() }

      /// Most recent (old, new) pair.
      pub fn last_swap(&self) -> Option<(SubKernelId, SubKernelId)> {
          self.history.last().copied()
      }
  }
  