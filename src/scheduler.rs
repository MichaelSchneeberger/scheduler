use chrono::prelude::{DateTime, Utc};

pub trait Task: Send {
    fn run(&self, scheduler: &dyn Scheduler);
}

pub trait Scheduler {
    fn name(&self) -> &str;
    fn schedule(&self, task: Box<dyn Task>);
    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>);
    fn schedule_relative(&self, duetime: f64, task: Box<dyn Task>);
    fn stop(&self);
}
