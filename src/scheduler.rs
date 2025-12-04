use std::time::Duration;

use chrono::prelude::{DateTime, Utc};

pub trait Task: Send {
    // fn run(&self);
    fn run(self : Box<Self>);
}

pub trait Scheduler: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn schedule<T: Task + 'static>(&self, task: T);
    fn schedule_absolute<T: Task + 'static>(&self, duetime: DateTime<Utc>, task: T);
    fn schedule_relative<T: Task + 'static>(&self, duration: Duration, task: T);
    fn stop(&self);
}
