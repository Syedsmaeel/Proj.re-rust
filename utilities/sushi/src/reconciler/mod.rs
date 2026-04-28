pub mod commit;
pub mod fiber;
pub mod hooks;
pub mod scheduler;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fiber::{flags, ChildSpec, Fiber, FiberId, Props, RenderFn, WorkTag};
use hooks::{enter_render, exit_render, UpdateQueue};
use scheduler::Scheduler;

pub struct Reconciler {
    pub fibers: HashMap<FiberId, Fiber>,
    pub next_id: FiberId,
    pub current_root: Option<FiberId>,
    pub work_in_progress_root: Option<FiberId>,
    pub next_unit_of_work: Option<FiberId>,
    pub update_queue: Arc<UpdateQueue>,
    pub scheduler: Scheduler,
}

impl Default for Reconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            fibers: HashMap::new(),
            next_id: 0,
            current_root: None,
            work_in_progress_root: None,
            next_unit_of_work: None,
            update_queue: UpdateQueue::new(),
            scheduler: Scheduler::unbounded(),
        }
    }

    pub fn queue(&self) -> Arc<UpdateQueue> {
        self.update_queue.clone()
    }

    pub fn create_fiber(&mut self, tag: WorkTag, props: Props) -> FiberId {
        let id = self.next_id;
        self.next_id += 1;
        let mut fiber = Fiber::new(id, tag, props);
        fiber.flags |= flags::PLACEMENT;
        self.fibers.insert(id, fiber);
        id
    }

    pub fn create_function_fiber<F>(&mut self, name: impl Into<String>, render: F) -> FiberId
    where
        F: Fn() -> Vec<ChildSpec> + Send + Sync + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        let mut fiber = Fiber::new(id, WorkTag::FunctionComponent, Props::None);
        fiber.flags |= flags::PLACEMENT;
        let boxed: Arc<RenderFn> = Arc::new(render);
        fiber.render_fn = Some(boxed);
        fiber.name = name.into();
        self.fibers.insert(id, fiber);
        id
    }

    pub fn create_host_root(&mut self) -> FiberId {
        let id = self.create_fiber(WorkTag::HostRoot, Props::None);
        self.current_root = Some(id);
        id
    }

    pub fn append_child(&mut self, parent: FiberId, child: FiberId) {
        let head = self.fibers.get(&parent).and_then(|p| p.child_id);
        match head {
            None => {
                if let Some(p) = self.fibers.get_mut(&parent) {
                    p.child_id = Some(child);
                }
            }
            Some(first) => {
                let mut cur = first;
                loop {
                    let next = self.fibers.get(&cur).and_then(|f| f.sibling_id);
                    match next {
                        Some(n) => cur = n,
                        None => {
                            if let Some(f) = self.fibers.get_mut(&cur) {
                                f.sibling_id = Some(child);
                            }
                            break;
                        }
                    }
                }
            }
        }
        if let Some(c) = self.fibers.get_mut(&child) {
            c.return_id = Some(parent);
        }
    }

    pub fn schedule_update(&mut self, root_id: FiberId) {
        self.work_in_progress_root = Some(root_id);
        self.next_unit_of_work = Some(root_id);
    }

    pub fn flush_updates(&mut self) -> bool {
        let mut any = false;
        for u in self.update_queue.drain() {
            if let Some(f) = self.fibers.get_mut(&u.fiber_id) {
                if u.hook_index < f.hooks.len() {
                    f.hooks[u.hook_index].state = u.new_value;
                    f.flags |= flags::UPDATE;
                    any = true;
                }
            }
        }
        for d in self.update_queue.drain_dirty() {
            if let Some(f) = self.fibers.get_mut(&d) {
                f.flags |= flags::UPDATE;
                any = true;
            }
        }
        if any {
            if let Some(root) = self.current_root {
                self.schedule_update(root);
            }
        }
        any
    }

    pub fn work_loop(&mut self) {
        self.work_loop_with_deadline(None);
        if let Some(root) = self.work_in_progress_root.take() {
            let _ = self.commit_root(root);
            self.current_root = Some(root);
        }
    }

    pub fn work_loop_with_deadline(&mut self, budget: Option<Duration>) {
        self.scheduler = match budget {
            Some(b) => Scheduler::with_budget(b),
            None => Scheduler::unbounded(),
        };
        while let Some(unit) = self.next_unit_of_work {
            if self.scheduler.should_yield() {
                return;
            }
            self.next_unit_of_work = self.perform_unit_of_work(unit);
        }
    }

    pub fn is_render_complete(&self) -> bool {
        self.next_unit_of_work.is_none() && self.work_in_progress_root.is_some()
    }

    pub fn finish_commit(&mut self, host: &mut dyn crate::host::Host) -> Result<(), String> {
        if self.next_unit_of_work.is_some() {
            return Ok(());
        }
        if let Some(root) = self.work_in_progress_root.take() {
            self.commit_root_with_host(root, host)?;
            self.current_root = Some(root);
        }
        Ok(())
    }

    pub fn pump(&mut self, host: &mut dyn crate::host::Host, budget: Duration) {
        let had_work = self.flush_updates() || self.next_unit_of_work.is_some();
        if !had_work {
            return;
        }
        self.work_loop_with_deadline(Some(budget));
        if self.next_unit_of_work.is_none() {
            let _ = self.finish_commit(host);
        }
    }

    fn perform_unit_of_work(&mut self, fiber_id: FiberId) -> Option<FiberId> {
        let _ = self.begin_work(fiber_id);
        if let Some(f) = self.fibers.get(&fiber_id) {
            if let Some(c) = f.child_id {
                return Some(c);
            }
        }
        let mut current = Some(fiber_id);
        while let Some(id) = current {
            self.complete_unit_of_work(id);
            if let Some(f) = self.fibers.get(&id) {
                if let Some(s) = f.sibling_id {
                    return Some(s);
                }
                current = f.return_id;
            } else {
                current = None;
            }
        }
        None
    }

    pub fn begin_work(&mut self, fiber_id: FiberId) -> Result<(), String> {
        let snapshot = match self.fibers.get(&fiber_id) {
            Some(f) => (
                f.tag.clone(),
                f.render_fn.clone(),
                f.memoized_props.clone(),
                f.pending_props.clone(),
            ),
            None => return Err("fiber not found".into()),
        };
        let (tag, render_fn, props_old, props_new) = snapshot;

        if let Some(old) = &props_old {
            if *old != props_new {
                if let Some(f) = self.fibers.get_mut(&fiber_id) {
                    f.flags |= flags::UPDATE;
                }
            }
        }

        match tag {
            WorkTag::FunctionComponent => {
                if let Some(rf) = render_fn {
                    let prior_hooks = self
                        .fibers
                        .get_mut(&fiber_id)
                        .map(|f| std::mem::take(&mut f.hooks))
                        .unwrap_or_default();
                    enter_render(fiber_id, prior_hooks, self.update_queue.clone());
                    let children_spec = (rf)();
                    let new_hooks = exit_render();
                    if let Some(f) = self.fibers.get_mut(&fiber_id) {
                        f.hooks = new_hooks;
                    }
                    self.reconcile_children(fiber_id, children_spec);
                }
            }
            WorkTag::HostComponent => {
                if let Props::Text(t) = &props_new {
                    if let Some(f) = self.fibers.get_mut(&fiber_id) {
                        if f.host_text.as_deref() != Some(t.as_str()) {
                            f.host_text = Some(t.clone());
                            f.flags |= flags::UPDATE;
                        }
                    }
                }
            }
            WorkTag::HostRoot => {}
        }
        Ok(())
    }

    fn reconcile_children(&mut self, parent: FiberId, specs: Vec<ChildSpec>) {
        let mut existing: Vec<FiberId> = Vec::new();
        let mut cur = self.fibers.get(&parent).and_then(|f| f.child_id);
        while let Some(id) = cur {
            existing.push(id);
            cur = self.fibers.get(&id).and_then(|f| f.sibling_id);
        }

        let mut new_children: Vec<FiberId> = Vec::new();
        for (i, spec) in specs.into_iter().enumerate() {
            if i < existing.len() {
                let id = existing[i];
                self.update_existing_child(id, &spec);
                new_children.push(id);
            } else {
                let id = self.create_child_from_spec(spec);
                if let Some(f) = self.fibers.get_mut(&id) {
                    f.return_id = Some(parent);
                }
                new_children.push(id);
            }
        }

        let reused = new_children.len().min(existing.len());
        for &id in &existing[reused..] {
            if let Some(f) = self.fibers.get_mut(&id) {
                f.flags |= flags::DELETION;
            }
        }

        for w in new_children.windows(2) {
            if let Some(f) = self.fibers.get_mut(&w[0]) {
                f.sibling_id = Some(w[1]);
            }
        }
        if let Some(&last) = new_children.last() {
            if let Some(f) = self.fibers.get_mut(&last) {
                f.sibling_id = None;
            }
        }
        if let Some(f) = self.fibers.get_mut(&parent) {
            f.child_id = new_children.first().copied();
        }
    }

    fn create_child_from_spec(&mut self, spec: ChildSpec) -> FiberId {
        match spec {
            ChildSpec::Text(t) => {
                let id = self.create_fiber(WorkTag::HostComponent, Props::Text(t.clone()));
                if let Some(f) = self.fibers.get_mut(&id) {
                    f.host_text = Some(t);
                }
                id
            }
            ChildSpec::Host { tag, text } => {
                let id = self.create_fiber(WorkTag::HostComponent, Props::Text(text.clone()));
                if let Some(f) = self.fibers.get_mut(&id) {
                    f.host_text = Some(text);
                    f.name = tag;
                }
                id
            }
            ChildSpec::Function { name, render } => {
                let id = self.next_id;
                self.next_id += 1;
                let mut fiber = Fiber::new(id, WorkTag::FunctionComponent, Props::None);
                fiber.flags |= flags::PLACEMENT;
                fiber.render_fn = Some(render);
                fiber.name = name;
                self.fibers.insert(id, fiber);
                id
            }
        }
    }

    fn update_existing_child(&mut self, id: FiberId, spec: &ChildSpec) {
        let new_text = match spec {
            ChildSpec::Text(t) => Some(t.clone()),
            ChildSpec::Host { text, .. } => Some(text.clone()),
            ChildSpec::Function { .. } => None,
        };
        if let Some(f) = self.fibers.get_mut(&id) {
            match new_text {
                Some(t) => {
                    if f.host_text.as_deref() != Some(t.as_str()) {
                        f.host_text = Some(t.clone());
                        f.pending_props = Props::Text(t);
                        f.flags |= flags::UPDATE;
                    }
                }
                None => f.flags |= flags::UPDATE,
            }
        }
    }

    fn complete_unit_of_work(&mut self, _fiber_id: FiberId) {}
}
