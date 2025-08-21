use std::time::Duration;

use scheduler::scheduler::{Scheduler, Task};
use scheduler::schedulers::asyncscheduler::AsyncScheduler;
use scheduler::task::TaskFromClosure;

pub fn main() {
    fn count_down(n_iter: u64) -> Box<dyn Task> {
        let closure = {
            move |scheduler: &dyn Scheduler| {
                println!("{}: {}", scheduler.name(), n_iter);

                if n_iter > 0 {
                    let task = count_down(n_iter - 1);
                    scheduler.schedule_relative(Duration::from_secs(1), task);
                } else {
                    scheduler.stop();
                }
            }
        };

        let task = TaskFromClosure { run_func: closure };
        Box::new(task)
    }

    AsyncScheduler::run("main", count_down(5));
}
