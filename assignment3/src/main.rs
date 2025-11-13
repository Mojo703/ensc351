use crate::{
    hal::{encoder::Encoder, mcp320x::MCP320X},
    input::accelerometer::Accelerometer,
};
use hal::mcp320x::Channel as C;

pub mod audio;
pub mod hal;
pub mod input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut adc = MCP320X::new("/dev/spidev0.0", 3.3)?;
    let mut encoder = {
        use gpiod::*;
        let chip = Chip::new("gpiochip0")?;

        let pins = Options::input([7, 10])
            .active(Active::High)
            .bias(Bias::PullDown);
        let pins = chip.request_lines(pins)?;

        Encoder::new(0, 100, 10, pins)
    }?;

    let acc_meter = Accelerometer::new(C::CH0, C::CH1, C::CH2, 3.3);

    for channel in [C::CH0, C::CH1] {
        println!(
            "ADC value {channel}: {}V",
            adc.get_voltage_median(channel, 10)?
        );
    }

    println!("Encoder offset: {}", encoder.get_offset());

    println!("Accelerometer reading: {:?}", acc_meter.get(&mut adc));

    Ok(())
}
