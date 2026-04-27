pub mod fiber;
pub mod commit;
use fiber::{Fiber, FiberId, WorkTag};
use std::collections::HashMap;
use std::any::Any;

pub struct Reconciler {
    pub fibers: HashMap<FiberId, Fiber>,
    pub next_id: FiberId,
    
    // The "Real React" Pointers
    pub current_root: Option<FiberId>,
    pub work_in_progress_root: Option<FiberId>,
    pub next_unit_of_work: Option<FiberId>,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            fibers: HashMap::new(),
            next_id: 0,
            current_root: None,
            work_in_progress_root: None,
            next_unit_of_work: None,
        }
    }

    pub fn create_fiber(&mut self, tag: WorkTag, props: Box<dyn Any>) -> FiberId {
        let id = self.next_id;
        self.next_id += 1;
        let fiber = Fiber::new(id, tag, props);
        self.fibers.insert(id, fiber);
        id
    }

    /// The Entry Point: Starts the rendering work
    pub fn schedule_update(&mut self, root_id: FiberId) {
        self.work_in_progress_root = Some(root_id);
        self.next_unit_of_work = Some(root_id);
    }

    /// The Work Loop: Processes one fiber at a time.
    /// Can be called in a loop, and interrupted if 'deadline' is hit.
    pub fn work_loop(&mut self) {
        while let Some(unit) = self.next_unit_of_work {
            self.next_unit_of_work = self.perform_unit_of_work(unit);
        }
        
        // Once loop is done, we commit
        if let Some(root) = self.work_in_progress_root {
            let _ = self.commit_root(root);
            self.current_root = Some(root);
            self.work_in_progress_root = None;
        }
    }

    fn perform_unit_of_work(&mut self, fiber_id: FiberId) -> Option<FiberId> {
        // 1. Begin Work (Step Down)
        println!("  ⬇️  Performing Unit of Work: Fiber #{}", fiber_id);
        let _ = self.begin_work(fiber_id);

        // Return the child to continue moving down
        if let Some(fiber) = self.fibers.get(&fiber_id) {
            if let Some(child) = fiber.child_id {
                return Some(child);
            }
        }

        // 2. No child? Complete Work and look for siblings (Step Across/Up)
        let mut current_id = Some(fiber_id);
        while let Some(id) = current_id {
            self.complete_unit_of_work(id);
            
            if let Some(fiber) = self.fibers.get(&id) {
                if let Some(sibling) = fiber.sibling_id {
                    return Some(sibling);
                }
                current_id = fiber.return_id;
            } else {
                current_id = None;
            }
        }

        None
    }

    pub fn begin_work(&mut self, fiber_id: FiberId) -> Result<(), String> {
        // (Translation of ReactFiberBeginWork.js)
        Ok(())
    }

    fn complete_unit_of_work(&mut self, fiber_id: FiberId) {
        println!("  ⬆️  Completing Unit of Work: Fiber #{}", fiber_id);
        // (Translation of ReactFiberCompleteWork.js - gathering side-effects)
    }
}
