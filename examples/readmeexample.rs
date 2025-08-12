use scheduler::scheduler::Scheduler;
use scheduler::schedulers::eventloopscheduler::EventLoopScheduler;
use scheduler::task::TaskFromClosure;

pub fn main() {
    let closure = {
        move |scheduler: &dyn Scheduler| {
            println!("task executed");
            scheduler.stop();
        }
    };
    let task = TaskFromClosure { run_func: closure };
    EventLoopScheduler::run("main", Box::new(task));
}
