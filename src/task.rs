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

pub struct TaskFromClosure<F>
where
    F: Fn(),
{
    pub run_func: F,
}

impl<F: Fn() + Send> Task for TaskFromClosure<F> {
    fn run(&self) {
        (&self.run_func)()
    }
}
