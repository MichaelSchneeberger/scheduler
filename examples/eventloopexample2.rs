use std::sync::Arc;
use std::thread;
use std::time::Duration;

use scheduler::scheduler::Scheduler;
use scheduler::scheduler::Task;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;

fn count_down<S: Scheduler + 'static>(
    scheduler: &Arc<S>,
    counter: u64,
    pause_ms: u64,
) -> impl Task + 'static {
    let scheduler = Arc::clone(scheduler);
    move || {
        println!("{}: {}", scheduler.name(), counter);

        if counter > 0 {
            let task = count_down(&scheduler, counter - 1, pause_ms);
            scheduler.schedule_relative(Duration::from_millis(pause_ms), task);
        } else {
            scheduler.stop();
        }
    }
}

pub fn main() {
    let s1 = Arc::new(EventLoopScheduler::new("s1"));
    let s2 = Arc::new(EventLoopScheduler::new("s2"));

    let task = count_down(&s1, 5, 1000);
    s1.schedule(task);

    let task = count_down(&s2, 20, 200);
    s2.schedule(task);

    thread::spawn(move || s1.start_loop());
    thread::spawn(move || s2.start_loop());

    thread::sleep(Duration::from_secs(6));
}
