use std::time::{Duration, Instant};

pub struct Scheduler {
    deadline: Option<Instant>,
}

impl Scheduler {
    pub fn unbounded() -> Self {
        Self { deadline: None }
    }
    pub fn with_budget(budget: Duration) -> Self {
        Self {
            deadline: Some(Instant::now() + budget),
        }
    }
    pub fn should_yield(&self) -> bool {
        self.deadline.map_or(false, |d| Instant::now() >= d)
    }
    pub fn reset(&mut self, budget: Duration) {
        self.deadline = Some(Instant::now() + budget);
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
}
