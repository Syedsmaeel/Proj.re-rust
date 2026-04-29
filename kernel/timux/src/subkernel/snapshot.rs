//! SubKernelSnapshot — full state capture for clone/migrate

use super::instance::{SubKernel, SubKernelConfig, SubKernelId, SubKernelState};

/// A point-in-time capture of a sub-kernel's state
/// Used for: cloning, migration, checkpoint/restore
#[derive(Debug)]
pub struct SubKernelSnapshot {
    pub source_id:    SubKernelId,
    pub config:       SubKernelConfig,
    pub state:        SubKernelState,
    pub tick:         u64,
    pub task_count:   usize,
    pub driver_count: usize,
    pub cap_count:    usize,
    pub parent:       Option<SubKernelId>,
    pub children:     alloc::vec::Vec<SubKernelId>,
}

impl SubKernelSnapshot {
    pub fn capture(sk: &SubKernel) -> Self {
        Self {
            source_id:    sk.id,
            config:       sk.config.clone(),
            state:        sk.state,
            tick:         sk.tick,
            task_count:   sk.scheduler.task_count(),
            driver_count: sk.drivers.count(),
            cap_count:    sk.caps.count(),
            parent:       sk.parent,
            children:     sk.children.clone(),
        }
    }
}

extern crate alloc;
