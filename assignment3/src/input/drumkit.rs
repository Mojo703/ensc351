use std::time::{Duration, Instant};

use crate::{hal::mcp320x::MCP320X, input::accelerometer::Accelerometer};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    A,
    B,
    C,
}

impl Event {
    const ALL: [Event; 3] = [Self::A, Self::B, Self::C];
}

pub struct Drumkit {
    acc: Accelerometer,

    thresholds: [f64; 3],
    prev: [Option<Instant>; 3],

    timeout: Duration,
}

impl Drumkit {
    pub fn new(acc: Accelerometer, thresholds: [f64; 3], timeout: Duration) -> Self {
        Self {
            acc,
            thresholds,
            prev: [None, None, None],
            timeout,
        }
    }

    pub fn get(&mut self, adc: &mut MCP320X, now: Instant) -> Vec<Event> {
        match self.acc.get(adc) {
            Some(vals) => (0..3)
                .filter_map(|i| {
                    let val = vals[i];
                    let threshold = self.thresholds[i];
                    let prev = self.prev[i];
                    let event = Event::ALL[i];

                    if prev.is_some_and(|prev| now - prev < self.timeout) {
                        return None;
                    }

                    if val > threshold {
                        self.prev[i] = Some(now);
                        Some(event)
                    } else {
                        None
                    }
                })
                .collect(),
            None => Vec::new(),
        }
    }
}
