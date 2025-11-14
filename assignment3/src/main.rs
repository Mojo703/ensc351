use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    hal::{button::Button, encoder::Encoder, mcp320x::MCP320X},
    input::{
        accelerometer::Accelerometer,
        drumkit::{self, Drumkit},
        joystick::Joystick,
    },
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
    let (mut encoder, mut button) = {
        use gpiod::*;
        let chip = Chip::new("gpiochip0")?;

        let encoder = Options::input([7, 10])
            .active(Active::High)
            .bias(Bias::PullDown);
        let encoder = chip.request_lines(encoder)?;
        let button = Options::input([7, 10])
            .active(Active::High)
            .bias(Bias::PullDown);
        let button = chip.request_lines(button)?;

        (
            Encoder::new(0, 100, 25, encoder)?,
            Button::new(
                button,
                Duration::from_millis(20),
                Duration::from_millis(250),
                Duration::from_millis(100),
            )?,
        )
    };

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

    println!("joystick reading: {:?}", joystick.get(&mut adc));

    let score_choices = [Score::standard, Score::funky];
    let mut score = Score::standard();
    let mut score_index = 0;
    let bpm = 100.0;

    pcm.prepare()?;
    let start = Instant::now();
    loop {
        let now = Instant::now();
        // Play for 10 seconds
        if (now - start).as_secs_f64() > 20.0 {
            break;
        }

        // Handle changing the chosen score.
        if matches!(button.update(now), Some(_)) {
            score_index += 1;
            score = score_choices[score_index % score_choices.len()]();
        }

        // Get the score events
        let events = score.update(bpm, now).into_iter().map(|e| e.instrument);

        // Get the drumkit events
        let events = events.chain(drumkit.get(&mut adc).into_iter().map(|event| match event {
            drumkit::Event::A => Instrument::Snare,
            drumkit::Event::B => Instrument::HiHat,
            drumkit::Event::C => Instrument::BassDrum,
        }));

        for &handle in events.filter_map(|instrument| sound_handles.get(&instrument)) {
            playback.start_sound(handle);
        }

        let volume = encoder.get_offset() as f64 / 100.0;
        playback.update(&pcm, volume)?;
    }
    pcm.drain()?;

    Ok(())
}
