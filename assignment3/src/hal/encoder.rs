/**
 * Hardware interface for a quadrature encoder.
 */
/// Encoder direction pulse
enum Pulse {
    Cw,
    Ccw,
}

impl Pulse {
    fn delta(self) -> i32 {
        match self {
            Self::Cw => -1,
            Self::Ccw => 1,
        }
    }
}

/// Quadrature encoder that is polled on a seperate thread.
pub struct Encoder {
    offset: i32,
    limit_min: i32,
    limit_max: i32,

    pins: gpiod::Lines<gpiod::Input>,

    last_state: Option<(bool, bool)>,
}

impl Encoder {
    /// Create an encoder that is polled on a seperate thread. It has a limited range of allowed values.
    pub fn new(
        limit_min: i32,
        limit_max: i32,
        initial: i32,
        pins: gpiod::Lines<gpiod::Input>,
    ) -> std::io::Result<Self> {
        Ok(Self {
            offset: initial,
            limit_max,
            limit_min,
            pins,
            last_state: None,
        })
    }

    pub fn update(&mut self) {
        let [a, b] = self.pins.get_values([false, false]).unwrap();

        let Some(last_state) = self.last_state else {
            self.last_state = Some((a, b));
            return;
        };

        let state = (a, b);
        if let Some(pulse) = match (last_state, state) {
            ((false, false), (true, false))
            | ((true, false), (true, true))
            | ((true, true), (false, true))
            | ((false, true), (false, false)) => Some(Pulse::Cw),
            ((false, false), (false, true))
            | ((false, true), (true, true))
            | ((true, true), (true, false))
            | ((true, false), (false, false)) => Some(Pulse::Ccw),
            _ => None,
        } {
            self.last_state = Some(state);
            self.offset = self
                .offset
                .saturating_add(pulse.delta())
                .clamp(self.limit_min, self.limit_max);
        }
    }

    /// Get the current position of the encoder.
    pub fn get_offset(&mut self) -> i32 {
        self.offset
    }
}
