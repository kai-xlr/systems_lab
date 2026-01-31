# Rust Systems Lab

A hands-on exploration of high-performance systems programming in Rust, focusing on HFT (High-Frequency Trading) system design principles. This project demonstrates lock-free data structures, low-latency networking, CPU pinning, and zero-copy techniques.

## Project Overview

This repository contains a progressive series of systems programming experiments, each building on concepts from the previous:

1. **Lock-based SPSC Queue** → Understanding contention and allocation bottlenecks
2. **Lock-free Atomics** → Achieving 13x speedup with atomic operations
3. **Latency Profiling** → Quantifying the cost of locks, allocations, and branches
4. **HFT UDP System** → Production-grade market data receiver with 99.89% efficiency

## Projects

### 1. `queues/` - SPSC Queue Analysis
**Package**: `spsc-queue`

Mutex-based Single Producer Single Consumer queue implementation.

**Performance**:
- Single-threaded: ~3.8M ops/sec
- Multi-threaded: ~10M ops/sec
- **Bottleneck**: Mutex contention on every send/receive

**Key Learnings**:
- Dynamic allocation overhead (VecDeque resizing)
- Mutex lock/unlock system call costs
- Cache line bouncing under contention

**Run**:
```bash
cd queues
cargo run --release
```

### 2. `threading/` - Atomic Operations & Lock-Free Design
**Package**: `atomics-bench`

Lock-free ring buffer and atomic counter implementations.

**Performance**:
- Atomic counter: 1.2M ops/sec (13.3x faster than mutex)
- Lock-free SPSC: 2.3M ops/sec (3.4x faster than mutex)

**Key Learnings**:
- Memory ordering (Relaxed, Acquire, Release)
- UnsafeCell for interior mutability
- Power-of-2 sizing for fast modulo operations
- Lock-free = predictable latency

**Run**:
```bash
cd threading
cargo run --release
```

### 3. `latency/` - Latency Profiling & CPU Pinning
**Package**: `latency-lab`

Thread-per-core worker model with CPU pinning and latency measurement.

**Performance Impact** (P99 latency):
- **Baseline**: 189ns
- **One branch**: 157ns (17% better)
- **Vec allocation**: 218ns (15% worse)
- **One lock**: 39.4µs (208x worse) ❌

**CPU Pinning Benefits**:
- P99: 19% improvement
- Max latency: 65% reduction (5.79ms → 2.04ms)
- Consistency: P99/P50 ratio improved (1.47x → 1.19x)

**Key Learnings**:
- Locks are poison for HFT (unpredictable latency)
- Allocations manageable with object pools
- CPU pinning eliminates thread migration
- P99/P50 ratio = consistency metric

**Run**:
```bash
cd latency
cargo run --release
```

### 4. `networking/` - HFT UDP System
**Package**: `hft-system`

Production-grade UDP market data receiver with lock-free message passing.

**Architecture**:
- UDP receiver (CPU core 0) → Lock-free SPSC queue → Metrics
- UDP sender (CPU core 1) → Load testing
- Fixed 29-byte MarketMessage structure
- Zero-copy message parsing

**Performance**:
- **Before optimization**: 10.23% efficiency (queue overflow)
- **After optimization**: 99.89% efficiency
- **Improvement**: 9.8x throughput (998.9 msg/sec)

**Key Learnings**:
- UDP vs TCP for low-latency messaging
- Backpressure management via queue sizing
- Zero-copy via `mem::transmute`
- Thread-per-core architecture
- Empirical performance tuning

**Run**:
```bash
cd networking
cargo run --release
```

## Technical Highlights

### Lock-Free Programming
- **AtomicUsize** with Relaxed/Acquire/Release ordering
- **UnsafeCell** for interior mutability in SPSC queues
- **Power-of-2 ring buffers** for bitwise modulo operations

### Low-Latency Techniques
- **CPU pinning** via `sched_setaffinity()` (Linux)
- **Zero-copy networking** using `mem::transmute`
- **Pre-allocated buffers** to avoid runtime allocations
- **UDP protocol** for minimal overhead

### Performance Engineering
- **P50/P99 latency analysis** for consistency metrics
- **Thread-per-core model** for predictable scheduling
- **Empirical tuning** based on throughput measurements
- **Backpressure management** via queue capacity planning

## HFT Design Rules Learned

1. **Kernel Transition Rule**: Any system call = too slow (locks add 208x latency)
2. **Predictability Rule**: P99/P50 ratio < 2.0x is acceptable
3. **Memory Management Rule**: Pre-allocate everything in critical paths
4. **CPU Affinity Rule**: Pin threads to cores for cache consistency
5. **Backpressure Rule**: Measure first, size queues empirically
6. **Zero-Copy Rule**: Eliminate serialization via packed binary formats
7. **UDP Protocol Rule**: TCP guarantees too expensive for HFT

## Performance Progression

| Project | Throughput | Key Optimization |
|---------|-----------|------------------|
| SPSC Queue | 10M ops/sec | Parallelization |
| Atomics | 1.2M ops/sec | Lock-free counters |
| Lock-free SPSC | 2.3M ops/sec | Atomic ring buffer |
| HFT System | 998.9 msg/sec | Queue sizing + backpressure |

## Documentation

Detailed notes for each project phase are in `notes/`:
- `week1-spsc-analysis.md` - Mutex-based queue bottlenecks
- `week3-4-atomic-concurrency.md` - Lock-free data structures
- `week5-6-latency-awareness.md` - CPU pinning and latency profiling
- `week7-8-hft-system.md` - UDP networking and backpressure

## Requirements

- **Rust**: Edition 2021 (2024 for queues package)
- **Platform**: Linux (for CPU pinning via `libc`)
- **Dependencies**: `libc = "0.2"`

## Build & Run

```bash
# Build all projects
cargo build --release --workspace

# Run individual projects
cd queues && cargo run --release
cd threading && cargo run --release
cd latency && cargo run --release
cd networking && cargo run --release
```

## Skills Demonstrated

✅ Lock-free programming with atomic operations  
✅ Low-latency system design (sub-microsecond)  
✅ UDP networking with zero-copy optimization  
✅ CPU affinity and thread pinning (Linux)  
✅ Performance profiling (P50/P99 analysis)  
✅ Memory management (UnsafeCell, transmute)  
✅ Thread-per-core architecture  
✅ Backpressure and flow control  
✅ Empirical performance tuning  
✅ Production-grade systems engineering

## License

MIT
