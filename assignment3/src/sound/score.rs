use std::time::Instant;

use crate::sound::{Beat, Instrument, NoteEvent};

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

    pub fn update(&mut self, bpm: f64, now: Instant) -> Vec<NoteEvent> {
        let Some(prev) = self.prev else {
            self.prev = Some(now);
            return Vec::new();
        };

        let elapsed = now - prev;
        let elapsed: Beat = elapsed.as_secs_f64() * (bpm / 60.0);

        let loop_len: Beat = self.length; // loop length in beats
        let start: Beat = (self.beat_time) % loop_len;
        let end: Beat = (self.beat_time + elapsed) % loop_len;

        let events = self
            .tracks
            .iter()
            .flat_map(|track| {
                let instrument = track.instrument;
                track
                    .notes
                    .iter()
                    .filter(|&&time| {
                        if end > start {
                            time > start && time <= end
                        } else {
                            // Handle looping
                            (time > start && time <= loop_len) || (time >= 0.0 && time <= end)
                        }
                    })
                    .map(move |_| NoteEvent { instrument })
            })
            .collect();

        self.beat_time = end;

        events
    }
}
