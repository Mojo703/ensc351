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

fn main() {
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

    let mut adc = MCP320X::new("/dev/spidev0.0", 3.3).expect("ADC creation must work.");
    let (mut encoder, mut button) = {
        use gpiod::*;
        let chip = Chip::new("gpiochip0").expect("GPIO chip must be avaliable.");

        let encoder = Options::input([7, 10])
            .active(Active::High)
            .bias(Bias::PullDown);
        let encoder = chip
            .request_lines(encoder)
            .expect("Encoder pin creation must work.");
        let button = Options::input([17])
            .active(Active::High)
            .bias(Bias::PullDown);
        let button = chip
            .request_lines(button)
            .expect("Button pin creation must work.");

        (
            Encoder::new(40, 300, 4, encoder).expect("Encoder creation must work."),
            Button::new(
                button,
                Duration::from_millis(20),
                Duration::from_millis(250),
                Duration::from_millis(100),
            )
            .expect("Button creation must work."),
        )
    };

    let pcm = PCM::new("default", Direction::Playback, false).expect("PCM creation must work");

    let channels = 1;
    let rate = 44100;

    let mut playback = Playback::new(&pcm, channels, rate, channels as usize * 64)
        .expect("Playback start must work.");
    let joystick = Joystick::new(C::CH0, C::CH1);
    let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 3.3);
    let drumkit = Drumkit::new(acc, [1.0, 1.0, 2.0]);

    for (instrument, path) in instruments {
        playback.add_instrument(load_wav(path), instrument);
    }

    let score_choices = [Score::standard, Score::funky];
    let mut score_index = 0;
    let mut score = score_choices[score_index % score_choices.len()]();
    let volume = 0.20;
    let bpm = 100.0;

    pcm.prepare().expect("PCM prepare must work.");

    let log_period = Duration::from_millis(750);
    let mut last_log = None;

    loop {
        let now = Instant::now();

        // Handle changing volume
        // match joystick.get(&mut adc) {
        //     Some(joystick::State::Up) => volume += 5.0,
        //     Some(joystick::State::Down) => volume -= 5.0,
        //     Some(joystick::State::Left) => {
        //         break; // Exit from the program
        //     }
        //     _ => {}
        // }

        // Handle bpm update
        // let bpm = encoder.get_offset() as f64;

        // Handle changing the chosen score.
        // if matches!(button.update(now), Some(_)) {
        //     score_index += 1;
        //     score = score_choices[score_index % score_choices.len()]();
        // }

        // Get the score events
        let events = score.update(bpm, now).into_iter().map(|e| e.instrument);

        // Get the drumkit events
        // let events = events.chain(drumkit.get(&mut adc).into_iter().map(|event| match event {
        //     drumkit::Event::A => Instrument::Snare,
        //     drumkit::Event::B => Instrument::HiHat,
        //     drumkit::Event::C => Instrument::BassDrum,
        // }));

        if last_log.is_none_or(|last| now - last >= log_period) {
            last_log = Some(now);
            println!(
                "\nbpm: {bpm}, volume: {volume}, instruments playing: {}, beat: {:.2}",
                playback.playing_count(),
                score.current_beat(),
            );
        }

        for instrument in events {
            playback.start_sound(instrument);
            print!("{instrument:?}, ");
        }

        playback
            .update(&pcm, volume)
            .expect("Playback update must work.");
    }
    pcm.drain().expect("PCM drain must work.");
}
