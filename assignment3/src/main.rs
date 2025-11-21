use std::time::{Duration, Instant};

use crate::{
    hal::{button::Button, encoder::Encoder, mcp320x::MCP320X},
    input::{
        accelerometer::Accelerometer,
        drumkit::{self, Drumkit},
        joystick::{self, Joystick},
    },
    sound::{Instrument, load_wav, playback::Playback, score::ScoreType},
    udp::UdpConn,
};
use alsa::{Direction, PCM};
use hal::mcp320x::Channel as C;

pub mod command;
pub mod hal;
pub mod input;
pub mod sound;
pub mod udp;

fn main() {
    let instruments = [
        (
            Instrument::BassDrum,
            "./sounds/100051__menegass__gui-drum-bd-hard.wav",
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
            .active(Active::Low)
            .bias(Bias::PullDown);
        let button = chip
            .request_lines(button)
            .expect("Button pin creation must work.");

        (
            Encoder::new(40, 300, 120, encoder).expect("Encoder creation must work."),
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

    let mut playback = Playback::new(&pcm, channels, rate, channels as usize * 128)
        .expect("Playback start must work.");
    let joystick = Joystick::new(C::CH0, C::CH1);
    let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 1.57, 0.42);
    let mut drumkit = Drumkit::new(acc, [2.0, 2.0, 2.0], Duration::from_millis(100));

    let udp = match UdpConn::bind("127.0.0.1:12345") {
        Ok(u) => Some(u),
        Err(e) => {
            eprintln!("Warning: could not bind UDP socket 127.0.0.1:12345: {}", e);
            None
        }
    };

    for (instrument, path) in instruments {
        playback.add_instrument(load_wav(path), instrument);
    }

    let mut score_index = 1;
    let mut score = ScoreType::from_index(score_index).apply();
    let mut volume = 0.20;
    let mut bpm = 120.0;

    let mut prev_joystick = None;
    let joystick_period = Duration::from_millis(100);

    let mut last_log = None;
    let log_period = Duration::from_millis(750);

    pcm.prepare().expect("PCM prepare must work.");

    loop {
        let now = Instant::now();

        // Handle changing volume
        if let Some(event) = joystick.get(&mut adc) {
            if prev_joystick
                .is_some_and(|(time, prev)| prev != event || (now - time) > joystick_period)
            {
                match event {
                    joystick::State::Up => volume += 0.05,
                    joystick::State::Down => volume -= 0.05,
                    joystick::State::Left => break,
                    _ => {}
                }
            }
            prev_joystick = Some((now, event));
        }

        let mut events = Vec::new();
        if let Some(ref udp) = udp {
            match udp.try_recv_command() {
                Ok(Some((cmd, addr))) => {
                    println!("UDP command received from {}: {:?}", addr, cmd);

                    match cmd {
                        command::Command::Mode(new_mode) => score = new_mode.apply(),
                        command::Command::Volume(new_volume) => volume = new_volume as f32 / 100.0,
                        command::Command::Tempo(new_tempo) => bpm = new_tempo as f64,
                        command::Command::Play(trigger) => {
                            events.push(trigger);
                        }
                        command::Command::Stop => todo!(),
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("UDP receive error: {}", e),
            }
        }

        // Handle bpm update
        encoder.update();
        let bpm = encoder.get_offset() as f64;

        // Handle changing the chosen score.
        if matches!(button.update(now), Some(_)) {
            score_index += 1;
            let prev_beat = score.get_beat();
            score = ScoreType::from_index(score_index).apply();
            score.set_beat(prev_beat);
        }

        // Get the score events
        events.extend(score.update(bpm, now).into_iter().map(|e| e.instrument));

        // Get the drumkit events
        events.extend(drumkit.get(&mut adc, now).into_iter().map(|d| match d {
            drumkit::Event::A => Instrument::BassDrum,
            drumkit::Event::B => Instrument::HiHat,
            drumkit::Event::C => Instrument::Snare,
        }));

        if last_log.is_none_or(|last| now - last >= log_period) {
            last_log = Some(now);

            let joystick = joystick.get(&mut adc);
            println!(
                "\nbpm: {bpm}, volume: {volume}, instruments playing: {}, beat: {:.2}, joystick: {joystick:?}, score_index: {score_index}, get_offset: {}",
                playback.playing_count(),
                score.get_beat(),
                encoder.get_offset(),
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
