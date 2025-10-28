use std::{fmt::Display, time};

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

#[derive(Debug)]
pub struct Sampler {
    total: u128,
    avg: Option<f64>,
    period: time::Duration,
    samples: Vec<Sample>,
}

pub struct JitterInfo {
    max: time::Duration,
    min: time::Duration,
    avg: time::Duration,
    num_samples: usize,
}

impl Display for JitterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let JitterInfo {
            max,
            min,
            avg,
            num_samples,
        } = self;
        let max = max.as_micros() as f64 / 1000.0;
        let min = min.as_micros() as f64 / 1000.0;
        let avg = avg.as_micros() as f64 / 1000.0;
        write!(
            f,
            "Smpl ms[{min:6.3}, {max:6.3}] avg {avg:>6.3}/{num_samples:<4}",
        )
    }
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            total: 0,
            avg: None,
            period: time::Duration::from_secs(1),
            samples: Vec::new(),
        }
    }

    pub fn extend_samples<I: IntoIterator<Item = Sample>>(
        &mut self,
        samples: I,
        now: time::Instant,
    ) {
        self.cull_old_samples(now);

        for sample in samples.into_iter() {
            self.samples.push(sample);
            self.total += 1;

            let avg = self.avg.get_or_insert(sample.voltage);
            *avg = *avg * 0.999 + sample.voltage * 0.001;
        }
    }

    pub fn add_sample(&mut self, sample: Sample, now: time::Instant) {
        self.cull_old_samples(now);
        self.samples.push(sample);
        self.total += 1;

        let avg = self.avg.get_or_insert(sample.voltage);
        *avg = *avg * 0.999 + sample.voltage * 0.001;
    }

    fn cull_old_samples(&mut self, now: time::Instant) {
        self.samples
            .retain(|&Sample { voltage: _, time }| now - time < self.period);
    }

    // Get the number of samples collected during the previous complete second.
    pub fn history_size(&self, now: time::Instant) -> usize {
        self.history(now).count()
    }

    // Get a copy of the samples in the sample history.
    pub fn history(&self, now: time::Instant) -> impl Iterator<Item = f64> {
        self.samples
            .iter()
            .copied()
            .filter_map(move |Sample { voltage, time }| {
                (now - time < self.period).then_some(voltage)
            })
    }

    // Get the total number of light level samples taken so far.
    pub fn get_total_samples(&self) -> u128 {
        self.total
    }

    // Get the running average voltage
    pub fn get_avg(&self) -> Option<f64> {
        self.avg
    }

    // Count the number of falling edges in the history
    pub fn get_dips_count(&self, now: time::Instant) -> usize {
        enum State {
            High,
            Low,
        }
        use State::{High, Low};

        let Some(avg) = self.avg else { return 0 };

        self.history(now)
            .fold((High, 0), |(state, count), voltage| match state {
                High => {
                    if voltage < avg - 0.1 {
                        (Low, count + 1)
                    } else {
                        (High, count)
                    }
                }
                Low => {
                    if voltage > avg - 0.07 {
                        (High, count)
                    } else {
                        (Low, count)
                    }
                }
            })
            .1
    }

    pub fn get_jitter_info(&self, now: time::Instant) -> Option<JitterInfo> {
        self.samples
            .windows(2)
            .filter(|s| now - s[0].time < self.period)
            .map(|s| s[1].time - s[0].time)
            .fold(
                None::<(time::Duration, time::Duration, time::Duration, usize)>,
                |stats, between| {
                    Some(match stats {
                        None => (between, between, between, 2),
                        Some((min, max, total, count)) => (
                            min.min(between),
                            max.max(between),
                            total + between,
                            count + 1,
                        ),
                    })
                },
            )
            .map(|(min, max, total, num_samples)| JitterInfo {
                max,
                min,
                avg: total / (num_samples as u32),
                num_samples,
            })
    }
}
