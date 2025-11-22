use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::sound::Instrument;
use crate::sound::score::ScoreType;
use crate::units::{Bpm, Volume};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Mode(Option<ScoreType>),
    Volume(Option<Volume>),
    Tempo(Option<Bpm>),
    Play(Option<Instrument>),
    Stop,
}

pub enum Error {
    Empty,
    Invalid(String),
    MissingArg(&'static str),
    InvalidArg(&'static str, ParseIntError),
    OutOfRangeArg(&'static str),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Mode(n) => write!(
                f,
                "mode {}",
                n.map(|v| v.to_index().to_string())
                    .unwrap_or("null".to_owned())
            ),
            &Command::Volume(n) => write!(
                f,
                "volume {}",
                n.map(|v| u32::from(v).to_string())
                    .unwrap_or("null".to_owned())
            ),
            &Command::Tempo(n) => write!(
                f,
                "tempo {}",
                n.map(|v| u32::from(v).to_string())
                    .unwrap_or("null".to_owned())
            ),
            Command::Play(n) => write!(
                f,
                "play {}",
                n.map(|v| v.to_index().to_string())
                    .unwrap_or("null".to_owned())
            ),
            Command::Stop => write!(f, "stop"),
        }
    }
}

impl FromStr for Command {
    type Err = Error;

    /// Parse a command from a UTF-8 string received over UDP.
    ///
    /// Examples of accepted forms:
    /// - "mode 1"
    /// - "volume 50"
    /// - "tempo 120"
    /// - "play 2"
    /// - "stop"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::Empty);
        }

        let mut parts = s.split_whitespace();
        let cmd = parts.next().unwrap().to_lowercase();

        match cmd.as_str() {
            "mode" => Ok(Command::Mode(
                parts
                    .next()
                    .and_then(|p| p.parse().ok())
                    .map(|n| ScoreType::from_index(n)),
            )),
            "volume" => Ok(Command::Volume(
                parts
                    .next()
                    .and_then(|p| p.parse::<u32>().ok())
                    .and_then(|n| Volume::try_from(n).ok()),
            )),
            "tempo" => Ok(Command::Tempo(
                parts
                    .next()
                    .and_then(|p| p.parse::<u32>().ok())
                    .and_then(|n| Bpm::try_from(n).ok()),
            )),
            "play" => Ok(Command::Play(
                parts
                    .next()
                    .and_then(|p| p.parse().ok())
                    .map(|n| Instrument::from_index(n)),
            )),
            "stop" => Ok(Command::Stop),
            other => Err(Error::Invalid(other.to_owned())),
        }
    }
}
