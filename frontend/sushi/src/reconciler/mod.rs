pub mod fiber;
pub mod commit;
use fiber::{Fiber, FiberId, WorkTag};
use std::collections::HashMap;
use std::any::Any;

pub struct Reconciler {
    pub fibers: HashMap<FiberId, Fiber>,
    pub next_id: FiberId,
    pub work_in_progress: Option<FiberId>,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            fibers: HashMap::new(),
            next_id: 0,
            work_in_progress: None,
        }
    }

    /// Creates a new Fiber and registers it in the arena
    pub fn create_fiber(&mut self, tag: WorkTag, props: Box<dyn Any>) -> FiberId {
        let id = self.next_id;
        self.next_id += 1;
        let fiber = Fiber::new(id, tag, props);
        self.fibers.insert(id, fiber);
        id
    }

    pub fn begin_work(&mut self, fiber_id: FiberId) -> Result<Option<FiberId>, String> {
        let fiber = self.fibers.get(&fiber_id).ok_or("Fiber not found")?;
        println!("🛠️  BeginWork on Fiber #{} ({:?})", fiber.id, fiber.tag);
        match fiber.tag {
            WorkTag::FunctionComponent => self.update_function_component(fiber_id),
            WorkTag::HostComponent => self.update_host_component(fiber_id),
            WorkTag::HostRoot => self.update_host_root(fiber_id),
        }
    }

    fn update_function_component(&mut self, _fiber_id: FiberId) -> Result<Option<FiberId>, String> { Ok(None) }
    fn update_host_component(&mut self, _fiber_id: FiberId) -> Result<Option<FiberId>, String> { Ok(None) }
    fn update_host_root(&mut self, _fiber_id: FiberId) -> Result<Option<FiberId>, String> { Ok(None) }

    pub fn reconcile_children(&mut self, _return_fiber: FiberId, _current_first_child: Option<FiberId>, _next_children: Vec<Box<dyn Any>>) -> Option<FiberId> {
        None
    }
}
