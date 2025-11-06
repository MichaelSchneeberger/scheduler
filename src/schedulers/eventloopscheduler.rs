use crate::scheduler::{Scheduler, Task};
use crate::task::DelayedTask;
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::collections::{BinaryHeap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

struct State {
    is_stopped: bool,
    immediate_tasks: VecDeque<Box<dyn Task>>,
    delayed_tasks: BinaryHeap<DelayedTask>,
}

pub struct EventLoopScheduler {
    pub name: String,
    state_cv: Arc<(Mutex<State>, Condvar)>,
}

impl EventLoopScheduler {
    pub fn new(name: &str) -> EventLoopScheduler {
        let state = State {
            is_stopped: false,
            immediate_tasks: VecDeque::new(),
            delayed_tasks: BinaryHeap::new(),
        };

        EventLoopScheduler {
            name: name.to_string(),
            state_cv: Arc::new((Mutex::new(state), Condvar::new())),
        }
    }

    pub fn run<T: Task + 'static>(&self, task: T) {
        self.schedule(task);
        self.start_loop();
    }

    pub fn start_loop(&self) {
        let state_cv = Arc::clone(&self.state_cv);
        let (state, cond_var) = &*state_cv;

        loop {
            let is_stopped = {
                let result = (*state.lock().unwrap()).is_stopped;
                result
            };

            if is_stopped {
                break;
            }

            let immediate_task = {
                let deque = &mut(*state.lock().unwrap()).immediate_tasks;
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
                        let binary_heap = &mut (*state.lock().unwrap()).delayed_tasks;

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

                    let mut state_guard = state.lock().unwrap();
                    let deque = &mut (*state_guard).immediate_tasks;
                    match delayed_task {
                        DelayedTaskAction::ImmediateTask(task) => {
                            deque.push_back(task);
                        }
                        _ => {
                            if deque.is_empty() {
                                match delayed_task {
                                    DelayedTaskAction::NextDueTime(duetime) => {
                                        // Wait for a new task or until the next delayed task is due.
                                        if TimeDelta::seconds(0) < duetime - Utc::now() {
                                            let _ = cond_var
                                                .wait_timeout(
                                                    state_guard,
                                                    (duetime - Utc::now()).to_std().unwrap(),
                                                )
                                                .unwrap();
                                        }
                                    }
                                    DelayedTaskAction::NoTasks => {
                                        state_guard = cond_var.wait(state_guard).unwrap();
                                        drop(state_guard);
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

    fn schedule<T: Task + 'static>(&self, task: T) {
        let state_cv = Arc::clone(&self.state_cv);
        let (state, cond_var) = &*state_cv;
        let deque = &mut (*state.lock().unwrap()).immediate_tasks;
        deque.push_back(Box::new(task));
        cond_var.notify_one();
    }

    fn schedule_absolute<T: Task + 'static>(&self, duetime: DateTime<Utc>, task: T) {
        let state_cv = Arc::clone(&self.state_cv);
        let (state, cond_var) = &*state_cv;
        let binary_heap = &mut (*state.lock().unwrap()).delayed_tasks;
        binary_heap.push(DelayedTask {
            task: Box::new(task),
            duetime: duetime,
        });
        cond_var.notify_one();
    }

    fn schedule_relative<T: Task + 'static>(&self, duration: Duration, task: T) {
        let duetime_datetime = Utc::now() + TimeDelta::from_std(duration).unwrap();
        self.schedule_absolute(duetime_datetime, task);
    }

    fn stop(&self) {
        let state_cv = Arc::clone(&self.state_cv);
        let (state, cond_var) = &*state_cv;
        let is_stopped_ref = &mut (*state.lock().unwrap()).is_stopped;
        if *is_stopped_ref {
            panic!("Scheduler can only be stopped once.")
        }
        *is_stopped_ref = true;
        cond_var.notify_one();
    }
}
