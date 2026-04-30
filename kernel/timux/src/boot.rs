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
use crate::sched::{Scheduler, Task};
use crate::mm::LinkedListAllocator;
use crate::subkernel::{SubKernelManager, SubKernelConfig};
use crate::subkernel::instance::SubKernelProfile;

/// Heap size: 4MB default
pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

/// The global allocator — registered with Rust's #[global_allocator]
/// Declared here, registered in arch-specific main
pub static ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

/// Boot info passed from bootloader (Multiboot2 / UEFI / OpenSBI etc.)
#[derive(Debug)]
pub struct BootInfo {
    pub heap_start:   usize,   // where to place the kernel heap
    pub heap_size:    usize,   // how many bytes for the heap
    pub kernel_start: usize,
    pub kernel_end:   usize,
    pub ramdisk_addr: Option<usize>,
    pub ramdisk_size: Option<usize>,
    pub cmdline:      &'static str,
}

impl BootInfo {
    /// Minimal boot info for testing — heap at a fixed address
    pub fn minimal(heap_start: usize) -> Self {
        Self {
            heap_start,
            heap_size: HEAP_SIZE,
            kernel_start: 0,
            kernel_end: 0,
            ramdisk_addr: None,
            ramdisk_size: None,
            cmdline: "",
        }
    }
}

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
        // Step 1: Initialize the heap allocator
        ALLOCATOR.init(info.heap_start, info.heap_size);

        // Step 2: Create the root capability authority
        let authority = CapAuthority::new(RingLevel::KernelCore);
        let root_cap  = authority.mint_root(RingLevel::KernelCore);

        // Step 3: Create the sub-kernel manager
        let sk_manager = SubKernelManager::new();

        // Step 4: Create the master scheduler
        let scheduler = Scheduler::new();

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

    /// Spawn a legacy task directly on the master scheduler (ring 2)
    pub fn spawn_task(&mut self, name: &'static str, entry: usize, stack: usize) {
        let init_cap = self.authority.mint_user();
        let mut task = Task::new(
            self.scheduler.task_count() as u64 + 1,
            name,
            RingLevel::SystemService,
            0,
            entry,
            stack,
        );
        task.caps.insert(init_cap).expect("cap insert failed");
        self.scheduler.add_task(task);
    }
}
