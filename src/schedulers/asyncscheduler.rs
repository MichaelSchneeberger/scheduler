use crate::scheduler::{Scheduler, Task};
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
    recv: Rc<RefCell<mpsc::UnboundedReceiver<Command>>>,
    send: mpsc::UnboundedSender<Command>,
}

impl AsyncScheduler {
    pub fn new(name: &str) -> AsyncScheduler {
        let (send, recv) = mpsc::unbounded_channel();
        AsyncScheduler {
            name: name.to_string(),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
            recv: Rc::new(RefCell::new(recv)),
            send: send,
        }
    }

    pub fn run(name: &str, task: Box<dyn Task>) {
        let s = AsyncScheduler::new(name);
        s.schedule(task);
        AsyncScheduler::start_loop(Arc::new(s));
    }

    pub fn start_loop(scheduler: Arc<Self>) {
        let scheduler = Arc::clone(&scheduler);
        let lock = Arc::clone(&scheduler.runtime);
        let recv = Rc::clone(&scheduler.recv);

        let local = LocalSet::new();
        local.spawn_local(async move {
            loop {
                match (*recv.borrow_mut()).recv().await {
                    Some(Command::Task(task_fn)) => {
                        let scheduler = Arc::clone(&scheduler);
                        tokio::task::spawn_local(async move { task_fn(scheduler).await });
                    }
                    Some(Command::Stop) => break,
                    _ => break,
                }
            }
        });

        let runtime = lock.lock().unwrap();
        runtime.block_on(local);
    }
}

impl Scheduler for AsyncScheduler {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule(&self, task: Box<dyn Task>) {
        let task = Command::Task(Box::new(move |s| Box::pin(async move { task.run(&*s) })));

        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }

    fn schedule_absolute(&self, duetime: DateTime<Utc>, task: Box<dyn Task>) {
        let task = Command::Task(Box::new(move |s| {
            // let s_rc = Arc::new(s);
            Box::pin(async move {
                let duration = (duetime - Utc::now()).to_std().unwrap();
                sleep(duration).await;
                task.run(&*s)
            })
        }));

        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.")
    }

    fn schedule_relative(&self, duration: Duration, task: Box<dyn Task>) {
        let duetime_datetime = Utc::now() + TimeDelta::from_std(duration).unwrap();
        self.schedule_absolute(duetime_datetime, task);
    }

    fn stop(&self) {
        self.send
            .send(Command::Stop)
            .expect("Thread with LocalSet has shut down.")
    }
}
