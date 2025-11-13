use crate::hal::mcp320x::{Channel, MCP320X};

type Acceleration = f64;

type Measurement = [Acceleration; 3];

pub struct Accelerometer {
    x_axis: Channel,
    y_axis: Channel,
    z_axis: Channel,

    voltage_per_g: f64,

    sample_count: usize,
}

impl Accelerometer {
    pub fn new(x_axis: Channel, y_axis: Channel, z_axis: Channel, voltage_per_g: f64) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
            voltage_per_g,
            sample_count: 5,
        }
    }

    pub fn get(&self, adc: &mut MCP320X) -> Option<Measurement> {
        match self.channels().map(|channel| {
            adc.get_voltage_median(channel, self.sample_count)
                .map(|voltage| voltage / self.voltage_per_g)
                .ok()
        }) {
            [Some(x), Some(y), Some(z)] => Some([x, y, z]),
            _ => None,
        }
    }

    fn channels(&self) -> [Channel; 3] {
        [self.x_axis, self.y_axis, self.z_axis]
    }
}
