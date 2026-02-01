//! High-Performance Trading Primitives
//!
//! A collection of lock-free data structures and low-latency utilities
//! optimized for high-frequency trading systems.
//!
//! # Features
//! - Lock-free SPSC ring buffer
//! - Atomic counters with relaxed ordering
//! - CPU pinning utilities (Linux)
//! - Performance metrics collection

pub mod atomic_counter;
pub mod cpu_pinning;
pub mod metrics;
pub mod ring_buffer;

pub use atomic_counter::AtomicCounter;
pub use cpu_pinning::pin_thread_to_core;
pub use metrics::LatencyMetrics;
pub use ring_buffer::LockFreeRingBuffer;
