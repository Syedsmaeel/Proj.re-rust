pub mod fiber;
use fiber::{Fiber, FiberId};
use std::collections::HashMap;

/// The Reconciler: Manages the Fiber tree and updates
pub struct Reconciler {
    pub fibers: HashMap<FiberId, Fiber>,
    pub next_id: FiberId,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            fibers: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_fiber(&mut self, tag: fiber::WorkTag, props: Box<dyn std::any::Any>) -> FiberId {
        let id = self.next_id;
        self.next_id += 1;
        let fiber = Fiber::new(id, tag, props);
        self.fibers.insert(id, fiber);
        id
    }
}
