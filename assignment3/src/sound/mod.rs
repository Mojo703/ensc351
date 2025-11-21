use std::rc::Rc;

pub mod playback;
pub mod score;

type Beat = f64;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instrument {
    HiHat,
    Snare,
    BassDrum,
}

impl Instrument {
    pub fn from_index(index: usize) -> Self {
        match usize::strict_rem(index, 3) {
            0 => Instrument::HiHat,
            1 => Instrument::Snare,
            2 => Instrument::BassDrum,
            _ => unreachable!(),
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Instrument::HiHat => 0,
            Instrument::Snare => 1,
            Instrument::BassDrum => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NoteEvent {
    pub instrument: Instrument,
}

pub fn load_wav_mono_i16(path: &str) -> Rc<[i16]> {
    let reader = hound::WavReader::open(path).expect("Failed to open WAV file");
    let spec = reader.spec();

    assert_eq!(
        spec.sample_format,
        hound::SampleFormat::Int,
        "Only integer WAV supported"
    );
    assert_eq!(spec.bits_per_sample, 16, "Only 16-bit WAV supported");
    assert_eq!(spec.channels, 1, "Only mono WAV supported.");

    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .map(|s| s.expect("Error reading sample"))
        .collect();

    Rc::from(samples.into_boxed_slice())
}
