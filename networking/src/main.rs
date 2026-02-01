use std::cell::UnsafeCell;
use std::mem;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// CPU pinning for Linux
#[cfg(target_os = "linux")]
fn pin_thread_to_core(core_id: usize) {
    use libc::{cpu_set_t, sched_setaffinity};

    let mut cpu_set: cpu_set_t = unsafe { mem::zeroed() };
    unsafe {
        libc::CPU_SET(core_id, &mut cpu_set);
        sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set);
    }
}

// Fixed-size market message
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MarketMessage {
    message_type: u8,
    symbol: [u8; 8],
    price: u64,
    quantity: u32,
    timestamp: u64,
}

impl MarketMessage {
    fn new(symbol: &str, price: f64, quantity: u32) -> Self {
        let mut symbol_bytes = [0u8; 8];
        let symbol_len = symbol.len().min(8);
        symbol_bytes[..symbol_len].copy_from_slice(symbol.as_bytes());

        let price_fixed = (price * 10000.0) as u64;

        Self {
            message_type: 1,
            symbol: symbol_bytes,
            price: price_fixed,
            quantity,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64,
        }
    }
}

// Send + Sync traits for threading
unsafe impl<T: Send> Send for LockFreeRingBuffer<T> {}
unsafe impl<T: Sync> Sync for LockFreeRingBuffer<T> {}

// Lock-free ring buffer
pub struct LockFreeRingBuffer<T> {
    buffer: Box<[UnsafeCell<Option<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
}

impl<T> LockFreeRingBuffer<T> {
    pub fn new(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let mask = capacity - 1;
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
            return Err(item);
        }

        let cell = &self.buffer[current_head];
        unsafe {
            *cell.get() = Some(item);
        }
        self.head.store(next_head, Ordering::Release);
        Ok(())
    }

    pub fn receive(&self) -> Option<T> {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let current_head = self.head.load(Ordering::Acquire);

        if current_head == current_tail {
            return None;
        }

        let cell = &self.buffer[current_tail];
        let item = unsafe { (*cell.get()).take() };
        let next_tail = (current_tail + 1) & self.mask;
        self.tail.store(next_tail, Ordering::Release);
        item
    }
}

// Convert MarketMessage to bytes safely
fn message_to_bytes(message: &MarketMessage) -> [u8; 29] {
    unsafe {
        mem::transmute_copy::<[u8; 29], _>(&*(message as *const MarketMessage as *const [u8; 29]))
    }
}

// UDP receiver thread
fn udp_receiver_thread(
    queue: Arc<LockFreeRingBuffer<MarketMessage>>,
    port: u16,
    message_count: Arc<AtomicUsize>,
) {
    // Pin network thread to CPU core 0
    #[cfg(target_os = "linux")]
    pin_thread_to_core(0);

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();
    let mut recv_buf = [0u8; 2048];

    println!("UDP receiver listening on port {}", port);

    loop {
        match socket.recv_from(&mut recv_buf) {
            Ok((len, _addr)) => {
                if len == mem::size_of::<MarketMessage>() {
                    // Parse bytes to MarketMessage safely
                    let recv_slice = &recv_buf[..mem::size_of::<MarketMessage>()];
                    let message = unsafe {
                        mem::transmute_copy::<[u8; 29], _>(
                            &*(recv_slice.as_ptr() as *const [u8; 29]),
                        )
                    };

                    // Push to lock-free queue
                    if let Err(_) = queue.send(message) {
                        eprintln!("Queue full - dropping message");
                    } else {
                        // Count Messages
                        message_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            Err(e) => eprintln!("UDP receive error: {}", e),
        }
    }
}

// UDP sender thread for load testing
fn udp_sender_thread(messages_to_send: usize) {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let target_addr: SocketAddr = "127.0.0.1:9001".parse().unwrap();

    // Test message
    let test_message = MarketMessage::new("AAPL", 150.25, 100);
    let message_bytes = message_to_bytes(&test_message);

    println!("Sending {} messages to port 9001", messages_to_send);

    for i in 0..messages_to_send {
        match socket.send_to(&message_bytes, target_addr) {
            Ok(_) => {
                if i % 1000 == 0 {
                    println!("Sent {} messages", i + 1);
                }
            }
            Err(e) => eprintln!("Send error: {}", e),
        }

        // Small delay between sends
        thread::sleep(Duration::from_micros(50));
    }

    println!("UDP sender finished");
}

fn main() {
    println!("HFT System - First Real System");
    println!("================================");

    // Create shared structures
    let queue = Arc::new(LockFreeRingBuffer::<MarketMessage>::new(16384));
    let message_count = Arc::new(AtomicUsize::new(0));

    // Start UDP receiver thread (pinned to core 0)
    let queue_clone = Arc::clone(&queue);
    let count_clone = Arc::clone(&message_count);
    let receiver_handle = thread::spawn(move || {
        udp_receiver_thread(queue_clone, 9001, count_clone);
    });

    // Start UDP sender thread for load testing (pinned to core 1)
    let sender_handle = thread::spawn(move || {
        udp_sender_thread(10000); // Send 10k messages
    });

    println!("System started:");
    println!("  - UDP receiver on port 9001 (CPU core 0)");
    println!("  - UDP sender to port 9001 (CPU core 1)");
    println!("  - Lock-free queue (1024 capacity)");
    println!();

    // Run for 10 seconds
    thread::sleep(Duration::from_secs(10));

    let final_count = message_count.load(Ordering::Relaxed);
    println!("=== Performance Metrics ===");
    println!("Messages received: {}", final_count);
    println!("Messages/sec: {:.2}", final_count as f64 / 10.0);
    println!(
        "Queue efficiency: {:.2}%",
        (final_count as f64 / 10000.0) * 100.0
    );

    // Keep system running
    println!("System running... Press Ctrl+C to stop");

    // Join threads
    receiver_handle.join().unwrap();
    sender_handle.join().unwrap();
}
