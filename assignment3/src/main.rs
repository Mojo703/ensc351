use std::{collections::HashMap, time::Instant};

use crate::{
    hal::{encoder::Encoder, mcp320x::MCP320X},
    input::{accelerometer::Accelerometer, drumkit::Drumkit, joystick::Joystick},
    sound::{
        Instrument, load_wav,
        playback::{InstrumentHandle, Playback},
        score::Score,
    },
};
use alsa::{Direction, PCM};
use hal::mcp320x::Channel as C;

pub mod hal;
pub mod input;
pub mod sound;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instruments = [
        (
            Instrument::BassDrum,
            "./sounds/100052__menegass__gui-drum-bd-soft.wav",
        ),
        (
            Instrument::HiHat,
            "./sounds/100063__menegass__gui-drum-tom-hi-soft.wav",
        ),
        (
            Instrument::Snare,
            "./sounds/100059__menegass__gui-drum-snare-soft.wav",
        ),
    ];

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

    let pcm = PCM::new("default", Direction::Playback, false)?;

    let channels = 2;
    let rate = 44100;

    let mut playback = Playback::new(&pcm, channels, rate, channels as usize * 1000)?;
    let joystick = Joystick::new(C::CH0, C::CH1);
    let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 3.3);
    let drumkit = Drumkit::new(acc, [1.0, 1.0, 2.0]);

    let sound_handles: HashMap<Instrument, InstrumentHandle> = instruments
        .into_iter()
        .map(|(instrument, path)| (instrument, playback.add_instrument(load_wav(path))))
        .collect();

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

    pcm.prepare()?;
    let start = Instant::now();
    loop {
        let now = Instant::now();
        // Play for 10 seconds
        if (now - start).as_secs_f64() > 20.0 {
            break;
        }

        for &handle in score
            .update(bpm, now)
            .into_iter()
            .filter_map(|note| sound_handles.get(&note.instrument))
        {
            playback.start_sound(handle);
        }

        playback.update(&pcm)?;
    }
    pcm.drain()?;

    Ok(())
}
