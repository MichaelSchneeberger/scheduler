use std::collections::VecDeque;
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::Duration;


struct Scheduler<'a> {
    name: String,
    deque: Arc<Mutex<VecDeque<Box<dyn Fn(&Scheduler) + 'a>>>>,
}

unsafe impl<'a> Send for Scheduler<'a> {}

impl<'a> Scheduler<'a> {
    fn start_loop(&self) {
        let lock = Arc::clone(&self.deque);

        loop {
            let entry = {
                let mut deque = lock.lock().unwrap();
                deque.pop_front()
            };

            match entry {
                None => break,
                Some(f) => f(self),
            }
        }
    }

    // fn schedule<F>(&self, task: F) where F: Fn(&Scheduler) + 'a {
    fn schedule(&self, task: Box<impl Fn(&Scheduler) + 'a>) {
        let lock = Arc::clone(&self.deque);
        let mut deque = lock.lock().unwrap();

        // coersion from impl to dyn
        deque.push_back(task);
    }
}



fn create_task<'a>(num: i32) -> Box<impl Fn(&Scheduler<'_>)> {
    let closure = {
        move |scheduler: &Scheduler| {
            println!("{}: {}", scheduler.name, num);
            
            if num > 0 {
                let task = create_task(num - 1);
                scheduler.schedule(task);
            }
        }
    };

    Box::new(closure)
}

#[allow(dead_code)]
pub fn run() {

    let s1 = Scheduler{
        name: "s1".to_string(),
        deque: Arc::new(Mutex::new(VecDeque::new())),
    };
    let s2 = Scheduler{
        name: "s2".to_string(),
        deque: Arc::new(Mutex::new(VecDeque::new())),
    };

    let task = create_task(10);
    s1.schedule(task);

    let task = create_task(10);
    s2.schedule(task);

    thread::spawn(move || s1.start_loop());
    thread::spawn(move || s2.start_loop());

    thread::sleep(Duration::from_secs(1));
}