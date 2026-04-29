//! Timux Scheduler — ring-aware preemptive task scheduler

use crate::priv_model::{CapabilityTable, RingLevel};

/// Task state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Zombie,
}

/// A scheduled task — one per thread/process
#[derive(Debug)]
pub struct Task {
    pub id:       u64,
    pub name:     &'static str,
    pub ring:     RingLevel,
    pub state:    TaskState,
    pub caps:     CapabilityTable,
    pub priority: u8,   // 0 = highest, 255 = lowest
    pub ticks:    u64,  // total CPU ticks consumed
    pub ctx:      TaskContext,
}

/// Saved register state (arch-independent view)
#[derive(Debug, Default, Clone)]
pub struct TaskContext {
    pub pc:   usize, // program counter
    pub sp:   usize, // stack pointer
    pub regs: [usize; 16], // general purpose registers
}

impl Task {
    pub fn new(
        id: u64,
        name: &'static str,
        ring: RingLevel,
        priority: u8,
        entry: usize,
        stack: usize,
    ) -> Self {
        let mut ctx = TaskContext::default();
        ctx.pc = entry;
        ctx.sp = stack;
        Self {
            id, name, ring,
            state: TaskState::Ready,
            caps: CapabilityTable::new(),
            priority,
            ticks: 0,
            ctx,
        }
    }

    pub fn is_runnable(&self) -> bool {
        self.state == TaskState::Ready || self.state == TaskState::Running
    }
}

/// Round-robin scheduler with ring-priority weighting
pub struct Scheduler {
    tasks:   alloc::vec::Vec<Task>,
    current: Option<usize>, // index into tasks
    tick:    u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: alloc::vec::Vec::new(), current: None, tick: 0 }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Pick next task — lower ring level and lower priority number = higher weight
    pub fn next(&mut self) -> Option<&mut Task> {
        let len = self.tasks.len();
        if len == 0 { return None; }

        let start = self.current.map(|c| (c + 1) % len).unwrap_or(0);
        let mut best: Option<usize> = None;
        let mut best_score = u32::MAX;

        for i in 0..len {
            let idx = (start + i) % len;
            let t = &self.tasks[idx];
            if !t.is_runnable() { continue; }
            // Score = ring_weight * 100 + priority
            let score = (t.ring as u32) * 100 + t.priority as u32;
            if score < best_score {
                best_score = score;
                best = Some(idx);
            }
        }

        if let Some(idx) = best {
            if let Some(prev) = self.current {
                if self.tasks[prev].state == TaskState::Running {
                    self.tasks[prev].state = TaskState::Ready;
                }
            }
            self.tasks[idx].state = TaskState::Running;
            self.tasks[idx].ticks += 1;
            self.current = Some(idx);
            Some(&mut self.tasks[idx])
        } else {
            None
        }
    }

    pub fn tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn task_count(&self) -> usize { self.tasks.len() }
    pub fn current_task(&self) -> Option<&Task> {
        self.current.map(|i| &self.tasks[i])
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}

extern crate alloc;
