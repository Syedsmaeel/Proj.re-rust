//! Fractal Scheduler
//! Delegates CPU budgets to sub-kernels.

use crate::subkernel::instance::SubKernelId;

pub struct CpuBudget {
    pub quota_ms: u64,
    pub priority: u8,
}

pub struct FractalScheduler {
    pub budgets: alloc::collections::BTreeMap<SubKernelId, CpuBudget>,
}

impl FractalScheduler {
    pub fn new() -> Self {
        Self { budgets: alloc::collections::BTreeMap::new() }
    }

    pub fn set_budget(&mut self, sk_id: SubKernelId, budget: CpuBudget) {
        self.budgets.insert(sk_id, budget);
    }
}
