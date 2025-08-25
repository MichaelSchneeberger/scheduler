use std::sync::Arc;
use std::time::Duration;

use scheduler::scheduler::{Scheduler, Task};
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use scheduler::task::TaskFromClosure;

fn count_down<S: Scheduler + 'static>(scheduler: &Arc<S>, n_iter: u64) -> Box<dyn Task> {
    let scheduler = Arc::clone(scheduler);
    let action = {
        move || {
            println!("{}: {}", scheduler.name(), n_iter);

            if n_iter > 0 {
                let task = count_down(&scheduler, n_iter - 1);
                scheduler.schedule_relative(Duration::from_secs(1), task);
            } else {
                scheduler.stop();
            }
        }
    };

    let task = TaskFromClosure { action };
    Box::new(task)
}

pub fn main() {
    let scheduler = Arc::new(EventLoopScheduler::new("main"));
    let task = count_down(&scheduler, 5);
    scheduler.run(task);
}
