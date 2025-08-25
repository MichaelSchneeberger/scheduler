use scheduler::scheduler::Scheduler;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use std::sync::Arc;

pub fn main() {
    let scheduler = Arc::new(EventLoopScheduler::new("main"));
    let action = {
        let scheduler = Arc::clone(&scheduler);
        move || {
            println!("task executed");
            scheduler.stop();
        }
    };
    scheduler.run(Box::new(action));
}
