use clokwerk::{Scheduler, TimeUnits};
use std::thread;
use std::time::Duration;


#[allow(dead_code)]
pub fn run() {
    let mut scheduler = Scheduler::new();
    scheduler.every(1.seconds()).run(|| println!("Periodic task"));

    for _ in 1..10{
        scheduler.run_pending();
        thread::sleep(Duration::from_secs(1));
    }
}