use crate::scheduler::{Scheduler, Task};
use chrono::TimeDelta;
use chrono::prelude::{DateTime, Utc};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio::time::sleep;

enum Command {
    Task(Pin<Box<dyn Future<Output = ()> + Send>>),
    Stop,
}

pub struct AsyncScheduler {
    pub name: String,
    pub runtime: Arc<Mutex<Runtime>>,
    recv: Arc<Mutex<mpsc::UnboundedReceiver<Command>>>,
    send: mpsc::UnboundedSender<Command>,
}

impl AsyncScheduler {
    pub fn new(name: &str) -> AsyncScheduler {
        let (send, recv) = mpsc::unbounded_channel();
        AsyncScheduler {
            name: name.to_string(),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
            recv: Arc::new(Mutex::new(recv)),
            send: send,
        }
    }

    pub fn run<T: Task + 'static>(&self, task: T) {
        self.schedule(task);
        self.start_loop();
    }

    pub fn start_loop(&self) {
        let recv = Arc::clone(&self.recv);
        let local = LocalSet::new();
        local.spawn_local(async move {
            let recv = Arc::clone(&recv);
            let mut recv_guard = recv.lock().unwrap();
            loop {
                match recv_guard.recv().await {
                    Some(Command::Task(task_fn)) => {
                        tokio::task::spawn_local(async move { task_fn.await });
                    }
                    Some(Command::Stop) => break,
                    _ => break,
                }
            }
        });

        let runtime = Arc::clone(&self.runtime);
        let runtime_guard = runtime.lock().unwrap();
        runtime_guard.block_on(local);
    }

    pub fn schedule_async<F: Future<Output = ()> + Send + 'static>(&self, future: F) {
        let task = Command::Task(Box::pin(future));

        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }
}

impl Scheduler for AsyncScheduler {
    fn name(&self) -> &str {
        &self.name
    }

    fn schedule<T: Task + 'static>(&self, task: T) {
        let boxed_task = Box::new(task);
        let task = Command::Task(Box::pin(async move { boxed_task.run() }));

        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }

    fn schedule_absolute<T: Task + 'static>(&self, duetime: DateTime<Utc>, task: T) {

        let boxed_task = Box::new(task);
        let task = Command::Task(            // let s_rc = Arc::new(s);
            Box::pin(async move {
                let duration = (duetime - Utc::now()).to_std().unwrap();
                sleep(duration).await;
                boxed_task.run()
            })
        );

        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.")
    }

    fn schedule_relative<T: Task + 'static>(&self, duration: Duration, task: T) {
        let duetime_datetime = Utc::now() + TimeDelta::from_std(duration).unwrap();
        self.schedule_absolute(duetime_datetime, task);
    }

    fn stop(&self) {
        self.send
            .send(Command::Stop)
            .expect("Thread with LocalSet has shut down.")
    }
}
