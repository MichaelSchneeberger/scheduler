use scheduler::scheduler::{Scheduler, Task};
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use std::sync::Arc;
use std::time::Duration;

fn count_down<S: Scheduler + 'static>(scheduler: &Arc<S>, counter: u64) -> impl Task + 'static {
    let scheduler = Arc::clone(scheduler);
    // let c = move || {
    //     println!("{}: {}", scheduler.name(), counter);
    //
    //     if counter > 0 {
    //         let task = count_down(&scheduler, counter - 1);
    //         scheduler.schedule_relative(Duration::from_secs(1), task);
    //     } else {
    //         scheduler.stop();
    //     }
    // };
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
