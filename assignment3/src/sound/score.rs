use std::time::Instant;

use crate::sound::{Beat, Instrument, NoteEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreType {
    Empty,
    Standard,
    Funky,
}

impl ScoreType {
    pub fn from_index(index: usize) -> Self {
        match usize::strict_rem(index, 3) {
            0 => ScoreType::Empty,
            1 => ScoreType::Standard,
            2 => ScoreType::Funky,
            _ => unreachable!(),
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            ScoreType::Empty => 0,
            ScoreType::Standard => 1,
            ScoreType::Funky => 2,
        }
    }

    pub fn apply(self) -> Score {
        match self {
            ScoreType::Empty => Score::empty(),
            ScoreType::Standard => Score::standard(),
            ScoreType::Funky => Score::funky(),
        }
    }
}

pub struct Track {
    instrument: Instrument,
    notes: Vec<Beat>,
}

pub struct Score {
    tracks: Vec<Track>,
    length: Beat,

    prev: Option<Instant>,
    beat_time: Beat,
}

impl Score {
    pub fn empty() -> Self {
        Self {
            tracks: vec![],
            length: 8.0,
            prev: None,
            beat_time: 0.0,
        }
    }

    pub fn standard() -> Self {
        let hihat = Track {
            instrument: Instrument::HiHat,
            notes: vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0],
        };

        let snare = Track {
            instrument: Instrument::Snare,
            notes: vec![4.0, 12.0],
        };

        let bassdrum = Track {
            instrument: Instrument::BassDrum,
            notes: vec![0.0, 8.0],
        };

        Self {
            tracks: vec![hihat, snare, bassdrum],
            length: 8.0,
            prev: None,
            beat_time: 0.0,
        }
    }

    pub fn funky() -> Self {
        let hihat = Track {
            instrument: Instrument::HiHat,
            notes: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.5, 6.0, 7.0, 7.5],
        };

        let snare = Track {
            instrument: Instrument::Snare,
            notes: vec![2.0, 6.0],
        };

        let bassdrum = Track {
            instrument: Instrument::BassDrum,
            notes: vec![0.0, 3.0, 4.0, 7.0],
        };

        Self {
            tracks: vec![hihat, snare, bassdrum],
            length: 8.0,
            prev: None,
            beat_time: 0.0,
        }
    }

    pub fn update(&mut self, bpm: f64, now: Instant) -> Vec<NoteEvent> {
        let Some(prev) = self.prev else {
            self.prev = Some(now);
            return Vec::new();
        };

        let elapsed: Beat = (now - prev).as_secs_f64() * (bpm / 60.0);

        let start: Beat = self.beat_time;
        let end: Beat = self.beat_time + elapsed;

        let offset = (start / self.length).floor() * self.length;

        let events = self
            .tracks
            .iter()
            .flat_map(|track| {
                let instrument = track.instrument;
                track
                    .notes
                    .iter()
                    .filter(|&&time| {
                        let time = time + offset;
                        time > start && time < end
                    })
                    .map(move |_| NoteEvent { instrument })
            })
            .collect();

        self.beat_time = end;
        self.prev = Some(now);

        events
    }

    pub fn get_beat(&self) -> Beat {
        self.beat_time
    }

    pub fn set_beat(&mut self, beat: Beat) {
        self.beat_time = beat;
    }
}
