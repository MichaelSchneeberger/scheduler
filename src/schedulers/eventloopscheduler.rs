use crate::scheduler::{Scheduler, Task};
use crate::task::DelayedTask;
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::collections::{BinaryHeap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub struct EventLoopScheduler {
    pub name: String,
    pub is_stopped: Arc<Mutex<bool>>,
    pub immediate_tasks: Arc<Mutex<VecDeque<Box<dyn Task>>>>,
    pub delayed_tasks: Arc<Mutex<BinaryHeap<DelayedTask>>>,
    pub cond_var: Arc<(Mutex<()>, Condvar)>,
}

impl EventLoopScheduler {
    pub fn new(name: &str) -> EventLoopScheduler {
        EventLoopScheduler {
            name: name.to_string(),
            is_stopped: Arc::new(Mutex::new(false)),
            immediate_tasks: Arc::new(Mutex::new(VecDeque::new())),
            delayed_tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            cond_var: Arc::new((Mutex::new(()), Condvar::new())),
        }
    }

    pub fn run(&self, task: Box<dyn Task>) {
        self.schedule(task);
        self.start_loop();
    }

    pub fn start_loop(&self) {
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
                Some(task) => task.run(),

                // if no immediate tasks are scheduled, check if a delayed task is due
                None => {
                    enum DelayedTaskAction {
                        NoTasks,
                        NextDueTime(DateTime<Utc>),
                        ImmediateTask(Box<dyn Task>),
                    }
                    let delayed_task = {
                        let mut binary_heap = delayed_tasks_mutex.lock().unwrap();

                        match binary_heap.peek() {
                            None => DelayedTaskAction::NoTasks,
                            Some(first) => {
                                let duetime = first.duetime;
                                if duetime - Utc::now() < TimeDelta::seconds(0) {
                                    DelayedTaskAction::ImmediateTask(
                                        binary_heap.pop().unwrap().task,
                                    )
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
                                let mut guard = lock.lock().unwrap();
                                match delayed_task {
                                    DelayedTaskAction::NextDueTime(duetime) => {
                                        if TimeDelta::seconds(0) < duetime - Utc::now() {
                                            let _ = cvar
                                                .wait_timeout(
                                                    guard,
                                                    (duetime - Utc::now()).to_std().unwrap(),
                                                )
                                                .unwrap();
                                        }
                                    }
                                    DelayedTaskAction::NoTasks => {
                                        guard = cvar.wait(guard).unwrap();
                                        drop(guard);
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

impl Scheduler for EventLoopScheduler {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule(&self, task: Box<dyn Task>) {
        let lock = Arc::clone(&self.immediate_tasks);
        let mut deque = lock.lock().unwrap();
        let (_, cvar) = &*self.cond_var;
        deque.push_back(task);
        cvar.notify_one();
    }

    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>) {
        let lock = Arc::clone(&self.delayed_tasks);
        let mut binary_heap = lock.lock().unwrap();
        let (_, cvar) = &*self.cond_var;
        binary_heap.push(DelayedTask {
            task: task,
            duetime: duetime,
        });
        cvar.notify_one();
    }

    fn schedule_relative(&self, duration: Duration, task: Box<dyn Task>) {
        let duetime_datetime = Utc::now() + TimeDelta::from_std(duration).unwrap();
        // println!("{}", duetime_datetime);
        self.schedule_absolute(duetime_datetime, task);
    }

    fn stop(&self) {
        let mut is_stopped_lock = self.is_stopped.lock().unwrap();
        if *is_stopped_lock {
            panic!("Scheduler can only be stopped once.")
        }
        *is_stopped_lock = true;
        let (_, cvar) = &*self.cond_var;
        cvar.notify_one();
    }
}
