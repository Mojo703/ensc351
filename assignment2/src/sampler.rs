/**
 * Tool for collecting samples over time, and calculating metrics.
 */
use std::{fmt::Display, time};

/// A single sensor sample in time
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    voltage: f64,
    time: time::Instant,
}

impl Sample {
    pub fn new(voltage: f64, now: time::Instant) -> Self {
        Self { voltage, time: now }
    }
}

/// Container for samples
#[derive(Debug)]
pub struct Sampler {
    total: u128,
    avg: Option<f64>,
    period: time::Duration,
    samples: Vec<Sample>,
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
            "Smpl ms[{min:6.3}, {max:6.3}] avg {avg:>6.3}/{num_samples:<4}",
        )
    }
}

impl Sampler {
    /// Create a basic sampler with history period of 1.0 second.
    pub fn new() -> Self {
        Self {
            total: 0,
            avg: None,
            period: time::Duration::from_secs(1),
            samples: Vec::new(),
        }
    }

    /// Add multiple samples to the history
    pub fn extend_samples<I: IntoIterator<Item = Sample>>(
        &mut self,
        samples: I,
        now: time::Instant,
    ) {
        self.cull_old_samples(now);

        for sample in samples.into_iter() {
            assert!(
                self.samples
                    .last()
                    .is_none_or(|prev| prev.time <= sample.time),
                "Samples must be inserted chronological."
            );
            self.samples.push(sample);
            self.total += 1;

            let avg = self.avg.get_or_insert(sample.voltage);
            *avg = *avg * 0.999 + sample.voltage * 0.001;
        }
    }

    /// Remove samples older than the period
    fn cull_old_samples(&mut self, now: time::Instant) {
        let cutoff = now - self.period;
        let idx = self.samples.partition_point(|s| s.time < cutoff);
        self.samples.drain(..idx);
    }

    /// Get the number of samples collected during the previous complete second.
    pub fn history_size(&self, now: time::Instant) -> usize {
        self.history(now).count()
    }

    /// Get a copy of the samples in the sample history.
    pub fn history(&self, now: time::Instant) -> impl Iterator<Item = f64> {
        self.samples
            .iter()
            .copied()
            .filter_map(move |Sample { voltage, time }| {
                (now - time < self.period).then_some(voltage)
            })
    }

    /// Get the total number of light level samples taken so far.
    pub fn get_total_samples(&self) -> u128 {
        self.total
    }

    /// Get the running average voltage
    pub fn get_avg(&self) -> Option<f64> {
        self.avg
    }

    /// Count the number of falling edges in the history
    pub fn get_dips_count(&self, now: time::Instant) -> usize {
        enum State {
            High,
            Low,
        }
        use State::{High, Low};

        let Some(avg) = self.avg else { return 0 };

        self.history(now)
            .fold((High, 0), |(state, count), voltage| {
                if matches!(state, High) && voltage < avg - 0.1 {
                    (Low, count + 1)
                } else if matches!(state, Low) && voltage > avg - 0.07 {
                    (High, count)
                } else {
                    (state, count)
                }
            })
            .1
    }

    /// Calculate the jitter information (statistics on the time between samples).
    pub fn get_jitter_info(&self, now: time::Instant) -> Option<JitterInfo> {
        HistoryJitterStats::try_from_iter(
            self.samples
                .windows(2)
                .filter(|s| now - s[0].time < self.period)
                .map(|s| s[1].time - s[0].time),
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
