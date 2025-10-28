use std::{sync::mpsc, thread};

enum Pulse {
    Cw,
    Ccw,
}

impl Pulse {
    fn delta(self) -> i32 {
        match self {
            Self::Cw => -1,
            Self::Ccw => 1,
        }
    }
}

pub struct Encoder {
    data_rx: mpsc::Receiver<Pulse>,
    kill_tx: mpsc::Sender<()>,
    handle: thread::JoinHandle<std::io::Result<()>>,
    offset: i32,
    limit_min: i32,
    limit_max: i32,
}

impl Encoder {
    pub fn start(limit_min: i32, limit_max: i32) -> std::io::Result<Self> {
        let (data_tx, data_rx) = mpsc::channel::<Pulse>();
        let (kill_tx, kill_rx) = mpsc::channel::<()>();

        let handle = thread::spawn(move || {
            let chip = gpiod::Chip::new("gpiochip0")?;

            let pins = gpiod::Options::input([7, 10])
                .active(gpiod::Active::High)
                .bias(gpiod::Bias::PullDown);
            let pins = chip.request_lines(pins)?;

            let mut last_state = (false, false);

            loop {
                let [a, b] = pins.get_values([false, false]).unwrap();
                let state = (a, b);
                if let Some(pulse) = match (last_state, state) {
                    ((false, false), (true, false))
                    | ((true, false), (true, true))
                    | ((true, true), (false, true))
                    | ((false, true), (false, false)) => Some(Pulse::Cw),
                    ((false, false), (false, true))
                    | ((false, true), (true, true))
                    | ((true, true), (true, false))
                    | ((true, false), (false, false)) => Some(Pulse::Ccw),
                    _ => None,
                } {
                    last_state = state;
                    data_tx.send(pulse).expect(
                        "Encoder data channel must exist. Maybe the thread was ended early.",
                    );
                }

                match kill_rx.try_recv() {
                    Err(mpsc::TryRecvError::Empty) => {}
                    Ok(_) | Err(mpsc::TryRecvError::Disconnected) => break Ok(()),
                }
            }
        });

        Ok(Self {
            kill_tx,
            data_rx,
            handle,
            offset: 0,
            limit_max,
            limit_min,
        })
    }

    pub fn get_offset(&mut self) -> i32 {
        self.offset = self
            .offset
            .saturating_add(self.data_rx.try_iter().map(|pulse| pulse.delta()).sum())
            .clamp(self.limit_min, self.limit_max);

        self.offset
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.kill_tx.send(())?;
        self.handle
            .join()
            .map_err(|e| anyhow::anyhow!("Encoder thread panicked: {e:?}"))??;

        Ok(())
    }
}
