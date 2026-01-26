# Week 1-2: SPSC Queue Analysis - Mutex<VecDeque<T>> Implementation

## Performance Results

### Single-threaded Performance
- Send: 274ms for 1M items (~3.6M ops/sec)
- Receive: 259ms for 1M items (~3.8M ops/sec)
- Total: 533ms for 1M operations

### Multi-threaded Performance (100K items)
- Producer: 138ms for 100K items
- Consumer: 138ms for 100K items  
- Total: 139ms for both threads
- ~10M ops/sec (faster due to parallelization)

## Key Findings

### 1. Allocation Patterns
- **VecDeque growth**: Dynamic resizing causes memory allocations
- **Memory fragmentation**: As queue grows/shrinks, memory usage becomes unpredictable
- **Cache inefficiency**: Dynamic allocation hurts cache locality

### 2. Contention Sources
- **Mutex overhead**: Each send/require requires lock/unlock
- **Lock duration**: Brief but unavoidable contention point
- **Cache line bouncing**: Mutex and queue data compete for cache lines

### 3. Ownership & Design Insights
- **Mutex takes ownership**: Enforces safe shared access
- **RAII pattern**: MutexGuard automatically unlocks on drop
- **Single mutable access**: Prevents data races at compile time

## Bottlenecks Identified

1. **Lock contention** under high frequency access
2. **Dynamic memory allocation** as queue grows
3. **Cache misses** from pointer chasing
4. **System call overhead** from mutex operations

## Performance vs. HFT Requirements

Current performance (~10M ops/sec) is good for general systems programming
but insufficient for HFT (typically need 100M+ ops/sec with nanosecond latency).

## Next: Refactoring Strategy

To achieve HFT-level performance, we need:
1. **Lock-free algorithms** using atomic operations
2. **Pre-allocated ring buffers** to eliminate dynamic allocation  
3. **Memory ordering optimization** for cache efficiency
4. **Wait-free operation** for predictable latency

## Resume Points

- **Systems Programming**: Built thread-safe SPSC queue with mutex synchronization
- **Performance Analysis**: Measured throughput and identified bottlenecks
- **Multi-threading**: Implemented concurrent producer/consumer pattern
- **Memory Management**: Analyzed allocation patterns and contention sources
- **Rust Ownership**: Applied ownership rules for thread-safe data structures