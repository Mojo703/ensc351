use std::time;

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

impl Sampler {
    pub fn new() -> Self {
        Self {
            total: 0,
            avg: None,
            period: time::Duration::from_secs(1),
            samples: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, sample: Sample, now: time::Instant) {
        self.cull_old_samples(now);
        self.samples.push(sample);

        let avg = self.avg.get_or_insert(sample.voltage);
        *avg = *avg * 0.999 + sample.voltage * 0.001;
    }

    fn cull_old_samples(&mut self, now: time::Instant) {
        self.samples
            .retain(|&Sample { voltage: _, time }| now - time < self.period);
    }

    // Get the number of samples collected during the previous complete second.
    pub fn store_size(&self, now: time::Instant) -> usize {
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
                High => (voltage < avg - 0.1)
                    .then_some((Low, count + 1))
                    .unwrap_or((High, count)),
                Low => (voltage > avg - 0.07)
                    .then_some((High, count))
                    .unwrap_or((Low, count)),
            })
            .1
    }
}
