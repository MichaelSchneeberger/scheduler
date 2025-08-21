use crate::scheduler::{Scheduler, Task};
use crate::task::DelayedTask;
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::{BinaryHeap, VecDeque};
use std::future::pending;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio::time::sleep;

enum Command {
    Task(Box<dyn FnOnce(Arc<dyn Scheduler>) -> Pin<Box<dyn Future<Output = ()>>>>),
    Stop,
}

pub struct AsyncScheduler {
    pub name: String,
    pub runtime: Arc<Mutex<Runtime>>,
    // pub runtime: Runtime,
    // recv: Rc<RefCell<mpsc::UnboundedReceiver<Box<dyn Task>>>>,
    recv: Rc<
        RefCell<
            mpsc::UnboundedReceiver<
                Box<dyn FnOnce(Arc<dyn Scheduler>) -> Pin<Box<dyn Future<Output = ()>>>>,
            >,
        >,
    >,
    pub send: mpsc::UnboundedSender<
        Box<dyn FnOnce(Arc<dyn Scheduler>) -> Pin<Box<dyn Future<Output = ()>>>>,
    >,
}

impl AsyncScheduler {
    pub fn new(name: &str) -> AsyncScheduler {
        let (send, mut recv) = mpsc::unbounded_channel();
        AsyncScheduler {
            name: name.to_string(),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
            // runtime: Runtime::new().unwrap(),
            recv: Rc::new(RefCell::new(recv)),
            send: send,
        }
    }

    pub fn run(name: &str, task: Box<dyn Task>) {
        let mut s = AsyncScheduler::new(name);
        s.schedule(task);
        AsyncScheduler::start_loop(Arc::new(s));
    }

    // pub fn start_loop<F: Future>(&self, future: F) -> F::Output {
    pub fn start_loop(self_arc: Arc<Self>) {
        let self_clone = Arc::clone(&self_arc);

        // pub fn start_loop(&self, self_arc: Self) {
        // pub fn start_loop(&self) {
        let lock = Arc::clone(&self_clone.runtime);
        let runtime = lock.lock().unwrap();
        // runtime.block_on(pending::<()>())

        let local = LocalSet::new();
        let mut recv = Rc::clone(&self_clone.recv);
        let self1 = Arc::clone(&self_arc);
        local.spawn_local(async move {
            loop {
                match (*recv.borrow_mut()).recv().await {
                    Some(task) => {
                        let self2 = Arc::clone(&self1);
                        // tokio::task::spawn_local(async move {
                        tokio::task::spawn_local(async move {
                            // let self_clone = Arc::clone(&self2);
                            task(self2).await
                        });
                    }
                    _ => break,
                }
            }
        });
        runtime.block_on(local);
    }
}

impl Scheduler for AsyncScheduler {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule(&self, task: Box<dyn Task>) {
        // let lock = Arc::clone(&self.runtime);
        // let runtime = lock.lock().unwrap();
        // runtime.spawn(async move { task.run(self) });

        self.send
            .send(Box::new(move |s| {
                // let s_rc = Arc::new(s);
                Box::pin(async move {
                    // let s_result = Arc::try_unwrap(s.clone());
                    // match s_result {
                    //     Ok(s_val) => task.run(s_val),
                    //     Err(msg) => panic!("not good"),
                    // }
                    task.run(s)
                })
            }))
            .expect("Thread with LocalSet has shut down.");
    }

    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>) {
        self.send
            .send(Box::new(move |s| {
                Box::pin(async move {
                    let duration = (duetime - Utc::now()).to_std().unwrap();
                    sleep(duration).await;
                    task.run(s)
                })
            }))
            .expect("Thread with LocalSet has shut down.")
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
