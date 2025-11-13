use std::rc::Rc;

use alsa::{PCM, pcm};

#[derive(Debug, Clone, Copy)]
pub struct InstrumentHandle(usize);

pub struct PlayingSound {
    pos: usize,
    handle: InstrumentHandle,
}

pub struct Playback<'a> {
    instruments: Vec<Rc<[i16]>>,

    playing: Vec<PlayingSound>,

    io: pcm::IO<'a, i16>,

    channels: u32,
    buffer_frame_size: usize,
}

impl<'a> Playback<'a> {
    pub fn new(
        pcm: &'a PCM,
        channels: u32,
        rate: i32,
        buffer_frame_size: usize,
    ) -> alsa::Result<Self> {
        let io = {
            use alsa::pcm::{Access, Format, HwParams};
            // --- Setup ALSA playback device ---
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(channels)?;
            hwp.set_rate(rate as u32, alsa::ValueOr::Nearest)?;
            hwp.set_format(Format::s16())?;
            hwp.set_access(Access::RWInterleaved)?;
            pcm.hw_params(&hwp)?;
            pcm.io_i16()?
        };

        let instrument = Vec::new();
        let playing = Vec::new();

        Ok(Playback {
            instruments: instrument,
            playing,
            io,
            channels,
            buffer_frame_size,
        })
    }

    pub fn add_instrument(&mut self, sound: Rc<[i16]>) -> InstrumentHandle {
        let index = self.instruments.len();
        self.instruments.push(sound);
        InstrumentHandle(index)
    }

    pub fn start_sound(&mut self, handle: InstrumentHandle) {
        self.playing.push(PlayingSound { pos: 0, handle });
    }

    /// Stream small frames of audio
    pub fn update(&mut self, pcm: &'a PCM) -> alsa::Result<()> {
        let status = pcm.status()?;

        // Limit to buffer_frame_size for low latency
        let avail = status.get_avail() as usize;
        let frames_to_write = self.buffer_frame_size.min(avail);
        if frames_to_write == 0 {
            return Ok(());
        }

        let mut buffer = vec![0i16; frames_to_write * self.channels as usize];

        // Mix currently playing instruments into buffer
        self.playing.retain_mut(|p| {
            let sound = &self.instruments[p.handle.0];

            for frame in 0..frames_to_write {
                if p.pos >= sound.len() / self.channels as usize {
                    return false; // This sound has finished playing. Remove it from `self.playing`.
                }

                for ch in 0..self.channels as usize {
                    let si = p.pos * self.channels as usize + ch;
                    let bi = frame * self.channels as usize + ch;

                    buffer[bi] = buffer[bi].saturating_add(sound[si]);
                }

                p.pos += 1;
            }

            true
        });

        // Write mixed frames to ALSA
        self.io.writei(&buffer)?;

        Ok(())
    }
}
