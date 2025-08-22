# Scheduler

**Scheduler** is a Rust project that implements the `Scheduler` trait for different execution contexts such as *threads* and *tokio Runtime*.


## Basic Example

```rust
use scheduler::scheduler::Scheduler;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use scheduler::task::TaskFromClosure;
use std::sync::Arc;

pub fn main() {
    let main = Arc::new(EventLoopScheduler::new("main"));
    let closure = {
        let main = Arc::clone(&main);
        move || {
            println!("task executed");
            main.stop();
        }
    };
    let task = TaskFromClosure { run_func: closure };
    main.run(Box::new(task));
}
```

