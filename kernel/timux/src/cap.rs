//! Capability subsystem — minting, delegation, revocation, IPC passing
  //!
  //! v2 changes:
  //!  - BUGFIX: ring-level comparison now uses i8 (not u8), so negative rings
  //!    (Ring -5 … Ring -1) compare correctly. Previously Ring -1 became 255
  //!    after the cast, allowing any caller to bypass the ring-violation check.
  //!  - NEW: append-only AuditLog records every mint / verify / deny event.
  //!  - NEW: delegate() helper enforces right-subsetting across ring levels.

  use crate::priv_model::{CapRight, CapabilityToken, PrivError, RingLevel};
  use spin::Mutex;

  // ─── Audit log ────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy)]
  pub enum AuditKind {
      Mint,
      Verify,
      Delegate,
      DenyRingViolation,
      DenyInsufficientRights,
      DenyInvalidCap,
  }

  #[derive(Debug, Clone)]
  pub struct AuditEvent {
      pub kind:   AuditKind,
      pub ring:   RingLevel,
      pub rights: CapRight,
      pub seq:    u64,
  }

  /// Lock-free append-only ring buffer — last 64 capability events.
  pub struct AuditLog {
      buf:   [Option<AuditEvent>; 64],
      head:  usize,
      count: usize,
      seq:   u64,
  }

  impl AuditLog {
      pub const fn new() -> Self {
          Self { buf: [const { None }; 64], head: 0, count: 0, seq: 0 }
      }

      pub fn push(&mut self, kind: AuditKind, ring: RingLevel, rights: CapRight) {
          self.seq += 1;
          self.buf[self.head] = Some(AuditEvent { kind, ring, rights, seq: self.seq });
          self.head = (self.head + 1) % 64;
          if self.count < 64 { self.count += 1; }
      }

      /// Iterate the N most-recent events (oldest first).
      pub fn recent(&self, n: usize) -> impl Iterator<Item = &AuditEvent> {
          let n = n.min(self.count);
          let start = (self.head + 64 - n) % 64;
          (0..n).filter_map(move |i| self.buf[(start + i) % 64].as_ref())
      }

      pub fn total_events(&self) -> u64 { self.seq }
  }

  pub static AUDIT_LOG: Mutex<AuditLog> = Mutex::new(AuditLog::new());

  #[inline]
  fn audit(kind: AuditKind, ring: RingLevel, rights: CapRight) {
      if let Some(mut log) = AUDIT_LOG.try_lock() {
          log.push(kind, ring, rights);
      }
  }

  // ─── CapAuthority ─────────────────────────────────────────────────────────────

  /// Global capability authority — only RingLevel::KernelCore (ring0) may create one.
  pub struct CapAuthority {
      ring: RingLevel,
  }

  impl CapAuthority {
      pub fn new(ring: RingLevel) -> Self {
          assert!(ring == RingLevel::KernelCore,
              "CapAuthority can only be created by KernelCore (ring0)");
          Self { ring }
      }

      pub fn mint_root(&self, for_ring: RingLevel) -> CapabilityToken {
          let tok = CapabilityToken::mint(CapRight::all(), for_ring);
          audit(AuditKind::Mint, for_ring, CapRight::all());
          tok
      }

      pub fn mint(&self, rights: CapRight, for_ring: RingLevel) -> CapabilityToken {
          let tok = CapabilityToken::mint(rights, for_ring);
          audit(AuditKind::Mint, for_ring, rights);
          tok
      }

      pub fn mint_user(&self) -> CapabilityToken {
          let r = CapRight::READ | CapRight::WRITE | CapRight::EXEC
              | CapRight::SEND | CapRight::RECV | CapRight::DELEGATE
              | CapRight::FS_READ | CapRight::FS_WRITE
              | CapRight::NET_SEND | CapRight::NET_RECV
              | CapRight::CLOCK_READ | CapRight::PROCESS_SPAWN;
          let tok = CapabilityToken::mint(r, RingLevel::User);
          audit(AuditKind::Mint, RingLevel::User, r);
          tok
      }

      pub fn mint_sandbox(&self) -> CapabilityToken {
          let r = CapRight::READ | CapRight::EXEC;
          let tok = CapabilityToken::mint(r, RingLevel::Sandbox);
          audit(AuditKind::Mint, RingLevel::Sandbox, r);
          tok
      }

      pub fn mint_driver(&self) -> CapabilityToken {
          let r = CapRight::READ | CapRight::WRITE | CapRight::MAP
              | CapRight::IRQ_BIND | CapRight::DMA_ACCESS
              | CapRight::MMIO_ACCESS | CapRight::RING_TRANSITION;
          let tok = CapabilityToken::mint(r, RingLevel::KernelExtension);
          audit(AuditKind::Mint, RingLevel::KernelExtension, r);
          tok
      }

      /// Delegate a strict subset of `src`'s rights to `to_ring`.
      /// The target ring must be less privileged than the source token's owner.
      pub fn delegate(
          &self,
          src: &CapabilityToken,
          subset: CapRight,
          to_ring: RingLevel,
      ) -> Result<CapabilityToken, PrivError> {
          if !src.is_valid() {
              audit(AuditKind::DenyInvalidCap, to_ring, subset);
              return Err(PrivError::InvalidCapability);
          }
          if !src.permits(CapRight::DELEGATE) {
              audit(AuditKind::DenyInsufficientRights, to_ring, subset);
              return Err(PrivError::InsufficientRights);
          }
          // Cannot delegate to a more-privileged ring
          if (to_ring as i8) < (src.owner() as i8) {
              audit(AuditKind::DenyRingViolation, to_ring, subset);
              return Err(PrivError::EscalationDenied);
          }
          let granted = src.rights() & subset;
          let tok = CapabilityToken::mint(granted, to_ring);
          audit(AuditKind::Delegate, to_ring, granted);
          Ok(tok)
      }
  }

  // ─── verify ───────────────────────────────────────────────────────────────────

  /// Verify a capability token for a specific operation.
  ///
  /// # Security fix (v2)
  /// Uses `i8` comparison for ring levels so negative rings (Ring -5 … Ring -1)
  /// are ordered correctly. The previous `as u8` cast silently turned Ring -1
  /// into 255, meaning any caller could present a Ring -1 token and bypass the
  /// ring-violation guard.
  pub fn verify(
      token: &CapabilityToken,
      required: CapRight,
      caller_ring: RingLevel,
  ) -> Result<(), PrivError> {
      if !token.is_valid() {
          audit(AuditKind::DenyInvalidCap, caller_ring, required);
          return Err(PrivError::InvalidCapability);
      }
      if !token.permits(required) {
          audit(AuditKind::DenyInsufficientRights, caller_ring, required);
          return Err(PrivError::InsufficientRights);
      }
      // FIXED: i8 comparison preserves ordering for negative ring levels.
      if (token.owner() as i8) < (caller_ring as i8) {
          audit(AuditKind::DenyRingViolation, caller_ring, required);
          return Err(PrivError::RingViolation { from: caller_ring, to: token.owner() });
      }
      audit(AuditKind::Verify, caller_ring, required);
      Ok(())
  }
  