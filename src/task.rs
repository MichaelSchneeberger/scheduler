use crate::scheduler::Task;
use chrono::prelude::{DateTime, Utc};
use std::cmp::Ordering;

pub struct DelayedTask {
    pub task: Box<dyn Task>,
    pub duetime: DateTime<Utc>,
}

impl PartialEq for DelayedTask {
    fn eq(&self, other: &Self) -> bool {
        self.duetime == other.duetime
    }
}

impl Eq for DelayedTask {}

impl Ord for DelayedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        other.duetime.cmp(&self.duetime)
    }
}

impl PartialOrd for DelayedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: FnOnce() + Send> Task for F {
    fn run(self: Box<Self>) {
        self()
    }
}

// impl<F: Fn() + Send> Task for F {
//     fn run(&self) {
//         self()
//     }
// }
