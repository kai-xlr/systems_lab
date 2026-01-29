use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

// Detect number of CPU cores
fn get_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) // Fallback to 4 cores
}

// Simple work task - baseline measurement
fn do_work(iteration: usize) {
    // Minimal work - just a simple computation
    let mut result = iteration * 2;
    result = result.wrapping_add(1);
    // Prevent compiler optimization
    std::hint::black_box(result);
}

// Worker function for each thread
fn worker_thread(
    worker_id: usize,
    iterations: usize,
    barrier: Arc<Barrier>,
    latencies: Arc<Vec<Duration>>,
) {
    // Wait for all workers to be ready
    barrier.wait();

    // Perform work and measure latencies
    for i in 0..iterations {
        let start = Instant::now();
        do_work(i);
        let latency = start.elapsed();

        // Store latency (using unsafe for mutable shared access)
        unsafe {
            let latency_ptr = latencies.as_ptr().add(worker_id * iterations + i) as *mut Duration;
            *latency_ptr = latency;
        }
    }
}

fn main() {
    let cpu_count = get_cpu_count();
    let iterations_per_thread = 100_000;
    let total_iterations = cpu_count * iterations_per_thread;

    println!("Thread-Per-Core Worker Model");
    println!("CPU Cores: {}", cpu_count);
    println!("Iterations per thread: {}", iterations_per_thread);
    println!("Total iterations: {}", total_iterations);
    println!();

    // Shared data structures
    let barrier = Arc::new(Barrier::new(cpu_count));
    let latencies = Arc::new(vec![Duration::ZERO; total_iterations]);

    // Create worker threads
    let mut handles = vec![];
    for worker_id in 0..cpu_count {
        let barrier_clone = Arc::clone(&barrier);
        let latencies_clone = Arc::clone(&latencies);

        let handle = thread::spawn(move || {
            worker_thread(
                worker_id,
                iterations_per_thread,
                barrier_clone,
                latencies_clone,
            );
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Analyze latencies
    analyze_latencies(&latencies);
}

fn analyze_latencies(latencies: &[Duration]) {
    let mut sorted_latencies = latencies.to_vec();
    sorted_latencies.sort();

    let len = sorted_latencies.len();
    let p50 = sorted_latencies[len / 2];
    let p99 = sorted_latencies[(len as f64 * 0.99) as usize];
    let max = sorted_latencies[len - 1];
    let avg = sorted_latencies.iter().sum::<Duration>() / len as u32;

    println!("=== Baseline Latency Analysis ===");
    println!("Samples: {}", len);
    println!("Average: {:?}", avg);
    println!("P50: {:?}", p50);
    println!("P99: {:?}", p99);
    println!("Max: {:?}", max);
    println!(
        "P99/P50 ratio: {:.2}x",
        p99.as_nanos() as f64 / p50.as_nanos() as f64
    );
}
