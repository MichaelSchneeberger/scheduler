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
        let (lock, cond_var) = &*self.state_cv;

        loop {
            let is_stopped = {
                let result = (*lock.lock().unwrap()).is_stopped;
                result
            };

            if is_stopped {
                break;
            }

            let immediate_task = {
                let mut state = lock.lock().unwrap();
                state.immediate_tasks.pop_front()
            };

            match immediate_task {
                Some(task) => task.run(),

                // if no immediate tasks are scheduled, check if a delayed task is due
                None => {
                    enum NextDelayedTask {
                        NoTasks,
                        NextDueTime(DateTime<Utc>),
                        ImmediateTask(Box<dyn Task>),
                    }

                    // A delayed task is due if its duetime is in the past
                    let next_delayed_task = {
                        let mut state = lock.lock().unwrap();

                        match state.delayed_tasks.peek() {
                            None => NextDelayedTask::NoTasks,
                            Some(first) => {
                                let duetime = first.duetime;
                                if duetime - Utc::now() < TimeDelta::seconds(0) {
                                    NextDelayedTask::ImmediateTask(
                                        state.delayed_tasks.pop().unwrap().task,
                                    )
                                } else {
                                    NextDelayedTask::NextDueTime(first.duetime)
                                }
                            }
                        }
                    };

                    // Here, the 'state' mutex is locked, but will be unlocked again throught the Condvar
                    // wait_timeout and wait functions.

                    let mut state = lock.lock().unwrap();
                    match next_delayed_task {
                        NextDelayedTask::ImmediateTask(task) => {
                            state.immediate_tasks.push_back(task);
                        }

                        _ => {
                            // check whether there are still no new immediate tasks, otherwise
                            // prioritize those first
                            if state.immediate_tasks.is_empty() {
                                match next_delayed_task {

                                    // Wait for a new task to be scheduled or for the next delayed task to become due.
                                    NextDelayedTask::NextDueTime(duetime) => {

                                        // Duetime of the next delayed task is still in the future
                                        if TimeDelta::seconds(0) < duetime - Utc::now() {
                                            let _ = cond_var
                                                .wait_timeout(
                                                    state,
                                                    (duetime - Utc::now()).to_std().unwrap(),
                                                )
                                                .unwrap();
                                        }

                                        // Duetime of the next delayed task is now due; run it in the
                                        // next loop iteration
                                        else {}
                                    }

                                    // No delayed tasks scheduled; wait for a new task to be scheduled
                                    NextDelayedTask::NoTasks => {
                                        let _result = cond_var.wait(state).unwrap();
                                        // state = cond_var.wait(state).unwrap();
                                        // drop(state);
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
        let (state, cond_var) = &*self.state_cv;
        let deque = &mut (*state.lock().unwrap()).immediate_tasks;
        deque.push_back(Box::new(task));
        cond_var.notify_one();
    }

    fn schedule_absolute<T: Task + 'static>(&self, duetime: DateTime<Utc>, task: T) {
        let (state, cond_var) = &*self.state_cv;
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
        let (state, cond_var) = &*self.state_cv;

        let is_stopped_ref = &mut (*state.lock().unwrap()).is_stopped;
        if *is_stopped_ref {
            panic!("Scheduler can only be stopped once.")
        }
        *is_stopped_ref = true;
        cond_var.notify_one();
    }
}
