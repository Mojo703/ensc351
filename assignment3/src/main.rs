use std::time::{Duration, Instant};

use crate::{
    hal::{button::Button, encoder::Encoder, mcp320x::MCP320X},
    input::{
        accelerometer::Accelerometer,
        drumkit::{self, Drumkit},
        joystick::{self, Joystick},
    },
    sound::{Instrument, load_wav, playback::Playback, score::Score},
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
            Encoder::new(40, 300, 120, encoder)?,
            Button::new(
                button,
                Duration::from_millis(20),
                Duration::from_millis(250),
                Duration::from_millis(100),
            )?,
        )
    };

    let pcm = PCM::new("default", Direction::Playback, false)?;

    let channels = 1;
    let rate = 44100;

    let mut playback = Playback::new(&pcm, channels, rate, channels as usize * 4000)?;
    let joystick = Joystick::new(C::CH0, C::CH1);
    let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 3.3);
    let drumkit = Drumkit::new(acc, [1.0, 1.0, 2.0]);

    for (instrument, path) in instruments {
        playback.add_instrument(load_wav(path), instrument);
    }

    let score_choices = [Score::standard, Score::funky];
    let mut score = Score::standard();
    let mut score_index = 0;
    let mut volume = 80.0;

    pcm.prepare()?;
    loop {
        let now = Instant::now();

        // Handle changing volume
        match joystick.get(&mut adc) {
            Some(joystick::State::Up) => volume += 5.0,
            Some(joystick::State::Down) => volume -= 5.0,
            Some(joystick::State::Left) => {
                break; // Exit from the program
            }
            _ => {}
        }

        // Handle bpm update
        let bpm = encoder.get_offset() as f64;

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

        for instrument in events {
            playback.start_sound(instrument);
        }

        playback.update(&pcm, volume)?;
    }
    pcm.drain()?;

    Ok(())
}
