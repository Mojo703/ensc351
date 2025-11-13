use crate::hal::mcp320x::{Channel, MCP320X};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Center,
    Up,
    Down,
    Left,
    Right,
}

impl State {
    fn new(x: f64, y: f64) -> Self {
        let dx = (0.5 - x).abs();
        let dy = (0.5 - y).abs();

        let threshold = 0.75;

        if dx < threshold && dy < threshold {
            Self::Center
        } else {
            if dx > dy {
                if x > 0.5 { Self::Left } else { Self::Right }
            } else {
                if y > 0.5 { Self::Up } else { Self::Down }
            }
        }
    }
}

pub struct Joystick {
    x_axis: Channel,
    y_axis: Channel,

    sample_count: usize,
}

impl Joystick {
    pub fn new(x_axis: Channel, y_axis: Channel) -> Self {
        Self {
            x_axis,
            y_axis,
            sample_count: 5,
        }
    }

    pub fn get(&self, adc: &mut MCP320X) -> Option<State> {
        match [self.x_axis, self.y_axis]
            .map(|channel| adc.get_voltage_median(channel, self.sample_count).ok())
        {
            [Some(x), Some(y)] => Some(State::new(x, y)),
            _ => None,
        }
    }
}
