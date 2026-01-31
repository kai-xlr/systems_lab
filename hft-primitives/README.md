# HFT Primitives

High-performance trading primitives for building low-latency systems in Rust.

## Features

- **Lock-Free SPSC Ring Buffer**: Single Producer Single Consumer queue with atomic synchronization
- **Atomic Counter**: High-throughput counter with relaxed memory ordering (13x faster than Mutex)
- **CPU Pinning**: Thread affinity utilities for predictable latency (Linux)
- **Latency Metrics**: P50/P95/P99/P999 percentile analysis with consistency ratios

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hft-primitives = { path = "../hft-primitives" }
```

Or use workspace dependencies:

```toml
[dependencies]
hft-primitives = { workspace = true }
```

## Usage Examples

### Lock-Free Ring Buffer

```rust
use hft_primitives::LockFreeRingBuffer;
use std::sync::Arc;
use std::thread;

// Create a queue with capacity 1024 (rounded to next power of 2)
let queue = Arc::new(LockFreeRingBuffer::new(1024));

// Producer thread
let queue_producer = Arc::clone(&queue);
let producer = thread::spawn(move || {
    for i in 0..10000 {
        while queue_producer.send(i).is_err() {
            // Spin if full
        }
    }
});

// Consumer thread
let queue_consumer = Arc::clone(&queue);
let consumer = thread::spawn(move || {
    let mut received = 0;
    while received < 10000 {
        if let Some(value) = queue_consumer.receive() {
            println!("Received: {}", value);
            received += 1;
        }
    }
});

producer.join().unwrap();
consumer.join().unwrap();
```

### Atomic Counter

```rust
use hft_primitives::AtomicCounter;
use std::sync::Arc;
use std::thread;

let counter = Arc::new(AtomicCounter::new());
let mut handles = vec![];

// Spawn 8 threads incrementing counter
for _ in 0..8 {
    let counter_clone = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        for _ in 0..100000 {
            counter_clone.increment();
        }
    }));
}

for handle in handles {
    handle.join().unwrap();
}

println!("Total: {}", counter.get()); // 800,000
```

### CPU Pinning

```rust
use hft_primitives::pin_thread_to_core;
use std::thread;

let handle = thread::spawn(|| {
    // Pin this thread to CPU core 0
    pin_thread_to_core(0);
    
    // Do latency-sensitive work here
    // Thread will stay on core 0, improving cache locality
});

handle.join().unwrap();
```

### Latency Metrics

```rust
use hft_primitives::LatencyMetrics;
use std::time::{Duration, Instant};

// Collect latency samples
let mut samples = vec![];
for _ in 0..10000 {
    let start = Instant::now();
    // ... do work ...
    samples.push(start.elapsed());
}

// Analyze
let metrics = LatencyMetrics::from_samples(&mut samples);
metrics.print_report("My Operation");

// Check if HFT-grade (P99/P50 < 2.0, P99 < 1µs)
if metrics.is_hft_grade() {
    println!("✅ HFT quality achieved!");
}
```

## Performance Characteristics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Ring Buffer Send | ~5ns | 2.3M ops/sec |
| Ring Buffer Receive | ~5ns | 2.3M ops/sec |
| Atomic Counter | ~1ns | 1.2M ops/sec |
| CPU Pinning | One-time | N/A |

### Benchmarking Results

From our benchmarks (see `notes/` for details):

- **Atomic vs Mutex Counter**: 13.3x faster under contention
- **Lock-free SPSC vs Mutex SPSC**: 3.4x faster
- **CPU Pinning**: 19% P99 improvement, 65% max latency reduction

## API Documentation

Generate and open full API docs:

```bash
cargo doc -p hft-primitives --open
```

## Running Tests

```bash
# Unit tests
cargo test -p hft-primitives

# Benchmarks
cargo bench -p hft-primitives

# All tests including doc tests
cargo test -p hft-primitives --doc
```

## Design Principles

1. **Lock-Free First**: No mutexes or locks in critical paths
2. **Zero-Copy**: Minimize allocations and memory copies
3. **Cache-Aware**: Consider cache-line alignment and false sharing
4. **Predictable Latency**: Optimize for P99, not just average case
5. **Linux-Optimized**: Take advantage of platform-specific features

## HFT Quality Standards

This library aims to meet:

- **P99/P50 ratio < 2.0**: Predictable, consistent latency
- **P99 latency < 1µs**: Sub-microsecond operations
- **Zero allocations**: No runtime allocation in hot paths
- **Lock-free**: All data structures use atomic operations only

## Contributing

This is a learning project. See `EXERCISES.md` for challenges to extend functionality.

## License

MIT

## Related Projects

- `queues/`: Original mutex-based SPSC queue
- `threading/`: Lock-free implementations
- `latency/`: CPU pinning experiments
- `networking/`: UDP system using these primitives
