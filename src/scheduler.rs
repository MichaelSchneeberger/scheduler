use std::time::Duration;

use chrono::prelude::{DateTime, Utc};

pub trait Task: Send {
    fn run(&self);
}

pub trait Scheduler: Send + Sync {
    fn name(&self) -> &str;
    fn schedule(&self, task: Box<dyn Task>);
    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>);
    fn schedule_relative(&self, duration: Duration, task: Box<dyn Task>);
    fn stop(&self);
}
