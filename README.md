# Scheduler

**Scheduler** is a Rust project that implements the `Scheduler` trait for different execution contexts such as *threads* and *tokio Runtime*.
The `Scheduler` trait implements the following methods:
* `schedule(&self, task: Box<dyn Task>)`
* `schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>)`
* `schedule_relative(&self, duration: Duration, task: Box<dyn Task>)`

## Basic Example

```rust
use scheduler::scheduler::Scheduler;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use std::sync::Arc;

pub fn main() {
    let scheduler = Arc::new(EventLoopScheduler::new("scheduler"));
    let action = {
        let scheduler = Arc::clone(&scheduler);
        move || {
            println!("task executed");
            scheduler.stop();
        }
    };
    scheduler.run(Box::new(action));
}
```

