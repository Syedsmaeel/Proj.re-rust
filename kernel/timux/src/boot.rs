//! Timux boot protocol — arch-independent early init
//!
//! Boot sequence:
//! 1. Arch-specific code sets up the stack, disables interrupts
//! 2. Calls KernelState::init(boot_info)
//! 3. Heap allocator is initialized from boot_info memory map
//! 4. Master capability authority is created (ring 0 only)
//! 5. Sub-kernel manager initialized
//! 6. Init sub-kernel spawned
//! 7. Scheduler runs

use crate::priv_model::{CapRight, CapabilityToken, RingLevel};
use crate::cap::CapAuthority;
use crate::sched::Scheduler;
use crate::mm::LinkedListAllocator;
use crate::subkernel::{SubKernelManager, SubKernelConfig};
use crate::subkernel::instance::SubKernelProfile;

pub use re_core::protocol::BootInfo;

/// Heap size: 4MB default (fallback if not specified in BootInfo)
pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

/// The global allocator — registered with Rust's #[global_allocator]
pub static ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

/// Full kernel state — built during boot, lives forever
pub struct KernelState {
    pub authority:  CapAuthority,
    pub scheduler:  Scheduler,
    pub sk_manager: SubKernelManager,
    pub root_cap:   CapabilityToken,
}

impl KernelState {
    /// Initialize the kernel
    ///
    /// # Safety
    /// Must be called exactly once, from ring 0 context
    pub unsafe fn init(info: &BootInfo) -> Self {
        // Verify BootInfo integrity
        assert_eq!(info.magic, BootInfo::MAGIC, "Invalid Timux BootInfo magic");

        // Step 1: Initialize the heap allocator
        let heap_size = if info.heap_size > 0 { info.heap_size } else { HEAP_SIZE };
        ALLOCATOR.init(info.heap_start, heap_size);

        // Step 2: Create the root capability authority
        // Timux is Ring -1 (Sovereign Hypervisor Base Layer)
        let authority = CapAuthority::new(RingLevel::KernelCore);
        let root_cap  = authority.mint_root(RingLevel::KernelCore);

        // Step 3: Create the sub-kernel manager
        let sk_manager = SubKernelManager::new();

        // Step 4: Create the master scheduler
        let scheduler = Scheduler::new();

        // TODO: Parse Sovereign Blueprints from info.blueprint_addr
        // TODO: Inject entropy from info.entropy_seed

        KernelState { authority, scheduler, sk_manager, root_cap }
    }

    /// Spawn the init sub-kernel — the first OS instance
    pub fn spawn_init_sk(&mut self) {
        let cap = self.authority.mint(
            CapRight::PROCESS_SPAWN | CapRight::SEND | CapRight::RECV
            | CapRight::FS_READ | CapRight::FS_WRITE | CapRight::NET_SEND
            | CapRight::NET_RECV | CapRight::CLOCK_READ,
            RingLevel::KernelCore,
        );
        let config = SubKernelConfig::new("init", SubKernelProfile::GeneralPurpose);
        self.sk_manager
            .spawn(&cap, config, None)
            .expect("failed to spawn init sub-kernel");
    }
}
