use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use scheduler::scheduler::Scheduler;
use scheduler::scheduler::Task;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use scheduler::task::TaskFromClosure;

fn count_down<S: Scheduler + 'static>(
    scheduler: &Arc<S>,
    n_iter: u64,
    pause_ms: u64,
) -> Box<dyn Task> {
    let scheduler = Arc::clone(scheduler);
    let closure = {
        move || {
            println!("{}: {}", scheduler.name(), n_iter);

            if n_iter > 0 {
                let task = count_down(&scheduler, n_iter - 1, pause_ms);
                scheduler.schedule_relative(Duration::from_millis(pause_ms), task);
            } else {
                scheduler.stop();
            }
        }
    };

    let task = TaskFromClosure { run_func: closure };
    Box::new(task)
}

// fn create_task<'a>(num: u64, pause_ms: u64) -> Box<dyn Task> {
//     let closure = {
//         move |scheduler: &dyn Scheduler| {
//             println!("{}: {}", scheduler.name(), num);
//
//             if num > 0 {
//                 let task = create_task(num - 1, pause_ms);
//                 scheduler.schedule(task);
//                 thread::sleep(Duration::from_millis(pause_ms));
//             }
//         }
//     };
//
//     let task = TaskFromClosure { run_func: closure };
//     Box::new(task)
// }

pub fn main() {
    let s1 = Arc::new(EventLoopScheduler {
        name: "s1".to_string(),
        is_stopped: Arc::new(Mutex::new(false)),
        immediate_tasks: Arc::new(Mutex::new(VecDeque::new())),
        delayed_tasks: Arc::new(Mutex::new(BinaryHeap::new())),
        cond_var: Arc::new((Mutex::new(()), Condvar::new())),
    });
    let s2 = Arc::new(EventLoopScheduler {
        name: "s2".to_string(),
        is_stopped: Arc::new(Mutex::new(false)),
        immediate_tasks: Arc::new(Mutex::new(VecDeque::new())),
        delayed_tasks: Arc::new(Mutex::new(BinaryHeap::new())),
        cond_var: Arc::new((Mutex::new(()), Condvar::new())),
    });

    let task = count_down(&s1, 5, 1000);
    s1.schedule(task);

    let task = count_down(&s2, 20, 200);
    s2.schedule(task);

    thread::spawn(move || s1.start_loop());
    thread::spawn(move || s2.start_loop());

    thread::sleep(Duration::from_secs(6));
}
