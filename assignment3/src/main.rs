use std::time::{Duration, Instant};

use crate::{
    hal::{button::Button, encoder::Encoder, mcp320x::MCP320X},
    input::{
        accelerometer::Accelerometer,
        drumkit::Drumkit,
        joystick::{Direction, Joystick},
    },
    sound::{Instrument, load_wav_mono_i16, playback::Playback, score::ScoreType},
    udp::UdpConn,
    units::{Bpm, Volume},
};
use alsa::PCM;
use hal::mcp320x::Channel as C;

pub mod command;
pub mod hal;
pub mod input;
pub mod sound;
pub mod udp;
pub mod units;

pub struct App<'a> {
    adc: MCP320X,
    encoder: Encoder,
    button: Button,
    joystick: Joystick,
    drumkit: Drumkit,
    udp: Option<UdpConn>,

    playback: Playback<'a, Instrument>,

    score_index: usize,
    score: crate::sound::score::Score,
    volume: Volume,
    bpm: Bpm,

    prev_joystick: Option<(Instant, Direction)>,
    joystick_period: Duration,

    last_log: Option<Instant>,
    log_period: Duration,
}

enum UpdateStatus {
    Continue,
    Quit,
}

impl UpdateStatus {
    fn do_continue(self) -> bool {
        match self {
            UpdateStatus::Continue => true,
            UpdateStatus::Quit => false,
        }
    }
}

impl<'a> App<'a> {
    pub fn new(pcm: &'a PCM) -> Self {
        let adc = MCP320X::new("/dev/spidev0.0", 3.3).expect("ADC creation must work.");
        let (encoder, button) = {
            use gpiod::*;
            let chip = Chip::new("gpiochip0").expect("GPIO chip must be avaliable.");

            let encoder = Options::input([7, 10]) // [GPIO 23, GPIO 24]
                .active(Active::High)
                .bias(Bias::PullDown);
            let encoder = chip
                .request_lines(encoder)
                .expect("Encoder pin creation must work.");
            let button = Options::input([17]) // [GPIO 3]
                .active(Active::Low)
                .bias(Bias::PullDown);
            let button = chip
                .request_lines(button)
                .expect("Button pin creation must work.");

            (
                Encoder::new(encoder).expect("Encoder creation must work."),
                Button::new(
                    button,
                    Duration::from_millis(20),
                    Duration::from_millis(250),
                    Duration::from_millis(100),
                )
                .expect("Button creation must work."),
            )
        };

        let udp = match UdpConn::bind("127.0.0.1:12345") {
            Ok(u) => Some(u),
            Err(e) => {
                eprintln!("Warning: could not bind UDP socket 127.0.0.1:12345: {}", e);
                None
            }
        };

        let channels = 1;
        let rate = 44100;
        let mut playback = Playback::new(pcm, channels, rate, channels as usize * 128)
            .expect("Playback start must work.");

        playback.add_instrument(
            load_wav_mono_i16("./sounds/100051__menegass__gui-drum-bd-hard.wav"),
            Instrument::BassDrum,
        );
        playback.add_instrument(
            load_wav_mono_i16("./sounds/100063__menegass__gui-drum-tom-hi-soft.wav"),
            Instrument::HiHat,
        );
        playback.add_instrument(
            load_wav_mono_i16("./sounds/100059__menegass__gui-drum-snare-soft.wav"),
            Instrument::Snare,
        );

        let joystick = Joystick::new(C::CH0, C::CH1);
        let acc = Accelerometer::new(C::CH2, C::CH3, C::CH4, 1.57, 0.42);
        let drumkit = Drumkit::new(acc, [2.0, 2.0, 2.0], Duration::from_millis(100));

        // Prepare initial score and state
        let score_index = 1usize;
        let score = ScoreType::from_index(score_index).apply();
        let volume = Volume::try_from(20).unwrap();
        let bpm = Bpm::try_from(120).unwrap();

        App {
            adc,
            encoder,
            button,
            joystick,
            drumkit,
            udp,

            playback,

            score_index,
            score,
            volume,
            bpm,

            prev_joystick: None,
            joystick_period: Duration::from_millis(100),

            last_log: None,
            log_period: Duration::from_millis(750),
        }
    }

