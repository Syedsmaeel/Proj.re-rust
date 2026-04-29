//! Capability subsystem — minting, delegation, revocation, IPC passing

use crate::priv_model::{CapRight, CapabilityToken, PrivError, RingLevel};

/// Global capability authority — only ring0 can call mint()
pub struct CapAuthority {
    ring: RingLevel,
}

impl CapAuthority {
    /// Create a new authority. Panics if not ring0.
    pub fn new(ring: RingLevel) -> Self {
        assert!(
            ring == RingLevel::KernelCore,
            "CapAuthority can only be created by KernelCore (ring0)"
        );
        Self { ring }
    }

    /// Mint a root capability with full rights for a given ring
    pub fn mint_root(&self, for_ring: RingLevel) -> CapabilityToken {
        CapabilityToken::mint(CapRight::all(), for_ring)
    }

    /// Mint a restricted capability
    pub fn mint(&self, rights: CapRight, for_ring: RingLevel) -> CapabilityToken {
        CapabilityToken::mint(rights, for_ring)
    }

    /// Mint a user-safe capability (no kernel rights)
    pub fn mint_user(&self) -> CapabilityToken {
        let user_rights = CapRight::READ
            | CapRight::WRITE
            | CapRight::EXEC
            | CapRight::SEND
            | CapRight::RECV
            | CapRight::DELEGATE
            | CapRight::FS_READ
            | CapRight::FS_WRITE
            | CapRight::NET_SEND
            | CapRight::NET_RECV
            | CapRight::CLOCK_READ
            | CapRight::PROCESS_SPAWN;
        CapabilityToken::mint(user_rights, RingLevel::User)
    }

    /// Mint a sandbox capability (read-only, no network, no spawn)
    pub fn mint_sandbox(&self) -> CapabilityToken {
        CapabilityToken::mint(
            CapRight::READ | CapRight::EXEC,
            RingLevel::Sandbox,
        )
    }

    /// Mint a driver capability (kernel extension rights)
    pub fn mint_driver(&self) -> CapabilityToken {
        let driver_rights = CapRight::READ
            | CapRight::WRITE
            | CapRight::MAP
            | CapRight::IRQ_BIND
            | CapRight::DMA_ACCESS
            | CapRight::MMIO_ACCESS
            | CapRight::RING_TRANSITION;
        CapabilityToken::mint(driver_rights, RingLevel::KernelExtension)
    }
}

/// Verify a capability token for a specific operation
pub fn verify(
    token: &CapabilityToken,
    required: CapRight,
    caller_ring: RingLevel,
) -> Result<(), PrivError> {
    if !token.is_valid() {
        return Err(PrivError::InvalidCapability);
    }
    if !token.permits(required) {
        return Err(PrivError::InsufficientRights);
    }
    // Token must belong to the caller's ring or a less-privileged ring
    if (token.owner() as u8) < (caller_ring as u8) {
        return Err(PrivError::RingViolation {
            from: caller_ring,
            to: token.owner(),
        });
    }
    Ok(())
}
