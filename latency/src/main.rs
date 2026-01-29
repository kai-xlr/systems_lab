use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
// Detect number of CPU cores
fn get_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) // Fallback to 4 cores
}
// Version 1: Baseline (current)
fn do_work_baseline(iteration: usize) {
    let mut result = iteration * 2;
    result = result.wrapping_add(1);
    // Prevent compiler optimization
    std::hint::black_box(result);
}
// Version 2: Add one allocation
fn do_work_with_allocation(iteration: usize) {
    let mut result = iteration * 2;
    result = result.wrapping_add(1);

    // ADD ONE ALLOCATION HERE
    let _allocated = vec![result]; // <-- This allocation

    std::hint::black_box(result);
}
// Version 3: Add heap allocation
fn do_work_with_box(iteration: usize) {
    let mut result = iteration * 2;
    result = result.wrapping_add(1);

    // ADD BOX ALLOCATION HERE
    let _boxed = Box::new(result); // <-- This heap allocation

    std::hint::black_box(result);
}
// Worker function for each thread
fn worker_thread<F>(
    worker_id: usize,
    iterations: usize,
    barrier: Arc<Barrier>,
    latencies: Arc<Vec<Duration>>,
    pin_to_core: bool,
    work_fn: F,
) where
    F: Fn(usize) + Send + Sync + Clone + 'static,
{
    // Pin thread to specific CPU core if requested
    #[cfg(target_os = "linux")]
    if pin_to_core {
        pin_thread_to_core(worker_id);
    }
    // Wait for all workers to be ready
    barrier.wait();
    // Perform work and measure latencies
    for i in 0..iterations {
        let start = Instant::now();
        work_fn(i);
        let latency = start.elapsed();
        // Store latency (using unsafe for mutable shared access)
        unsafe {
            let latency_ptr = latencies.as_ptr().add(worker_id * iterations + i) as *mut Duration;
            *latency_ptr = latency;
        }
    }
}
// CPU pinning for Linux
#[cfg(target_os = "linux")]
fn pin_thread_to_core(core_id: usize) {
    use libc::{cpu_set_t, sched_setaffinity};
    use std::mem;

    let mut cpu_set: cpu_set_t = unsafe { mem::zeroed() };
    unsafe {
        libc::CPU_SET(core_id, &mut cpu_set);
        sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set);
    }
}
fn run_experiment<F>(
    cpu_count: usize,
    iterations_per_thread: usize,
    pin_to_core: bool,
    work_fn: F,
    test_name: &str,
) where
    F: Fn(usize) + Send + Sync + Clone + 'static,
{
    // Shared data structures
    let barrier = Arc::new(Barrier::new(cpu_count));
    let latencies = Arc::new(vec![Duration::ZERO; cpu_count * iterations_per_thread]);
    // Create worker threads
    let mut handles = vec![];
    for worker_id in 0..cpu_count {
        let barrier_clone = Arc::clone(&barrier);
        let latencies_clone = Arc::clone(&latencies);
        let work_fn_clone = work_fn.clone();
        let handle = thread::spawn(move || {
            worker_thread(
                worker_id,
                iterations_per_thread,
                barrier_clone,
                latencies_clone,
                pin_to_core,
                work_fn_clone,
            );
        });
        handles.push(handle);
    }
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    // Analyze latencies
    println!("{} - Latency Analysis:", test_name);
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
    println!("  Samples: {}", len);
    println!("  Average: {:?}", avg);
    println!("  P50: {:?}", p50);
    println!("  P99: {:?}", p99);
    println!("  Max: {:?}", max);
    println!(
        "  P99/P50 ratio: {:.2}x",
        p99.as_nanos() as f64 / p50.as_nanos() as f64
    );
}
fn main() {
    let cpu_count = get_cpu_count();
    let iterations_per_thread = 100_000;
    let total_iterations = cpu_count * iterations_per_thread;
    println!("Thread-Per-Core Worker Model - Latency Impact Experiments");
    println!("CPU Cores: {}", cpu_count);
    println!("Iterations per thread: {}", iterations_per_thread);
    println!("Total iterations: {}", total_iterations);
    println!();
    // Test 1: Baseline (no allocation)
    println!("=== Test 1: Baseline (No Allocation) ===");
    run_experiment(
        cpu_count,
        iterations_per_thread,
        true,
        do_work_baseline,
        "Baseline",
    );
    println!();
    // Test 2: Add one allocation
    println!("=== Test 2: One Vec Allocation ===");
    run_experiment(
        cpu_count,
        iterations_per_thread,
        true,
        do_work_with_allocation,
        "Vec Allocation",
    );
    println!();
    // Test 3: Add heap allocation
    println!("=== Test 3: One Box Allocation ===");
    run_experiment(
        cpu_count,
        iterations_per_thread,
        true,
        do_work_with_box,
        "Box Allocation",
    );
}
