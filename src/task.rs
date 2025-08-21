use crate::scheduler::{Scheduler, Task};
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

// pub struct TaskFromClosure<F>
// where
//     F: Fn(Arc<dyn Scheduler>),
// {
//     pub run_func: F,
// }
//
// impl<F: Fn(Arc<dyn Scheduler>) + Send> Task for TaskFromClosure<F> {
//     fn run(&self, scheduler: Arc<dyn Scheduler>) {
//         let run_func = &self.run_func;
//         run_func(scheduler)
//     }
// }

pub struct TaskFromClosure<F>
where
    F: Fn(&dyn Scheduler),
{
    pub run_func: F,
}

impl<F: Fn(&dyn Scheduler) + Send> Task for TaskFromClosure<F> {
    fn run(&self, scheduler: &dyn Scheduler) {
        let run_func = &self.run_func;
        run_func(scheduler)
    }
}
