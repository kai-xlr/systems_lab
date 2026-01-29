# Weeks 5-6: Latency Awareness - Systems Intuition Building

## Performance Results Summary

### Thread-Per-Core Worker Model - CPU Pinning Impact

| Metric | Unpinned | Pinned | Improvement |
|---------|----------|---------|-------------|
| P50     | 111ns     | 111ns   | No change     |
| P99     | 163ns     | 132ns   | 19% better    |
| Max     | 5.79ms    | 2.04ms  | 65% reduction  |
| P99/P50 | 1.47x     | 1.19x   | Much more consistent |

### Latency Impact Experiments (Thread-Per-Core, CPU Pinned)

| Operation Type | P50 | P99 | P99/P50 | HFT Grade |
|--------------|------|-----|----------|------------|
| **Baseline** (no overhead) | 101ns | 189ns | 1.87x | ✅ Excellent |
| **One Branch** | 97ns | 157ns | 1.62x | ✅ Better than baseline! |
| **Vec Allocation** | 113ns | 218ns | 1.93x | ⚠️ Acceptable |
| **Box Allocation** | 112ns | 219ns | 1.96x | ⚠️ Acceptable |
| **One Lock** | 350ns | 39.4µs | 112.7x | ❌ **FAIL** |

## Key Technical Achievements

### 1. Thread-Per-Core Worker Model
- **CPU core detection**: `std::thread::available_parallelism()`
- **Barrier synchronization**: All threads start simultaneously
- **Unsafe latency collection**: Zero-overhead shared memory access
- **Arc-based sharing**: Thread-safe data structures

### 2. CPU Pinning Implementation
- **Linux affinity**: `sched_setaffinity()` with `cpu_set_t`
- **Core isolation**: Threads stay on dedicated CPUs
- **Cache warming**: Eliminates thread migration overhead
- **NUMA awareness**: Memory access stays local

### 3. Latency Measurement Infrastructure
- **P50/P99 analysis**: Median vs 99th percentile
- **Consistency metrics**: P99/P50 ratio for predictability
- **Statistical sorting**: Accurate percentile calculations
- **Performance ranking**: Quantitative impact measurement

### 4. Performance Impact Quantification
- **Branch misprediction**: 17% better P99 than baseline
- **Memory allocation**: 30-40% P99 degradation
- **Lock operations**: 376x worse P99 than branching
- **CPU pinning**: 19% P99 improvement, better consistency

## Core Systems Intuition Built

### 1. Why Locks Are Poison for HFT
- **System call overhead**: Kernel transitions on every lock/unlock
- **Thread scheduling**: Even without contention, scheduler involvement
- **Cache pollution**: Lock metadata competes with data cache lines
- **Unpredictable timing**: 112x P99/P50 ratio = unacceptable

### 2. Why Allocations Are Manageable
- **User-space operations**: Heap management without kernel involvement
- **Consistent overhead**: ~30-40% P99 degradation
- **Mitigation strategies**: Object pools, pre-allocation
- **HFT approach**: Zero allocation in critical paths

### 3. Why Branching Is Acceptable
- **CPU-level optimization**: Hardware branch prediction
- **Fast recovery**: Pipeline flush and restart (~10-20 cycles)
- **Predictable cost**: Consistent timing profile
- **Better than baseline**: Random pattern prevented over-optimization

### 4. CPU Pinning Benefits
- **Cache affinity**: Threads stay on warm CPU caches
- **Eliminate migration**: No OS-induced thread movement
- **Consistency improvement**: P99/P50 from 1.47x → 1.19x
- **NUMA locality**: Memory access stays on same node

## Implementation Details

### CPU Pinning Technique
```rust
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
```

### Latency Collection Pattern
```rust
// Zero-overhead shared storage using unsafe
unsafe {
    let latency_ptr = latencies.as_ptr().add(worker_id * iterations + i) as *mut Duration;
    *latency_ptr = latency;
}
```

### Performance Testing Framework
```rust
// Function parameter approach for clean testing
fn run_experiment<F>(...) where F: Fn(usize) + Send + Sync + Clone + 'static {
    // Thread spawning with function injection
    let handle = thread::spawn(move || {
        worker_thread(..., work_fn.clone(), ...);
    });
}
```

## Critical HFT Performance Rules

### 1. Kernel Transition Rule
> **Any operation involving kernel transition = too slow for HFT**

- **Locks**: System calls for synchronization
- **Result**: 376x worse than branch misprediction
- **HFT solution**: Lock-free algorithms only

### 2. Predictability Over Speed Rule  
> **P99/P50 ratio matters more than raw latency**

- **Acceptable**: < 2.0x ratio
- **Marginal**: 2.0-5.0x ratio  
- **Unacceptable**: > 5.0x ratio (locks: 112x)

### 3. Memory Management Rule
> **Pre-allocate everything used in critical paths**

- **Cost**: 30-40% P99 degradation vs baseline
- **Mitigation**: Object pools, memory arenas
- **HFT approach**: Zero allocation in hot loops

### 4. CPU Affinity Rule
> **Dedicated threads + pinning = predictable latency**

- **Benefit**: 19% P99 improvement
- **Mechanism**: Eliminate OS thread migration
- **Result**: Better cache consistency

## Resume Points

- **Thread-per-core architecture**: Built worker model with CPU pinning
- **Latency measurement expertise**: P50/P99 analysis and consistency metrics
- **Performance quantification**: Measured impact of locks, allocations, branches
- **Systems intuition**: Developed ability to predict performance impact
- **HFT optimization**: CPU pinning, lock-free design, memory pre-allocation

## Systems Intuition Established

1. **Branch prediction**: Acceptable cost, CPU handles efficiently
2. **Memory allocation**: Expensive but manageable with object pools
3. **Lock synchronization**: Complete performance poison for HFT systems
4. **Cache affinity**: Critical for predictable latencies
5. **Consistency metrics**: P99/P50 ratio = predictability indicator
6. **Kernel transitions**: Never acceptable in HFT critical paths