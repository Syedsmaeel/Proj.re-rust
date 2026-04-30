//! Timux Privilege Model — Hybrid Ring + Capability System
//!
//! Unlike Intel's fixed Ring 0-3, Timux defines its own ring hierarchy:
//!
//! Ring 0 — Kernel Core      (timux kernel itself, unrestricted)
//! Ring 1 — Kernel Extension (drivers, filesystems — supervised)
//! Ring 2 — System Services  (init, IPC daemons — capability-gated)
//! Ring 3 — User             (applications — fully capability-isolated)
//! Ring 4 — Sandbox          (untrusted code — maximum restriction)
//!
//! Every cross-ring operation requires a valid CapabilityToken.
//! No capability = no access, regardless of ring level.

use bitflags::bitflags;
use core::sync::atomic::{AtomicU64, Ordering};

/// The 5-level Timux ring hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RingLevel {
    KernelCore      = 0,
    KernelExtension = 1,
    SystemService   = 2,
    User            = 3,
    Sandbox         = 4,
}

impl RingLevel {
    pub fn can_access(self, target: RingLevel) -> bool {
        // Lower ring number = higher privilege
        (self as u8) <= (target as u8)
    }

    pub fn is_privileged(self) -> bool {
        (self as u8) < 2
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::KernelCore      => "ring0:kernel-core",
            Self::KernelExtension => "ring1:kernel-ext",
            Self::SystemService   => "ring2:sys-service",
            Self::User            => "ring3:user",
            Self::Sandbox         => "ring4:sandbox",
        }
    }
}

/// Active ring context for the current execution unit
#[derive(Debug, Clone)]
pub struct Ring {
    pub level: RingLevel,
    pub caps:  CapabilityTable,
}

impl Ring {
    pub fn new(level: RingLevel) -> Self {
        Self { level, caps: CapabilityTable::new() }
    }

    /// Attempt a ring transition. Requires a valid transition capability.
    pub fn transition_to(
        &self,
        target: RingLevel,
        token: &CapabilityToken,
    ) -> Result<Ring, PrivError> {
        if !token.is_valid() {
            return Err(PrivError::InvalidCapability);
        }
        if !token.permits(CapRight::RING_TRANSITION) {
            return Err(PrivError::InsufficientRights);
        }
        // Can only transition to less-privileged rings without special cap
        if (target as u8) < (self.level as u8)
            && !token.permits(CapRight::PRIVILEGE_ESCALATION)
        {
            return Err(PrivError::EscalationDenied);
        }
        Ok(Ring::new(target))
    }

    /// Check if this ring can perform an operation
    pub fn check(&self, right: CapRight) -> Result<(), PrivError> {
        if self.caps.has_right(right) {
            Ok(())
        } else {
            Err(PrivError::InsufficientRights)
        }
    }
}

// ─── Capabilities ──────────────────────────────────────────────────────────

static CAP_COUNTER: AtomicU64 = AtomicU64::new(1);

bitflags! {
    /// Rights encoded in a capability token
    #[derive(Debug, Clone, Copy)]
    pub struct CapRight: u64 {
        const READ               = 1 << 0;
        const WRITE              = 1 << 1;
        const EXEC               = 1 << 2;
        const MAP                = 1 << 3;
        const SEND               = 1 << 4;
        const RECV               = 1 << 5;
        const DELEGATE           = 1 << 6;
        const REVOKE             = 1 << 7;
        const RING_TRANSITION    = 1 << 8;
        const PRIVILEGE_ESCALATION = 1 << 9;
        const IRQ_BIND           = 1 << 10;
        const DMA_ACCESS         = 1 << 11;
        const MMIO_ACCESS        = 1 << 12;
        const PROCESS_SPAWN      = 1 << 13;
        const PROCESS_KILL       = 1 << 14;
        const FS_READ            = 1 << 15;
        const FS_WRITE           = 1 << 16;
        const NET_SEND           = 1 << 17;
        const NET_RECV           = 1 << 18;
        const CLOCK_READ         = 1 << 19;
        const CLOCK_SET          = 1 << 20;
    }
}

/// An unforgeable capability token
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    id:     u64,
    rights: CapRight,
    owner:  RingLevel,
    valid:  bool,
}

impl CapabilityToken {
    /// Mint a new capability (only ring0 can do this directly)
    pub fn mint(rights: CapRight, owner: RingLevel) -> Self {
        Self {
            id: CAP_COUNTER.fetch_add(1, Ordering::SeqCst),
            rights,
            owner,
            valid: true,
        }
    }

    pub fn is_valid(&self) -> bool { self.valid }

    pub fn permits(&self, right: CapRight) -> bool {
        self.valid && self.rights.contains(right)
    }

    /// Delegate a subset of rights to another ring
    pub fn delegate(
        &self,
        rights: CapRight,
        to: RingLevel,
    ) -> Result<CapabilityToken, PrivError> {
        if !self.permits(CapRight::DELEGATE) {
            return Err(PrivError::DelegationDenied);
        }
        // Can only delegate rights you hold
        if !self.rights.contains(rights) {
            return Err(PrivError::InsufficientRights);
        }
        // Cannot delegate to a higher-privilege ring
        if (to as u8) < (self.owner as u8) {
            return Err(PrivError::EscalationDenied);
        }
        Ok(CapabilityToken::mint(rights, to))
    }

    /// Revoke this capability
    pub fn revoke(&mut self) -> Result<(), PrivError> {
        if !self.permits(CapRight::REVOKE) {
            return Err(PrivError::RevocationDenied);
        }
        self.valid = false;
        Ok(())
    }

    pub fn id(&self) -> u64 { self.id }
    pub fn rights(&self) -> CapRight { self.rights }
    pub fn owner(&self) -> RingLevel { self.owner }
}

/// Alias for ergonomic use
pub type Capability = CapabilityToken;

// ─── Capability Table ───────────────────────────────────────────────────────

const MAX_CAPS: usize = 256;

/// Per-task capability table — stores all capabilities a task holds
#[derive(Debug)]
#[derive(Clone)]
pub struct CapabilityTable {
    slots: [Option<CapabilityToken>; MAX_CAPS],
    count: usize,
}

impl CapabilityTable {
    pub fn new() -> Self {
        Self {
            slots: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    pub fn insert(&mut self, cap: CapabilityToken) -> Result<usize, PrivError> {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(cap);
                self.count += 1;
                return Ok(i);
            }
        }
        Err(PrivError::TableFull)
    }

    pub fn get(&self, idx: usize) -> Option<&CapabilityToken> {
        self.slots.get(idx)?.as_ref()
    }

    pub fn remove(&mut self, idx: usize) -> Option<CapabilityToken> {
        let cap = self.slots.get_mut(idx)?.take();
        if cap.is_some() { self.count -= 1; }
        cap
    }

    pub fn has_right(&self, right: CapRight) -> bool {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .any(|c| c.permits(right))
    }

    pub fn count(&self) -> usize { self.count }
}

impl Default for CapabilityTable {
    fn default() -> Self { Self::new() }
}

// ─── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivError {
    InvalidCapability,
    InsufficientRights,
    EscalationDenied,
    DelegationDenied,
    RevocationDenied,
    TableFull,
    RingViolation { from: RingLevel, to: RingLevel },
}
