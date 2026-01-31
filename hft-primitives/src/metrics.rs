//! Performance metrics collection and analysis.
//!
//! Utilities for collecting and analyzing latency measurements.

use std::time::Duration;

/// Latency metrics analyzer for HFT systems.
///
/// Calculates percentiles and consistency metrics from latency samples.
///
/// # Examples
/// ```
/// use hft_primitives::LatencyMetrics;
/// use std::time::Duration;
///
/// let mut samples = vec![
///     Duration::from_nanos(100),
///     Duration::from_nanos(150),
///     Duration::from_nanos(200),
/// ];
///
/// let metrics = LatencyMetrics::from_samples(&mut samples);
/// println!("P50: {:?}", metrics.p50);
/// println!("P99: {:?}", metrics.p99);
/// ```
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub samples: usize,
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p999: Duration,
}

impl LatencyMetrics {
    /// Analyzes latency samples and returns metrics.
    ///
    /// Samples are sorted in-place for percentile calculation.
    pub fn from_samples(samples: &mut [Duration]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        samples.sort();
        let len = samples.len();

        let min = samples[0];
        let max = samples[len - 1];
        let sum: Duration = samples.iter().sum();
        let avg = sum / len as u32;

        let p50 = samples[len / 2];
        let p95 = samples[(len as f64 * 0.95) as usize];
        let p99 = samples[(len as f64 * 0.99) as usize];
        let p999 = samples[((len as f64 * 0.999) as usize).min(len - 1)];

        Self {
            samples: len,
            min,
            max,
            avg,
            p50,
            p95,
            p99,
            p999,
        }
    }

    /// Calculates the P99/P50 ratio as a measure of consistency.
    ///
    /// Values < 2.0 indicate good consistency.
    /// Values > 5.0 indicate high variance (poor for HFT).
    pub fn consistency_ratio(&self) -> f64 {
        self.p99.as_nanos() as f64 / self.p50.as_nanos() as f64
    }

    /// Prints a formatted report of the metrics.
    pub fn print_report(&self, name: &str) {
        println!("=== {} ===", name);
        println!("  Samples: {}", self.samples);
        println!("  Min: {:?}", self.min);
        println!("  Avg: {:?}", self.avg);
        println!("  P50: {:?}", self.p50);
        println!("  P95: {:?}", self.p95);
        println!("  P99: {:?}", self.p99);
        println!("  P999: {:?}", self.p999);
        println!("  Max: {:?}", self.max);
        println!("  P99/P50 ratio: {:.2}x", self.consistency_ratio());
    }

    /// Returns true if metrics meet HFT quality standards.
    ///
    /// Criteria:
    /// - P99/P50 ratio < 2.0 (good consistency)
    /// - P99 < 1 microsecond (low latency)
    pub fn is_hft_grade(&self) -> bool {
        self.consistency_ratio() < 2.0 && self.p99 < Duration::from_micros(1)
    }
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self {
            samples: 0,
            min: Duration::ZERO,
            max: Duration::ZERO,
            avg: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            p999: Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_metrics() {
        let mut samples = vec![
            Duration::from_nanos(100),
            Duration::from_nanos(200),
            Duration::from_nanos(300),
            Duration::from_nanos(400),
            Duration::from_nanos(500),
        ];

        let metrics = LatencyMetrics::from_samples(&mut samples);
        assert_eq!(metrics.samples, 5);
        assert_eq!(metrics.min, Duration::from_nanos(100));
        assert_eq!(metrics.max, Duration::from_nanos(500));
        assert_eq!(metrics.p50, Duration::from_nanos(300));
    }

    #[test]
    fn test_consistency_ratio() {
        let mut samples = vec![
            Duration::from_nanos(100),
            Duration::from_nanos(100),
            Duration::from_nanos(100),
            Duration::from_nanos(200), // P99
        ];

        let metrics = LatencyMetrics::from_samples(&mut samples);
        assert!(metrics.consistency_ratio() < 2.5);
    }

    #[test]
    fn test_empty_samples() {
        let mut samples = vec![];
        let metrics = LatencyMetrics::from_samples(&mut samples);
        assert_eq!(metrics.samples, 0);
        assert_eq!(metrics.p50, Duration::ZERO);
    }
}
