use crate::{hal::mcp320x::MCP320X, input::accelerometer::Accelerometer};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    A,
    B,
    C,
}

impl Event {
    fn all() -> [Self; 3] {
        [Self::A, Self::B, Self::C]
    }
}

pub struct Drumkit {
    acc: Accelerometer,

    thresholds: [f64; 3],
}

impl Drumkit {
    pub fn new(acc: Accelerometer, thresholds: [f64; 3]) -> Self {
        Self { acc, thresholds }
    }

    pub fn get(&self, adc: &mut MCP320X) -> Vec<Event> {
        self.acc
            .get(adc)
            .map(|a| a.map(Some))
            .unwrap_or([None, None, None])
            .into_iter()
            .zip(self.thresholds)
            .zip(Event::all())
            .filter_map(|((g, threshold), event)| g.and_then(|g| (g > threshold).then_some(event)))
            .collect()
    }
}
