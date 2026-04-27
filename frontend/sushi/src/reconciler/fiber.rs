use std::any::Any;

pub type FiberId = usize;

#[derive(Debug, Clone)]
pub enum WorkTag {
    FunctionComponent,
    HostComponent, // e.g. a Button or Label
    HostRoot,
}

/// The Fiber: The unit of work in Sushi (Translated from ReactFiber.js)
pub struct Fiber {
    pub id: FiberId,
    pub tag: WorkTag,
    
    // Relationships (Using IDs instead of pointers for Rust safety)
    pub return_id: Option<FiberId>,
    pub child_id: Option<FiberId>,
    pub sibling_id: Option<FiberId>,
    
    // State & Props
    pub pending_props: Box<dyn Any>,
    pub memoized_props: Option<Box<dyn Any>>,
    pub memoized_state: Option<Box<dyn Any>>,
    
    // The "Alt" fiber (used for double-buffering during updates)
    pub alternate_id: Option<FiberId>,
    
    // Flags for the Commit phase
    pub flags: u32,
}

impl Fiber {
    pub fn new(id: FiberId, tag: WorkTag, props: Box<dyn Any>) -> Self {
        Self {
            id,
            tag,
            return_id: None,
            child_id: None,
            sibling_id: None,
            pending_props: props,
            memoized_props: None,
            memoized_state: None,
            alternate_id: None,
            flags: 0,
        }
    }
}
