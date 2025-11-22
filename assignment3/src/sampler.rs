/**
 * Tool for collecting samples over time, and calculating metrics.
 */
use std::{
    fmt::Display,
    time::{self, Instant},
};

/// Container for samples
#[derive(Debug)]
pub struct Sampler {
    total: u128,
    period: time::Duration,
    samples: Vec<Instant>,
}

/// Metrics for sampling jitter
pub struct JitterInfo {
    max: time::Duration,
    min: time::Duration,
    avg: time::Duration,
    num_samples: usize,
}

impl Display for JitterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = self.max.as_micros() as f64 / 1000.0;
        let min = self.min.as_micros() as f64 / 1000.0;
        let avg = self.avg.as_micros() as f64 / 1000.0;
        let num_samples = self.num_samples;
        write!(
            f,
            "ms[{min:6.3}, {max:6.3}] avg {avg:>6.3}/{num_samples:<4}",
        )
    }
}

impl Sampler {
    /// Create a basic sampler with history period of 1.0 second.
    pub fn new() -> Self {
        Self {
            total: 0,
            period: time::Duration::from_secs(1),
            samples: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, now: time::Instant) {
        self.cull_old_samples(now);

        self.samples.push(now);
        self.total += 1;
    }

    /// Remove samples older than the period
    fn cull_old_samples(&mut self, now: time::Instant) {
        let cutoff = now - self.period;
        let idx = self.samples.partition_point(|&s| s < cutoff);
        self.samples.drain(..idx);
    }

    /// Get the total number of light level samples taken so far.
    pub fn get_total_samples(&self) -> u128 {
        self.total
    }

    /// Calculate the jitter information (statistics on the time between samples).
    pub fn get_jitter_info(&self, now: time::Instant) -> Option<JitterInfo> {
        HistoryJitterStats::try_from_iter(
            self.samples
                .windows(2)
                .filter(|s| now - s[0] < self.period)
                .map(|s| s[1] - s[0]),
        )
        .map(|h| h.into())
    }
}

/// Sampler history statistics
struct HistoryJitterStats {
    min: time::Duration,
    max: time::Duration,
    total: time::Duration,
    count: usize,
}

impl HistoryJitterStats {
    /// Create the initial history statistics from one data point
    fn new(delta: time::Duration) -> Self {
        Self {
            min: delta,
            max: delta,
            total: delta,
            count: 1,
        }
    }

    /// Create the statistics from an iterator
    fn try_from_iter<T: IntoIterator<Item = time::Duration>>(iter: T) -> Option<Self> {
        let mut iter = iter.into_iter();
        let stats = HistoryJitterStats::new(iter.next()?);
        Some(iter.fold(stats, |prev, delta| prev.update(delta)))
    }

    /// Update the statistics to include a new data point
    fn update(self, delta: time::Duration) -> Self {
        let min = self.min.min(delta);
        let max = self.max.max(delta);
        let total = self.total + delta;
        let count = self.count + 1;
        Self {
            min,
            max,
            total,
            count,
        }
    }
}

impl From<HistoryJitterStats> for JitterInfo {
    fn from(value: HistoryJitterStats) -> Self {
        let HistoryJitterStats {
            min,
            max,
            total,
            count,
        } = value;
        JitterInfo {
            min,
            max,
            avg: total / (count as u32),
            num_samples: count,
        }
    }
}
