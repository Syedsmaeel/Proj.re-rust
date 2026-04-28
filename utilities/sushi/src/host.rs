use std::collections::BTreeMap;

use crate::reconciler::fiber::FiberId;

pub trait Host {
    fn place(&mut self, fiber_id: FiberId, tag: &str, text: &str);
    fn update(&mut self, fiber_id: FiberId, tag: &str, text: &str);
    fn delete(&mut self, fiber_id: FiberId);
}

pub struct DebugHost {
    pub log: Vec<String>,
    pub silent: bool,
}

impl DebugHost {
    pub fn new() -> Self {
        Self {
            log: Vec::new(),
            silent: false,
        }
    }
    pub fn silent() -> Self {
        Self {
            log: Vec::new(),
            silent: true,
        }
    }
}

impl Default for DebugHost {
    fn default() -> Self {
        Self::new()
    }
}

impl Host for DebugHost {
    fn place(&mut self, fiber_id: FiberId, tag: &str, text: &str) {
        let line = format!("[place  #{fiber_id} {tag}] {text}");
        if !self.silent {
            println!("  {line}");
        }
        self.log.push(line);
    }
    fn update(&mut self, fiber_id: FiberId, tag: &str, text: &str) {
        let line = format!("[update #{fiber_id} {tag}] {text}");
        if !self.silent {
            println!("  {line}");
        }
        self.log.push(line);
    }
    fn delete(&mut self, fiber_id: FiberId) {
        let line = format!("[delete #{fiber_id}]");
        if !self.silent {
            println!("  {line}");
        }
        self.log.push(line);
    }
}

pub struct TuiHost {
    pub elements: BTreeMap<FiberId, (String, String)>,
}

impl TuiHost {
    pub fn new() -> Self {
        Self {
            elements: BTreeMap::new(),
        }
    }
    pub fn lines(&self) -> Vec<String> {
        self.elements.values().map(|(_, t)| t.clone()).collect()
    }
    pub fn render_text(&self) -> String {
        self.lines().join("\n")
    }
}

impl Default for TuiHost {
    fn default() -> Self {
        Self::new()
    }
}

impl Host for TuiHost {
    fn place(&mut self, fiber_id: FiberId, tag: &str, text: &str) {
        self.elements
            .insert(fiber_id, (tag.to_string(), text.to_string()));
    }
    fn update(&mut self, fiber_id: FiberId, tag: &str, text: &str) {
        self.elements
            .insert(fiber_id, (tag.to_string(), text.to_string()));
    }
    fn delete(&mut self, fiber_id: FiberId) {
        self.elements.remove(&fiber_id);
    }
}
