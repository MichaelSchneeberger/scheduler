use std::sync::Arc;
use std::thread;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use scheduler::scheduler::Scheduler;


pub fn main() {
    let s1 = Arc::new(EventLoopScheduler::new("s1"));
    let s2 = Arc::new(EventLoopScheduler::new("s2"));

    let s1_clone = Arc::clone(&s1);
    let s2_clone = Arc::clone(&s2);
    s2.schedule(move || {
        println!("Hello from s2");
        println!("Schedule task on s1");
        Arc::clone(&s1_clone).schedule(move || {
            println!("Hello from s1");
            Arc::clone(&s1_clone).stop();
            Arc::clone(&s2_clone).stop();
        });
    });

    thread::spawn(move || s2.start_loop());
    s1.start_loop();
}
