# Scheduler

**Scheduler** is a Rust project that implements the `Scheduler` trait for different execution contexts such as *threads* and *tokio Runtime*.
The `Scheduler` trait (inspired by the ReactiveX framework) implements the following methods:
* `schedule<T: Task + 'static>(&self, task: T)`
* `schedule_absolute<T: Task + 'static>(&self, duetime: DateTime<Utc>, task: T)`
* `schedule_relative<T: Task + 'static>(&self, duration: Duration, task: T)`

## Basic Example

The example schedules a task every second until a counter value becomes zero. 

```rust
use scheduler::scheduler::{Scheduler, Task};
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use std::sync::Arc;
use std::time::Duration;

fn count_down<S: Scheduler + 'static>(scheduler: &Arc<S>, counter: u64) -> impl Task + 'static {
    let scheduler = Arc::clone(scheduler);
    move || {
        println!("{}: {}", scheduler.name(), counter);

        if counter > 0 {
            let task = count_down(&scheduler, counter - 1);
            scheduler.schedule_relative(Duration::from_secs(1), task);
        } else {
            scheduler.stop();
        }
    }
}

pub fn main() {
    let scheduler = Arc::new(EventLoopScheduler::new("scheduler"));
    let task = count_down(&scheduler, 5);
    scheduler.run(task);
}
```

The code outputs:

```
scheduler: 5
scheduler: 4
scheduler: 3
scheduler: 2
scheduler: 1
scheduler: 0
```
