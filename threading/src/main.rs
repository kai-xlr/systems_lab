use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct AtomicCounter {
    counter: AtomicUsize,
}

pub struct MutexCounter {
    counter: Mutex<usize>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

impl MutexCounter {
    pub fn new() -> Self {
        Self {
            counter: Mutex::new(0),
        }
    }

    pub fn increment(&self) {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
    }

    pub fn get(&self) -> usize {
        *self.counter.lock().unwrap()
    }
}

fn benchmark_counter(counter_type: &str, iterations: usize, thread_count: usize) {
    let start = Instant::now();
    
    match counter_type {
        "atomic" => {
            let counter = Arc::new(AtomicCounter::new());
            let mut handles = vec![];
            
            for _ in 0..thread_count {
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..iterations {
                        counter_clone.increment();
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
            
            let duration = start.elapsed();
            let final_count = counter.get();
            let expected = iterations * thread_count;
            
            println!("[{}] Time: {:?}", counter_type, duration);
            println!("[{}] Final count: {} (expected: {})", counter_type, final_count, expected);
            println!("[{}] Correct: {}", counter_type, final_count == expected);
        }
        "mutex" => {
            let counter = Arc::new(MutexCounter::new());
            let mut handles = vec![];
            
            for _ in 0..thread_count {
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..iterations {
                        counter_clone.increment();
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
            
            let duration = start.elapsed();
            let final_count = counter.get();
            let expected = iterations * thread_count;
            
            println!("[{}] Time: {:?}", counter_type, duration);
            println!("[{}] Final count: {} (expected: {})", counter_type, final_count, expected);
            println!("[{}] Correct: {}", counter_type, final_count == expected);
        }
        _ => println!("Unknown counter type: {}", counter_type),
    }
}

fn main() {
    let iterations = 100_000;
    let thread_count = 8;
    
    println!("Benchmarking with {} iterations per thread, {} threads\n", iterations, thread_count);
    
    benchmark_counter("atomic", iterations, thread_count);
    println!();
    benchmark_counter("mutex", iterations, thread_count);
}