    pub fn run(mut self, pcm: &'a PCM) {
        pcm.prepare().expect("PCM prepare must work.");

        // Run the update loop, until quit
        while self.update(pcm).do_continue() {}

        pcm.drain().expect("PCM drain must work.");
    }

    fn update(&mut self, pcm: &'a PCM) -> UpdateStatus {
        let now = Instant::now();

        // Handle joystick changes for volume/break
        if let Some(event) = self.joystick.get(&mut self.adc)
            && self
                .prev_joystick
                .is_some_and(|(time, prev)| prev != event || (now - time) > self.joystick_period)
        {
            if let Some(delta) = match event {
                Direction::Up => Some(0.05),
                Direction::Down => Some(-0.05),
                Direction::Left => return UpdateStatus::Quit,
                _ => None,
            } {
                self.set_volume(self.volume + delta);
            }
            self.prev_joystick = Some((now, event));
        }

        let mut notes: Vec<Instrument> = Vec::new();

        // Handle events over UDP
        if let Some(ref udp) = self.udp {
            match udp.try_recv_command() {
                Ok(Some((cmd, addr))) => {
                    println!("UDP command received from {}: {:?}", addr, cmd);
                    match cmd {
                        command::Command::Mode(new_mode) => {
                            self.set_score(ScoreType::from_index(new_mode as usize));
                        }
                        command::Command::Volume(volume) => {
                            self.set_volume(volume);
                        }
                        command::Command::Tempo(bpm) => {
                            self.set_tempo(bpm);
                        }
                        command::Command::Play(trigger) => {
                            notes.push(trigger);
                        }
                        command::Command::Stop => {
                            return UpdateStatus::Quit;
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("UDP receive error: {}", e),
            }
        }

        // Handle bpm update from encoder
        self.encoder.update();
        self.bpm = self.bpm + self.encoder.get_acc_delta();

        // Handle changing the chosen score.
        if self.button.update(now).is_some() {
            self.score_index += 1;
            self.set_score(ScoreType::from_index(self.score_index));
        }

        // Get the score notes
        notes.extend(
            self.score
                .update(self.bpm, now)
                .into_iter()
                .map(|e| e.instrument),
        );

        // Get the drumkit notes
        notes.extend(
            self.drumkit
                .get(&mut self.adc, now)
                .into_iter()
                .map(Instrument::from),
        );

        // Handle logging
        if self
            .last_log
            .is_none_or(|last| now - last >= self.log_period)
        {
            self.last_log = Some(now);
            self.log();
        }

        for instrument in notes {
            self.playback.start_sound(instrument);
        }

        self.playback
            .update(pcm, self.volume)
            .expect("Playback update must work.");

        UpdateStatus::Continue
    }

    fn log(&mut self) {
        let joystick_status = self.joystick.get(&mut self.adc);
        println!(
            "\nbpm: {}, volume: {}, instruments playing: {}, beat: {:.2}, joystick: {:?}, score_index: {}",
            self.bpm,
            self.volume,
            self.playback.playing_count(),
            self.score.get_beat(),
            joystick_status,
            self.score_index,
        );
    }

    fn set_score(&mut self, score: ScoreType) {
        let prev_beat = self.score.get_beat();
        self.score = score.apply();
        self.score.set_beat(prev_beat);
    }

    fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }

    fn set_tempo(&mut self, bpm: Bpm) {
        self.bpm = bpm;
    }
}

fn main() {
    let pcm =
        PCM::new("default", alsa::Direction::Playback, false).expect("PCM creation must work");

    let app = App::new(&pcm);
    app.run(&pcm);
}
