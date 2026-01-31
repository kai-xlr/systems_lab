# Quick Start Guide

Get started with the Rust Systems Lab in 5 minutes.

## Setup

```bash
# Clone and navigate
cd rust-systems-lab

# Build everything
cargo build --workspace --release

# Run tests
cargo test --workspace
```

## Run Projects

### 1. SPSC Queue (Mutex-based)
```bash
cd queues
cargo run --release
```
**What you'll see**: Performance of mutex-based queue (~10M ops/sec)

### 2. Lock-Free Atomics
```bash
cd threading
cargo run --release
```
**What you'll see**: 13.3x speedup with atomics vs mutex

### 3. Latency Profiling
```bash
cd latency
cargo run --release
```
**What you'll see**: Impact of locks, allocations, branches on P99 latency

### 4. HFT UDP System
```bash
cd networking
cargo run --release
```
**What you'll see**: 99.89% efficiency UDP market data receiver

## Run Benchmarks

```bash
# Benchmark shared library primitives
cargo bench -p hft-primitives

# Results saved to: target/criterion/
```

## View Documentation

```bash
# Generate and open API docs
cargo doc --workspace --open

# Read learning exercises
cat EXERCISES.md
```

## Quick Commands Reference

```bash
# Build all in release mode
cargo build --workspace --release

# Run specific project
cargo run -p spsc-queue --release
cargo run -p atomics-bench --release
cargo run -p latency-lab --release
cargo run -p hft-system --release

# Test specific project
cargo test -p hft-primitives

# Benchmark specific component
cargo bench -p hft-primitives --bench ring_buffer
cargo bench -p hft-primitives --bench atomic_counter

# Check code without building
cargo check --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace
```

## Project Structure

```
rust-systems-lab/
├── Cargo.toml              # Workspace configuration
├── README.md               # Main documentation
├── EXERCISES.md            # Learning challenges
├── QUICKSTART.md           # This file
│
├── hft-primitives/         # Shared library
│   ├── src/
│   │   ├── ring_buffer.rs  # Lock-free SPSC queue
│   │   ├── atomic_counter.rs
│   │   ├── cpu_pinning.rs
│   │   └── metrics.rs
│   └── benches/            # Criterion benchmarks
│
├── queues/                 # Week 1-2: Mutex SPSC
├── threading/              # Week 3-4: Lock-free
├── latency/                # Week 5-6: CPU pinning
├── networking/             # Week 7-8: UDP system
│
└── notes/                  # Weekly learning notes
    ├── week1-spsc-analysis.md
    ├── week3-4-atomic-concurrency.md
    ├── week5-6-latency-awareness.md
    └── week7-8-hft-system.md
```

## Learning Path

1. **Start with README.md**: Understand the overall project
2. **Run each project**: See concepts in action
3. **Read notes/**: Deep dive into each week's learnings
4. **Try exercises**: Build on what you've learned
5. **Run benchmarks**: Quantify performance improvements

## Common Issues

### CPU Pinning Not Working
**Symptom**: Warning about CPU pinning
**Solution**: Linux required for `sched_setaffinity`. On macOS/Windows, it's a no-op.

### Benchmarks Taking Long
**Symptom**: `cargo bench` runs for several minutes
**Solution**: This is normal. Criterion runs statistical analysis.

### Tests Fail
**Symptom**: Atomic counter test fails intermittently
**Solution**: Thread timing issue. Re-run tests.

## Next Steps

- **Beginner**: Work through EXERCISES.md Level 1
- **Intermediate**: Add cache-line padding (Exercise 2.2)
- **Advanced**: Build order matching engine (Exercise 3.2)

## Getting Help

- Check the `notes/` directory for detailed explanations
- Review test cases in `src/` files for usage examples
- Read the API docs: `cargo doc --open`
- Try the exercises with hints in `EXERCISES.md`

## Performance Targets

If you've completed the project, you should see:

- ✅ Lock-free SPSC: 2.3M ops/sec
- ✅ Atomic counter: 13x faster than mutex
- ✅ CPU pinning: 19% P99 improvement
- ✅ UDP receiver: 99.89% efficiency

Happy learning! 🦀
