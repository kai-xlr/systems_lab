use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
// Fixed-size market message (29 bytes total)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MarketMessage {
    message_type: u8, // 1 byte
    symbol: [u8; 8],  // 8 bytes
    price: u64,       // 8 bytes (fixed-point)
    quantity: u32,    // 4 bytes
    timestamp: u64,   // 8 bytes
}
impl MarketMessage {
    fn new(symbol: &str, price: f64, quantity: u32) -> Self {
        let mut symbol_bytes = [0u8; 8];
        let symbol_len = symbol.len().min(8);
        symbol_bytes[..symbol_len].copy_from_slice(symbol.as_bytes());

        // Convert price to fixed-point (4 decimal places)
        let price_fixed = (price * 10000.0) as u64;

        Self {
            message_type: 1,
            symbol: symbol_bytes,
            price: price_fixed,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }

    fn get_price(&self) -> f64 {
        self.price as f64 / 10000.0
    }
}
fn main() {
    println!("HFT System - First Real System");
    println!("================================");

    // Test message creation and parsing
    let message = MarketMessage::new("AAPL", 150.25, 100);
    println!("Created message: {:?}", message);
    println!("Decoded price: ${:.4}", message.get_price());

    // TODO: Add UDP receiver
    // TODO: Add SPSC queue
    // TODO: Add worker threads
    // TODO: Add metrics
}
