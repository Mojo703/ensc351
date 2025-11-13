pub mod score;

type Beat = f64;

#[derive(Debug, Clone, Copy)]
pub enum Instrument {
    HiHat,
    Snare,
    BassDrum,
}

pub struct NoteEvent {
    instrument: Instrument,
}
