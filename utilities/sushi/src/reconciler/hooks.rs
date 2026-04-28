use std::any::Any;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::reconciler::fiber::FiberId;

pub struct HookSlot {
    pub state: Box<dyn Any + Send>,
}

impl std::fmt::Debug for HookSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HookSlot {{ ... }}")
    }
}

pub struct PendingUpdate {
    pub fiber_id: FiberId,
    pub hook_index: usize,
    pub new_value: Box<dyn Any + Send>,
}

#[derive(Default)]
pub struct UpdateQueue {
    pub pending: Mutex<Vec<PendingUpdate>>,
    pub dirty: Mutex<Vec<FiberId>>,
}

impl UpdateQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn drain(&self) -> Vec<PendingUpdate> {
        std::mem::take(&mut *self.pending.lock().unwrap())
    }
    pub fn drain_dirty(&self) -> Vec<FiberId> {
        std::mem::take(&mut *self.dirty.lock().unwrap())
    }
    pub fn has_work(&self) -> bool {
        !self.pending.lock().unwrap().is_empty() || !self.dirty.lock().unwrap().is_empty()
    }
}

pub struct Dispatcher {
    pub fiber_id: FiberId,
    pub hooks: Vec<HookSlot>,
    pub cursor: usize,
    pub queue: Arc<UpdateQueue>,
}

thread_local! {
    static CURRENT: RefCell<Option<Dispatcher>> = RefCell::new(None);
}

pub fn enter_render(fiber_id: FiberId, prior_hooks: Vec<HookSlot>, queue: Arc<UpdateQueue>) {
    CURRENT.with(|c| {
        *c.borrow_mut() = Some(Dispatcher {
            fiber_id,
            hooks: prior_hooks,
            cursor: 0,
            queue,
        });
    });
}

pub fn exit_render() -> Vec<HookSlot> {
    CURRENT.with(|c| c.borrow_mut().take().map(|d| d.hooks).unwrap_or_default())
}

pub struct Setter<T: 'static + Send> {
    fiber_id: FiberId,
    hook_index: usize,
    queue: Arc<UpdateQueue>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static + Send> Setter<T> {
    pub fn set(&self, new_value: T) {
        self.queue.pending.lock().unwrap().push(PendingUpdate {
            fiber_id: self.fiber_id,
            hook_index: self.hook_index,
            new_value: Box::new(new_value),
        });
        let mut d = self.queue.dirty.lock().unwrap();
        if !d.contains(&self.fiber_id) {
            d.push(self.fiber_id);
        }
    }
}

impl<T: 'static + Send> Clone for Setter<T> {
    fn clone(&self) -> Self {
        Self {
            fiber_id: self.fiber_id,
            hook_index: self.hook_index,
            queue: self.queue.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub fn use_state<T: 'static + Clone + Send>(initial: T) -> (T, Setter<T>) {
    CURRENT.with(|c| {
        let mut g = c.borrow_mut();
        let d = g
            .as_mut()
            .expect("use_state called outside of a component render");
        let idx = d.cursor;
        d.cursor += 1;
        if idx >= d.hooks.len() {
            d.hooks.push(HookSlot {
                state: Box::new(initial.clone()),
            });
        }
        let value = d.hooks[idx]
            .state
            .downcast_ref::<T>()
            .cloned()
            .unwrap_or(initial);
        let setter = Setter::<T> {
            fiber_id: d.fiber_id,
            hook_index: idx,
            queue: d.queue.clone(),
            _phantom: std::marker::PhantomData,
        };
        (value, setter)
    })
}

pub fn use_effect<F: FnOnce() + 'static>(_f: F) {}
