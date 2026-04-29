//! Timux boot protocol — arch-independent early init

use crate::priv_model::{CapRight, CapabilityToken, RingLevel};
use crate::cap::CapAuthority;
use crate::sched::{Scheduler, Task};

/// Boot info passed from bootloader
#[derive(Debug)]
pub struct BootInfo {
    pub mem_map_addr: usize,
    pub mem_map_len:  usize,
    pub kernel_start: usize,
    pub kernel_end:   usize,
    pub ramdisk_addr: Option<usize>,
    pub ramdisk_size: Option<usize>,
    pub cmdline:      &'static str,
}

/// Early kernel state built during boot
pub struct KernelState {
    pub authority: CapAuthority,
    pub scheduler: Scheduler,
    pub root_cap:  CapabilityToken,
}

impl KernelState {
    /// Initialize the kernel — called once from arch main
    pub fn init(_info: &BootInfo) -> Self {
        let authority = CapAuthority::new(RingLevel::KernelCore);
        let root_cap  = authority.mint_root(RingLevel::KernelCore);
        let scheduler = Scheduler::new();
        Self { authority, scheduler, root_cap }
    }

    /// Spawn the init task (ring2: SystemService)
    pub fn spawn_init(&mut self, entry: usize, stack: usize) {
        let init_cap = self.authority.mint_user();
        let mut task = Task::new(1, "init", RingLevel::SystemService, 0, entry, stack);
        task.caps.insert(init_cap).expect("cap table insert failed");
        self.scheduler.add_task(task);
    }
}
