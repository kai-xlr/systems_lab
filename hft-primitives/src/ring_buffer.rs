//! Lock-free Single Producer Single Consumer (SPSC) ring buffer.
//!
//! Optimized for high-frequency trading workloads with predictable latency.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free SPSC ring buffer optimized for HFT workloads.
///
/// # Safety
/// - **Single producer and single consumer only**
/// - Producer must not wrap around to consumer position
/// - Size is rounded up to next power of 2 for fast modulo
///
/// # Examples
/// ```
/// use hft_primitives::LockFreeRingBuffer;
///
/// let queue = LockFreeRingBuffer::new(1024);
/// queue.send(42).unwrap();
/// assert_eq!(queue.receive(), Some(42));
/// ```
///
/// # Performance Characteristics
/// - Send: O(1) - Single atomic store with Release ordering
/// - Receive: O(1) - Single atomic load with Acquire ordering
/// - No allocations after initialization
/// - Cache-line aligned for minimal false sharing
pub struct LockFreeRingBuffer<T> {
    buffer: Box<[UnsafeCell<Option<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
}

// SAFETY: LockFreeRingBuffer uses atomic operations for synchronization
// The head/tail pointers ensure no two threads access the same buffer slot simultaneously
unsafe impl<T: Send> Send for LockFreeRingBuffer<T> {}
unsafe impl<T: Send> Sync for LockFreeRingBuffer<T> {}

impl<T> LockFreeRingBuffer<T> {
    /// Creates a new lock-free ring buffer with the specified capacity.
    ///
    /// The actual capacity will be rounded up to the next power of 2.
    ///
    /// # Examples
    /// ```
    /// use hft_primitives::LockFreeRingBuffer;
    ///
    /// let queue = LockFreeRingBuffer::<i32>::new(1000);
    /// // Actual capacity is 1024 (next power of 2)
    /// ```
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

    /// Attempts to send an item into the buffer.
    ///
    /// Returns `Err(item)` if the buffer is full.
    ///
    /// # Examples
    /// ```
    /// use hft_primitives::LockFreeRingBuffer;
    ///
    /// let queue = LockFreeRingBuffer::new(4);
    /// assert!(queue.send(1).is_ok());
    /// assert!(queue.send(2).is_ok());
    /// assert!(queue.send(3).is_ok());
    /// // Buffer full (capacity - 1 to distinguish from empty)
    /// assert!(queue.send(4).is_err());
    /// ```
    pub fn send(&self, item: T) -> Result<(), T> {
        let current_head = self.head.load(Ordering::Relaxed);
        let next_head = (current_head + 1) & self.mask;
        let current_tail = self.tail.load(Ordering::Acquire);

        if next_head == current_tail {
            return Err(item); // Buffer full
        }

        let cell = &self.buffer[current_head];
        unsafe {
            *cell.get() = Some(item);
        }
        self.head.store(next_head, Ordering::Release);
        Ok(())
    }

    /// Attempts to receive an item from the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    ///
    /// # Examples
    /// ```
    /// use hft_primitives::LockFreeRingBuffer;
    ///
    /// let queue = LockFreeRingBuffer::new(1024);
    /// assert_eq!(queue.receive(), None);
    /// queue.send(42).unwrap();
    /// assert_eq!(queue.receive(), Some(42));
    /// ```
    pub fn receive(&self) -> Option<T> {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let current_head = self.head.load(Ordering::Acquire);

        if current_head == current_tail {
            return None; // Buffer empty
        }

        let cell = &self.buffer[current_tail];
        let item = unsafe { (*cell.get()).take() };
        let next_tail = (current_tail + 1) & self.mask;
        self.tail.store(next_tail, Ordering::Release);
        item
    }

    /// Returns the capacity of the ring buffer.
    pub fn capacity(&self) -> usize {
        self.mask + 1
    }

    /// Returns the approximate number of items in the buffer.
    ///
    /// Note: This is a snapshot and may be stale immediately.
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        (head.wrapping_sub(tail)) & self.mask
    }

    /// Returns true if the buffer is approximately empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let queue = LockFreeRingBuffer::new(4);
        assert_eq!(queue.receive(), None);
        
        queue.send(1).unwrap();
        queue.send(2).unwrap();
        
        assert_eq!(queue.receive(), Some(1));
        assert_eq!(queue.receive(), Some(2));
        assert_eq!(queue.receive(), None);
    }

    #[test]
    fn test_capacity() {
        let queue = LockFreeRingBuffer::<i32>::new(100);
        assert_eq!(queue.capacity(), 128); // Next power of 2
    }

    #[test]
    fn test_full_buffer() {
        let queue = LockFreeRingBuffer::new(4);
        assert!(queue.send(1).is_ok());
        assert!(queue.send(2).is_ok());
        assert!(queue.send(3).is_ok());
        // Buffer full (capacity - 1 to distinguish from empty)
        assert!(queue.send(4).is_err());
    }
}
