use std::sync::Arc;
use std::thread;
use std::time::Duration;

use scheduler::scheduler::Scheduler;
use scheduler::schedulers::asyncscheduler::AsyncScheduler;
use tokio::time::sleep;

pub fn main() {
    let s1 = Arc::new(AsyncScheduler::new("s1"));
    let s2 = Arc::new(AsyncScheduler::new("s2"));

    let delay = {
        let s1 = Arc::clone(&s1);
        move || {
            let s1 = Arc::clone(&s1);
            async move {
                sleep(Duration::from_secs(1)).await;
                println!("Stopped.");
                s1.stop();
            }
        }
    };

    thread::spawn({
        let s2 = Arc::clone(&s2);
        move || s2.start_loop()
    });

    s1.run(move || {
        s2.schedule_async(delay());
    });
}
