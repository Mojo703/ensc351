pub mod audio;
pub mod hal;
pub mod input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut adc = hal::mcp320x::MCP320X::new("/dev/spidev0.0", 3.3)?;
    let mut encoder = {
        use gpiod::*;
        let chip = Chip::new("gpiochip0")?;

        let pins = Options::input([7, 10])
            .active(Active::High)
            .bias(Bias::PullDown);
        let pins = chip.request_lines(pins)?;

        hal::encoder::Encoder::new(0, 100, 10, pins)
    }?;

    for channel in [hal::mcp320x::Channel::CH0, hal::mcp320x::Channel::CH1] {
        println!(
            "ADC value {channel}: {}V",
            adc.get_median_voltage(channel, 10)?
        );
    }

    println!("Encoder offset: {}", encoder.get_offset());

    Ok(())
}
