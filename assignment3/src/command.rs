use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::sound::Instrument;
use crate::sound::score::ScoreType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Mode(ScoreType),
    Volume(u32),
    Tempo(u32),
    Play(Instrument),
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
            Command::Mode(n) => write!(f, "mode {}", n.to_index()),
            Command::Volume(n) => write!(f, "volume {}", n),
            Command::Tempo(n) => write!(f, "tempo {}", n),
            Command::Play(n) => write!(f, "play {}", n.to_index()),
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
            "mode" => {
                let num = parts.next().ok_or(Error::MissingArg("mode"))?;
                let n = num
                    .parse::<u32>()
                    .map_err(|e| Error::InvalidArg("mode", e))?
                    .try_into()
                    .map_err(|_| Error::OutOfRangeArg("mode"))?;
                Ok(Command::Mode(ScoreType::from_index(n)))
            }
            "volume" => {
                let num = parts.next().ok_or(Error::MissingArg("volume"))?;
                let n = num
                    .parse::<u32>()
                    .map_err(|e| Error::InvalidArg("volume", e))
                    .and_then(|v| {
                        (0..=100)
                            .contains(&v)
                            .then_some(v)
                            .ok_or(Error::OutOfRangeArg("valid"))
                    })?;
                Ok(Command::Volume(n))
            }
            "tempo" => {
                let num = parts.next().ok_or(Error::MissingArg("tempo"))?;
                let n = num
                    .parse::<u32>()
                    .map_err(|e| Error::InvalidArg("tempo", e))
                    .and_then(|v| {
                        (40..=300)
                            .contains(&v)
                            .then_some(v)
                            .ok_or(Error::OutOfRangeArg("tempo"))
                    })?;

                Ok(Command::Tempo(n))
            }
            "play" => {
                let num = parts.next().ok_or(Error::MissingArg("play"))?;
                let n = num
                    .parse::<u32>()
                    .map_err(|e| Error::InvalidArg("play", e))?
                    .try_into()
                    .map_err(|_| Error::OutOfRangeArg("mode"))?;

                Ok(Command::Play(Instrument::from_index(n)))
            }
            "stop" => Ok(Command::Stop),
            other => Err(Error::Invalid(other.to_owned())),
        }
    }
}
