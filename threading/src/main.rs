use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct MutexSPSCQueue<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
}

pub struct AtomicCounter {
    counter: AtomicUsize,
}

pub struct MutexCounter {
    counter: Mutex<usize>,
}

pub struct LockFreeRingBuffer<T> {
    buffer: Box<[UnsafeCell<Option<T>>]>, // Array of UnsafeCell
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
}

// SAFETY: LockFreeRingBuffer uses atomic operations for synchronization
// The head/tail pointers ensure no two threads access the same buffer slot simultaneously
unsafe impl<T: Send> Sync for LockFreeRingBuffer<T> {}

impl<T> MutexSPSCQueue<T> {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn send(&self, item: T) -> Result<(), T> {
        let mut queue = self.buffer.lock().unwrap();
        queue.push_back(item);
        Ok(())
    }
    pub fn receive(&self) -> Option<T> {
        let mut queue = self.buffer.lock().unwrap();
        queue.pop_front()
    }
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

impl<T> LockFreeRingBuffer<T> {
    pub fn new(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let mask = capacity - 1;

        // Create array of UnsafeCell<Option<T>>
        let buffer: Vec<UnsafeCell<Option<T>>> =
            (0..capacity).map(|_| UnsafeCell::new(None)).collect();

        Self {
            buffer: buffer.into_boxed_slice(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            mask,
        }
    }

    pub fn send(&self, item: T) -> Result<(), T> {
        let current_head = self.head.load(Ordering::Relaxed);
        let next_head = (current_head + 1) & self.mask;

        let current_tail = self.tail.load(Ordering::Acquire);
        if next_head == current_tail {
            return Err(item); // Buffer full, return the item back
        }

        let cell = &self.buffer[current_head];
        unsafe {
            *cell.get() = Some(item);
        }

        // Update head to next position
        self.head.store(next_head, Ordering::Release);

        Ok(())
    }

    pub fn receive(&self) -> Option<T> {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (current_tail + 1) & self.mask;

        let current_head = self.head.load(Ordering::Acquire);
        if current_head == current_tail {
            return None; // Buffer empty, return None
        }

        let cell = &self.buffer[current_tail];
        let item = unsafe { (*cell.get()).take() };

        // Update tail to next position
        self.tail.store(next_tail, Ordering::Release);

        item
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
            println!("[{}] Time: {:?}", counter_type, duration);
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
            println!("[{}] Time: {:?}", counter_type, duration);
        }
        _ => println!("Unknown counter type: {}", counter_type),
    }
}

fn benchmark_ring_buffer(buffer_type: &str, iterations: usize, _thread_count: usize) {
    let start = Instant::now();

    match buffer_type {
        "lockfree" => {
            let buffer = Arc::new(LockFreeRingBuffer::new(1024));

            let buffer_producer = Arc::clone(&buffer);
            let producer_handle = thread::spawn(move || {
                for i in 0..iterations {
                    while buffer_producer.send(i).is_err() {
                        // Buffer full, spin wait
                    }
                }
            });

            let buffer_consumer = Arc::clone(&buffer);
            let consumer_handle = thread::spawn(move || {
                let mut received = 0;
                while received < iterations {
                    if let Some(_item) = buffer_consumer.receive() {
                        received += 1;
                    }
                }
            });

            producer_handle.join().unwrap();
            consumer_handle.join().unwrap();

            let duration = start.elapsed();
            println!("[{}] Time: {:?}", buffer_type, duration);
        }
        "mutex" => {
            let queue = Arc::new(MutexSPSCQueue::new());

            let queue_producer = Arc::clone(&queue);
            let producer_handle = thread::spawn(move || {
                for i in 0..iterations {
                    queue_producer.send(i).unwrap();
                }
            });

            let queue_consumer = Arc::clone(&queue);
            let consumer_handle = thread::spawn(move || {
                let mut received = 0;
                while received < iterations {
                    if let Some(_item) = queue_consumer.receive() {
                        received += 1;
                    }
                }
            });

            producer_handle.join().unwrap();
            consumer_handle.join().unwrap();

            let duration = start.elapsed();
            println!("[{}] Time: {:?}", buffer_type, duration);
        }
        _ => println!("Unknown buffer type: {}", buffer_type),
    }
}

fn main() {
    let iterations = 100_000;
    let thread_count = 8;

    println!("=== Counter Benchmarks ===");
    benchmark_counter("atomic", iterations, thread_count);
    println!();
    benchmark_counter("mutex", iterations, thread_count);

    println!("\n=== Ring Buffer Benchmarks ===");
    benchmark_ring_buffer("lockfree", iterations, 1);
    println!();
    benchmark_ring_buffer("mutex", iterations, 1);
}
