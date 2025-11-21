use crate::hal::mcp320x::{Channel, MCP320X};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Center,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    // x and y in range [0, 1.0]
    fn new(x: f64, y: f64) -> Self {
        const THRESHOLD: f64 = 0.25;

        let dx = (0.5 - x).abs();
        let dy = (0.5 - y).abs();

        if dx < THRESHOLD && dy < THRESHOLD {
            Self::Center
        } else if dx > dy {
            if x > 0.5 { Self::Left } else { Self::Right }
        } else if y > 0.5 {
            Self::Up
        } else {
            Self::Down
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

    pub fn get(&self, adc: &mut MCP320X) -> Option<Direction> {
        match [self.x_axis, self.y_axis].map(|channel| adc.get_median(channel, self.sample_count)) {
            [Ok(x), Ok(y)] => Some(Direction::new(x, y)),
            _ => None,
        }
    }
}
