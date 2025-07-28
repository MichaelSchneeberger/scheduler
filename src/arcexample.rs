use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub fn run() {
    // increases the reference count when aquired (calling the clone() function)
    // when the reference count is zero, then the value is dropped
    let counter = Arc::new(Mutex::new(0));

    let closure1 = {
        let counter = Arc::clone(&counter);
        move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
            println!("Closure 1: {}", *num);
        }
    };

    let closure2 = {
        let counter = Arc::clone(&counter);
        move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
            println!("Closure 2: {}", *num);
        }
    };

    closure1(); // Prints: Closure 1: 1
    closure2(); // Prints: Closure 2: 2
}
