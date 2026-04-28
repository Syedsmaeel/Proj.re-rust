use std::any::Any;
use std::sync::Arc;
use crate::reconciler::hooks::HookSlot;

pub type FiberId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkTag {
    FunctionComponent,
    HostComponent,
    HostRoot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Props {
    None,
    Text(String),
    Map(Vec<(String, String)>),
}

pub type RenderFn = dyn Fn() -> Vec<ChildSpec> + Send + Sync;

#[derive(Clone)]
pub enum ChildSpec {
    Text(String),
    Host { tag: String, text: String },
    Function { name: String, render: Arc<RenderFn> },
}

impl std::fmt::Debug for ChildSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChildSpec::Text(t) => write!(f, "Text({t:?})"),
            ChildSpec::Host { tag, text } => write!(f, "Host {{ tag: {tag:?}, text: {text:?} }}"),
            ChildSpec::Function { name, .. } => write!(f, "Function {{ name: {name:?} }}"),
        }
    }
}

pub struct Fiber {
    pub id: FiberId,
    pub tag: WorkTag,
    pub return_id: Option<FiberId>,
    pub child_id: Option<FiberId>,
    pub sibling_id: Option<FiberId>,
    pub pending_props: Props,
    pub memoized_props: Option<Props>,
    pub memoized_state: Option<Box<dyn Any + Send>>,
    pub alternate_id: Option<FiberId>,
    pub flags: u32,
    pub hooks: Vec<HookSlot>,
    pub host_text: Option<String>,
    pub render_fn: Option<Arc<RenderFn>>,
    pub name: String,
}

impl Fiber {
    pub fn new(id: FiberId, tag: WorkTag, props: Props) -> Self {
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
            hooks: Vec::new(),
            host_text: None,
            render_fn: None,
            name: String::new(),
        }
    }
}

pub mod flags {
    pub const NO_FLAGS: u32 = 0;
    pub const PLACEMENT: u32 = 1 << 1;
    pub const UPDATE: u32 = 1 << 2;
    pub const DELETION: u32 = 1 << 3;
}
