use crate::scheduler::{Scheduler, Task};
use crate::task::DelayedTask;
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::collections::{BinaryHeap, VecDeque};
use std::future::pending;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

pub struct AsyncScheduler {
    pub name: String,
    pub runtime: Arc<Mutex<Runtime>>,
}

impl AsyncScheduler {
    pub fn new(name: &str) -> AsyncScheduler {
        AsyncScheduler {
            name: name.to_string(),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
        }
    }

    pub fn run(name: &str, task: Box<dyn Task>) {
        let s = AsyncScheduler::new(name);
        s.schedule(task);
        s.start_loop();
    }

    // pub fn start_loop<F: Future>(&self, future: F) -> F::Output {
    pub fn start_loop(&self) {
        let lock = Arc::clone(&self.runtime);
        let runtime = lock.lock().unwrap();
        // runtime.block_on(future)
        runtime.block_on(pending::<()>())
    }
}

impl Scheduler for AsyncScheduler {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule(&self, task: Box<dyn Task>) {
        let lock = Arc::clone(&self.runtime);
        let runtime = lock.lock().unwrap();
        runtime.spawn(async move { task.run(self) });
    }

    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>) {
        self.runtime.spawn(async {
            let duration = (duetime - Utc::now()).to_std().unwrap();
            sleep(duration).await;
            task.run(self)
        });
    }

    fn schedule_relative(&self, duetime: f64, task: Box<dyn Task>) {
        let duration = Duration::from_secs_f64(duetime);
        let duetime_datetime = Utc::now() + TimeDelta::from_std(duration).unwrap();
        self.schedule_absolute(duetime_datetime, task);
    }

    fn stop(&self) {
        //     let mut is_stopped_lock = self.is_stopped.lock().unwrap();
        //     if *is_stopped_lock {
        //         panic!("Scheduler can only be stopped once.")
        //     }
        //     *is_stopped_lock = true;
        //     let (_, cvar) = &*self.cond_var;
        //     cvar.notify_one();
    }
}
