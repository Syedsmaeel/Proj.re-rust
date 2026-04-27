use crate::reconciler::fiber::{Fiber, FiberId, WorkTag, flags};
use crate::reconciler::Reconciler;

impl Reconciler {
    /// The 'commit_root' function (The Synchronous Phase)
    /// This is where we actually apply changes to the host (e.g., TUI or Desktop)
    pub fn commit_root(&mut self, root_id: FiberId) -> Result<(), String> {
        println!("🚀 Committing changes to host...");
        self.commit_work(root_id)?;
        Ok(())
    }

    fn commit_work(&mut self, fiber_id: FiberId) -> Result<(), String> {
        let (flags, tag, child_id, sibling_id) = {
            let fiber = self.fibers.get(&fiber_id).ok_or("Fiber not found")?;
            (fiber.flags, fiber.tag.clone(), fiber.child_id, fiber.sibling_id)
        };

        // 1. Apply effects to this fiber
        if flags & flags::PLACEMENT != 0 {
            println!("  [Placement] Render new element for Fiber #{}", fiber_id);
        }
        if flags & flags::UPDATE != 0 {
            println!("  [Update] Refresh existing element for Fiber #{}", fiber_id);
        }
        if flags & flags::DELETION != 0 {
            println!("  [Deletion] Remove element for Fiber #{}", fiber_id);
        }

        // 2. Commit children
        if let Some(child) = child_id {
            self.commit_work(child)?;
        }

        // 3. Commit siblings
        if let Some(sibling) = sibling_id {
            self.commit_work(sibling)?;
        }

        // 4. Clear flags after commit
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.flags = flags::NO_FLAGS;
        }

        Ok(())
    }
}
