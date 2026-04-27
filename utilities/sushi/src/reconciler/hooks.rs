use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct Hook {
    pub state: Box<dyn Any>,
}

pub struct HookContext {
    pub hooks: Vec<Hook>,
    pub current_hook: usize,
}

impl HookContext {
    pub fn new() -> Self {
        Self { hooks: vec![], current_hook: 0 }
    }
}
