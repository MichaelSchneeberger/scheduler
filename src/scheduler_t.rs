use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

trait Task: Send {
    fn run(&self, scheduler: &dyn Scheduler);
}

// #[derive(Copy, Clone, Eq, PartialEq)]
struct DelayedTask<T>
where
    T: Task,
{
    task: T,
    duetime: DateTime<Utc>,
}

impl<T> PartialEq for DelayedTask<T>
where
    T: Task,
{
    fn eq(&self, other: &Self) -> bool {
        self.duetime == other.duetime
    }
}

impl<T> Eq for DelayedTask<T> where T: Task {}

impl<T: Task> Ord for DelayedTask<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.duetime.cmp(&self.duetime)
    }
}

impl<T: Task> PartialOrd for DelayedTask<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

trait Scheduler {
    fn name(&self) -> &str;
    fn schedule(&self, task: Box<dyn Task>);
    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>);
}

struct TaskFromClosure<F>
where
    F: Fn(&dyn Scheduler),
{
    run_func: F,
}

impl<F: Fn(&dyn Scheduler) + Send> Task for TaskFromClosure<F> {
    fn run(&self, scheduler: &dyn Scheduler) {
        let run_func = &self.run_func;
        run_func(scheduler)
    }
}

struct EventLoopScheduler<T>
where
    T: Task,
{
    name: String,
    is_stopped: Arc<Mutex<bool>>,
    immediate_tasks: Arc<Mutex<VecDeque<Box<dyn Task>>>>,
    // delayed_tasks: Arc<Mutex<BinaryHeap<Box<dyn DelayedTask>>>>,
    delayed_tasks: Arc<Mutex<BinaryHeap<DelayedTask<T>>>>,
    cond_var: Arc<(Mutex<bool>, Condvar)>,
}

enum DelayedTaskAction {
    NoTasks,
    NextDueTime(DateTime<Utc>),
    ImmediateTask(Box<dyn Task>),
}

impl<T: Task> EventLoopScheduler<T> {
    fn start_loop(&self) {
        let is_stopped_mutex = Arc::clone(&self.is_stopped);
        let immediate_tasks_mutex = Arc::clone(&self.immediate_tasks);
        let delayed_tasks_mutex = Arc::clone(&self.delayed_tasks);

        loop {
            let is_stopped = {
                let result = is_stopped_mutex.lock().unwrap();
                *result
            };

            if is_stopped {
                break;
            }

            let immediate_task = {
                let mut deque = immediate_tasks_mutex.lock().unwrap();
                deque.pop_front()
            };

            match immediate_task {
                Some(task) => task.run(self),

                // if no immediate tasks are scheduled, check if a delayed task is due
                None => {
                    let delayed_task = {
                        let mut binary_heap = delayed_tasks_mutex.lock().unwrap();

                        match binary_heap.peek() {
                            None => DelayedTaskAction::NoTasks,
                            Some(first) => {
                                let duetime = first.duetime;
                                if TimeDelta::seconds(0) < duetime - Utc::now() {
                                    DelayedTaskAction::ImmediateTask(Box::new(
                                        binary_heap.pop().unwrap().task,
                                    ))
                                } else {
                                    DelayedTaskAction::NextDueTime(first.duetime)
                                }
                            }
                        }
                    };

                    let mut deque = immediate_tasks_mutex.lock().unwrap();
                    match delayed_task {
                        DelayedTaskAction::ImmediateTask(task) => {
                            deque.push_back(task);
                        }
                        _ => {
                            if deque.is_empty() {
                                let (lock, cvar) = &*self.cond_var;
                                let mut started = lock.lock().unwrap();
                                match delayed_task {
                                    DelayedTaskAction::NextDueTime(duetime) => {
                                        if TimeDelta::seconds(0) < duetime - Utc::now() {
                                            cvar.wait_timeout(
                                                started,
                                                (duetime - Utc::now()).to_std().unwrap(),
                                            )
                                            .unwrap();
                                        }
                                    }
                                    DelayedTaskAction::NoTasks => {
                                        cvar.wait(started).unwrap();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T: Task> Scheduler for EventLoopScheduler<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule(&self, task: Box<dyn Task>) {
        let lock = Arc::clone(&self.immediate_tasks);
        let mut deque = lock.lock().unwrap();
        // let boxed_task = Box::new(task);
        let (lock, cvar) = &*self.cond_var;
        let mut started = lock.lock().unwrap();
        deque.push_back(task);
        cvar.notify_one();
    }

    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>) {
        let lock = Arc::clone(&self.delayed_tasks);
        let mut binary_heap = lock.lock().unwrap();
        // let boxed_task = Box::new(task);
        let (lock, cvar) = &*self.cond_var;
        let mut started = lock.lock().unwrap();
        binary_heap.push(DelayedTask {
            task: *task,
            duetime: duetime,
        });
        cvar.notify_one();
    }
}

fn create_task<'a>(num: i32) -> Box<dyn Task> {
    let closure = {
        move |scheduler: &dyn Scheduler| {
            println!("{}: {}", scheduler.name(), num);

            if num > 0 {
                let task = create_task(num - 1);
                scheduler.schedule(task);
            }
        }
    };

    let task = TaskFromClosure { run_func: closure };
    Box::new(task)
}

#[allow(dead_code)]
pub fn run() {
    let s1 = EventLoopScheduler {
        name: "s1".to_string(),
        is_stopped: Arc::new(Mutex::new(false)),
        immediate_tasks: Arc::new(Mutex::new(VecDeque::new())),
        delayed_tasks: Arc::new(Mutex::new(BinaryHeap::new())),
        cond_var: Arc::new((Mutex::new(false), Condvar::new())),
    };
    let s2 = EventLoopScheduler {
        name: "s2".to_string(),
        is_stopped: Arc::new(Mutex(false)),
        immediate_tasks: Arc::new(Mutex::new(VecDeque::new())),
        delayed_tasks: Arc::new(Mutex::new(BinaryHeap::new())),
        cond_var: Arc::new((Mutex::new(false), Condvar::new())),
    };

    let task = create_task(10);
    s1.schedule(task);

    let task = create_task(10);
    s2.schedule(task);

    thread::spawn(move || s1.start_loop());
    thread::spawn(move || s2.start_loop());

    thread::sleep(Duration::from_secs(1));
}
