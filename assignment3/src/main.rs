use std::time::Instant;

use crate::{
    hal::{encoder::Encoder, mcp320x::MCP320X},
    input::{accelerometer::Accelerometer, drumkit::Drumkit, joystick::Joystick},
    sound::score::Score,
};
use hal::mcp320x::Channel as C;

pub mod hal;
pub mod input;
pub mod sound;

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

    let joystick = Joystick::new(C::CH0, C::CH1);
    let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 3.3);
    let drumkit = Drumkit::new(acc, [1.0, 1.0, 2.0]);

    for channel in [C::CH0, C::CH1] {
        println!(
            "ADC value {channel}: {}V",
            adc.get_voltage_median(channel, 10)?
        );
    }

    println!("Encoder offset: {}", encoder.get_offset());

    println!("joystick reading: {:?}", joystick.get(&mut adc));
    println!("drumkit events: {:?}", drumkit.get(&mut adc));

    let mut score = Score::standard();
    let bpm = 100.0;

    loop {
        let now = Instant::now();
        let notes = score.update(bpm, now);
    }

    Ok(())
}
