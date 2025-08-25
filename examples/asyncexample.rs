use std::sync::Arc;
use std::time::Duration;

use scheduler::scheduler::{Scheduler, Task};
use scheduler::schedulers::asyncscheduler::AsyncScheduler;

fn count_down<S: Scheduler + 'static>(scheduler: &Arc<S>, n_iter: u64) -> impl Task + 'static {
    let scheduler = Arc::clone(scheduler);
    move || {
        println!("{}: {}", scheduler.name(), n_iter);

        if n_iter > 0 {
            let task = count_down(&scheduler, n_iter - 1);
            scheduler.schedule_relative(Duration::from_secs(1), task);
        } else {
            scheduler.stop();
        }
    }
}

pub fn main() {
    let scheduler = Arc::new(AsyncScheduler::new("scheduler"));
    let task = count_down(&scheduler, 5);
    scheduler.run(task);
}
