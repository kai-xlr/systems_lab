# Learning Exercises

Progressive challenges to deepen understanding of HFT systems programming concepts.

## Prerequisites

Before starting these exercises, you should have:
- Completed the 4 main projects (queues, threading, latency, networking)
- Read the weekly notes in `notes/`
- Run the benchmarks: `cargo bench -p hft-primitives`

---

## Level 1: Foundation

### Exercise 1.1: Add MPSC Support
**Difficulty**: ⭐⭐☆☆☆

Modify `LockFreeRingBuffer` to support Multiple Producers, Single Consumer.

**Learning Goals**:
- Understand compare-and-swap (CAS) operations
- Learn why SPSC is simpler than MPSC
- Practice atomic ordering constraints

**Hints**:
- Use `compare_exchange` for the head pointer
- Multiple producers compete for same position
- Keep tail pointer simple (single consumer)

**Validation**:
```rust
#[test]
fn test_mpsc() {
    let queue = Arc::new(LockFreeRingBuffer::new(1024));
    let mut handles = vec![];
    
    // 4 producers
    for i in 0..4 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                queue.send(i * 1000 + j).unwrap();
            }
        }));
    }
    
    // Wait and verify count
    for h in handles { h.join().unwrap(); }
    
    let mut count = 0;
    while queue.receive().is_some() {
        count += 1;
    }
    assert_eq!(count, 4000);
}
```

**Bonus**: Benchmark SPSC vs MPSC throughput.

---

### Exercise 1.2: Implement Batching
**Difficulty**: ⭐⭐☆☆☆

Modify the UDP receiver to process messages in batches.

**Learning Goals**:
- Reduce syscall overhead
- Trade latency for throughput
- Measure batch size impact

**Target**: Reduce syscalls by 10x (receive 10 messages per `recv_from`).

**Hints**:
- Use larger receive buffer (e.g., 2048 bytes)
- Parse multiple messages from single buffer
- Track messages per syscall

**Validation**:
- Measure syscalls before/after: `strace -c ./your_program`
- Compare throughput: messages/sec should increase
- Measure P99 latency: might increase slightly

**Questions to Answer**:
- What's the optimal batch size?
- How does batching affect tail latency?
- When is batching a bad idea?

---

### Exercise 1.3: Add Histogram Metrics
**Difficulty**: ⭐⭐☆☆☆

Extend `LatencyMetrics` to collect latency histograms.

**Learning Goals**:
- Understand latency distribution visualization
- Practice bucketing and percentile calculation
- Learn HDR (High Dynamic Range) histogram concepts

**API Design**:
```rust
pub struct LatencyHistogram {
    buckets: Vec<(Duration, usize)>, // (upper_bound, count)
}

impl LatencyHistogram {
    pub fn new(bucket_size: Duration, max: Duration) -> Self;
    pub fn record(&mut self, latency: Duration);
    pub fn print_ascii_histogram(&self);
}
```

**Validation**:
- Generate test data with known distribution
- Verify bucket counts
- Visual inspection of ASCII histogram

---

## Level 2: Performance

### Exercise 2.1: Zero-Copy Optimization
**Difficulty**: ⭐⭐⭐☆☆

Eliminate all allocations from the UDP receiver hot path.

**Learning Goals**:
- Identify hidden allocations
- Use object pools
- Measure allocation impact

**Current Issues**:
- Error handling might allocate
- String formatting allocates
- Buffer resizing allocates

**Hints**:
- Pre-allocate error buffers
- Use `write!` to pre-allocated strings
- Profile with `valgrind --tool=massif`

**Validation**:
```bash
# Before optimization
valgrind --tool=massif ./hft-system

# After optimization
# Total allocations should be near zero in hot path
```

**Target**: 0 allocations per message in steady state.

---

### Exercise 2.2: Cache-Line Padding
**Difficulty**: ⭐⭐⭐☆☆

Add cache-line padding to eliminate false sharing in `LockFreeRingBuffer`.

**Learning Goals**:
- Understand cache-line bouncing
- Learn `#[repr(align(64))]`
- Measure false sharing impact

**Implementation**:
```rust
#[repr(align(64))]
struct CacheLinePadded<T> {
    value: T,
    _padding: [u8; 64 - std::mem::size_of::<T>()],
}

pub struct LockFreeRingBuffer<T> {
    buffer: Box<[UnsafeCell<Option<T>>]>,
    head: CacheLinePadded<AtomicUsize>,  // <-- Padded
    tail: CacheLinePadded<AtomicUsize>,  // <-- Padded
    mask: usize,
}
```

**Validation**:
- Benchmark before/after padding
- Use `perf stat -e cache-misses` to measure cache behavior
- Expect 5-20% throughput improvement

---

### Exercise 2.3: Custom Allocator
**Difficulty**: ⭐⭐⭐⭐☆

Implement a lock-free object pool for `MarketMessage`.

**Learning Goals**:
- Custom allocator implementation
- Memory reuse patterns
- Benchmark allocation strategies

**API Design**:
```rust
pub struct ObjectPool<T> {
    free_list: LockFreeStack<Box<T>>,
    capacity: usize,
}

impl<T: Default> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self;
    pub fn acquire(&self) -> Option<Box<T>>;
    pub fn release(&self, obj: Box<T>);
}
```

**Validation**:
- Benchmark allocation vs pool
- Measure fragmentation over time
- Test under contention (multiple threads)

**Expected Improvement**: 2-5x faster than `Box::new()`.

---

