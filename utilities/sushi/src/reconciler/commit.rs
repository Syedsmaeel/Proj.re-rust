use crate::host::{DebugHost, Host};
use crate::reconciler::fiber::{flags, FiberId, WorkTag};
use crate::reconciler::Reconciler;

impl Reconciler {
    pub fn commit_root(&mut self, root_id: FiberId) -> Result<(), String> {
        let mut sink = DebugHost::new();
        self.commit_root_with_host(root_id, &mut sink)
    }

    pub fn commit_root_with_host(
        &mut self,
        root_id: FiberId,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        self.commit_work(root_id, host)?;
        Ok(())
    }

    fn commit_work(&mut self, fiber_id: FiberId, host: &mut dyn Host) -> Result<(), String> {
        let snapshot = match self.fibers.get(&fiber_id) {
            Some(f) => (
                f.flags,
                f.tag.clone(),
                f.child_id,
                f.sibling_id,
                f.host_text.clone().unwrap_or_default(),
                f.name.clone(),
            ),
            None => return Ok(()),
        };
        let (flags_v, tag, child_id, sibling_id, text, name) = snapshot;

        let tag_str = match tag {
            WorkTag::FunctionComponent => "fn",
            WorkTag::HostComponent => "host",
            WorkTag::HostRoot => "root",
        };
        let display = if name.is_empty() {
            text.clone()
        } else if text.is_empty() {
            name.clone()
        } else {
            format!("{name}: {text}")
        };

        if flags_v & flags::DELETION != 0 {
            host.delete(fiber_id);
            self.fibers.remove(&fiber_id);
            return Ok(());
        }
        if flags_v & flags::PLACEMENT != 0 {
            host.place(fiber_id, tag_str, &display);
        } else if flags_v & flags::UPDATE != 0 {
            host.update(fiber_id, tag_str, &display);
        }

        if let Some(c) = child_id {
            self.commit_work(c, host)?;
        }
        if let Some(s) = sibling_id {
            self.commit_work(s, host)?;
        }

        if let Some(f) = self.fibers.get_mut(&fiber_id) {
            f.flags = flags::NO_FLAGS;
            f.memoized_props = Some(f.pending_props.clone());
        }
        Ok(())
    }
}