## Level 3: System Design

### Exercise 3.1: Multicast UDP
**Difficulty**: ⭐⭐⭐☆☆

Add UDP multicast support for market data feeds.

**Learning Goals**:
- Multicast socket programming
- IGMP protocol basics
- Network efficiency

**Requirements**:
- Join multicast group (e.g., 239.0.0.1:9001)
- Receive from multiple publishers
- Handle packet loss gracefully

**Hints**:
```rust
use std::net::Ipv4Addr;

socket.join_multicast_v4(
    &Ipv4Addr::new(239, 0, 0, 1),
    &Ipv4Addr::UNSPECIFIED
)?;
```

**Validation**:
- Test with multiple senders
- Measure throughput vs unicast
- Document packet loss rate

---

### Exercise 3.2: Order Matching Engine
**Difficulty**: ⭐⭐⭐⭐⭐

Build a simple limit order book with lock-free price levels.

**Learning Goals**:
- Financial market microstructure
- Priority queue implementation
- Concurrent data structure design

**Requirements**:
- Orders: Buy/Sell, Price, Quantity, Timestamp
- Matching: Price-time priority
- Operations: Add order, cancel order, match orders
- Target latency: < 10 microseconds per operation

**Data Structures**:
```rust
pub struct OrderBook {
    bids: BTreeMap<Price, VecDeque<Order>>,  // Sorted descending
    asks: BTreeMap<Price, VecDeque<Order>>,  // Sorted ascending
}

pub struct Order {
    id: u64,
    side: Side,
    price: u64,
    quantity: u32,
    timestamp: u64,
}
```

**Validation**:
- Unit tests for matching logic
- Benchmark order add/cancel/match
- Test fairness (price-time priority)

**Bonus**: Make it lock-free using skip lists.

---

### Exercise 3.3: Full HFT Stack Integration
**Difficulty**: ⭐⭐⭐⭐⭐

Combine all components into a complete trading system.

**Architecture**:
```
UDP Multicast Feed → Parse → Order Book → Strategy → Order Gateway
```

**Components**:
1. **Market Data Feed**: UDP multicast receiver
2. **Order Book**: Lock-free limit order book
3. **Strategy**: Simple market maker
4. **Order Gateway**: UDP unicast sender

**Requirements**:
- All components on dedicated CPU cores
- Lock-free queues between components
- < 1 microsecond end-to-end latency
- Handle 1M messages/sec

**Validation**:
- Measure tick-to-trade latency
- Test under market stress
- Profile with `perf` and flamegraphs

**Questions to Answer**:
- What's your limiting factor?
- How does CPU pinning help?
- What happens under backpressure?

---

## Level 4: Advanced Topics

### Exercise 4.1: Kernel Bypass with io_uring
**Difficulty**: ⭐⭐⭐⭐⭐

Replace standard UDP sockets with Linux io_uring.

**Learning Goals**:
- Kernel bypass techniques
- io_uring API
- Zero-copy networking

**Requirements**:
- Use `io_uring` crate
- Implement async receive
- Compare latency vs standard sockets

**Expected Improvement**: 20-40% P99 latency reduction.

---

### Exercise 4.2: DPDK Integration
**Difficulty**: ⭐⭐⭐⭐⭐

Port UDP receiver to DPDK for true kernel bypass.

**Learning Goals**:
- User-space networking
- Packet processing pipelines
- Hardware optimization

**Prerequisites**:
- DPDK installation
- Dedicated NIC
- Root access

**Target**: Sub-microsecond processing latency.

---

### Exercise 4.3: FPGA Offload
**Difficulty**: ⭐⭐⭐⭐⭐

Design FPGA logic for order parsing and matching.

**Learning Goals**:
- Hardware acceleration concepts
- HDL (Verilog/VHDL) basics
- Co-design (software + hardware)

**Out of Scope**: This is a research project, not a coding exercise!

---

## Validation & Testing

### How to Know You've Succeeded

For each exercise:

1. **Correctness**: All tests pass
2. **Performance**: Meets target metrics
3. **Understanding**: Can explain trade-offs
4. **Documentation**: Write notes on what you learned

### Recommended Tools

- **Profiling**: `perf`, `flamegraph`, `valgrind`
- **Benchmarking**: `criterion`, `hyperfine`
- **Tracing**: `strace`, `ltrace`
- **Network**: `tcpdump`, `wireshark`
- **System**: `htop`, `iostat`, `vmstat`

---

## Resources

### Books
- *C++ Concurrency in Action* by Anthony Williams
- *Systems Performance* by Brendan Gregg
- *Trading and Exchanges* by Larry Harris

### Papers
- "A Pragmatic Implementation of Non-Blocking Linked-Lists" (Michael & Scott)
- "Fast and Lock-Free Concurrent Priority Queues for Multi-Thread Systems" (Sundell & Tsigas)
- "SHM: A Shared Memory Lock-Free Queue for Modern Hardware" (Morrison & Afek)

### Online
- Rust Atomics and Locks: https://marabos.nl/atomics/
- 1024cores: http://www.1024cores.net/
- Linux Performance: https://www.brendangregg.com/

---

## Contributing Your Solutions

If you solve these exercises:

1. Create a branch: `git checkout -b exercise-1.1-mpsc`
2. Document your approach in `notes/exercises/`
3. Add benchmarks comparing your solution
4. Include lessons learned

**Remember**: The goal is learning, not perfect code. Focus on understanding the trade-offs and building intuition for HFT systems design.

Good luck! 🚀
