mod sampler;

use rand::Rng;
use sampler::Sampler;
use std::io;
use std::net;
use std::sync;
use std::sync::mpsc;
use std::thread;
use std::time;

#[derive(Debug, Clone, Copy)]
enum Command {
    Help,
    Count,
    Length,
    Dips,
    History,
    Stop,
    Repeat,
}

enum CommandParseError {
    Unknown,
}

impl TryFrom<String> for Command {
    type Error = CommandParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "help" | "?" => Command::Help,
            "count" => Command::Count,
            "length" => Command::Length,
            "dips" => Command::Dips,
            "history" => Command::History,
            "stop" => Command::Stop,
            "" => Command::Repeat,
            _ => return Err(CommandParseError::Unknown),
        })
    }
}

fn main() -> anyhow::Result<()> {
    const UDP_ADDR: &'static str = "0.0.0.0:12345";
    const UDP_BUF_SIZE: usize = 1024;

    let socket = net::UdpSocket::bind(UDP_ADDR)?;
    socket.set_nonblocking(true)?;
    let mut sampler = Sampler::new();
    let mut previous_command = None::<Command>;

    let (sample_tx, sample_rx) = sync::mpsc::channel();

    let sample_thread = thread::spawn(move || {
        let mut rng = rand::rng();
        let voltage: f64 = rng.random();
        let now = time::Instant::now();
        sample_tx.send(sampler::Sample::new(voltage, now))
    });

    loop {
        let now = time::Instant::now();

        match sample_rx.try_recv() {
            Ok(sample) => sampler.add_sample(sample, now),
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                break Err(anyhow::anyhow!("Sample channel disconnected."));
            }
        }

        let mut buf = [0u8; UDP_BUF_SIZE];
        let command = match socket.recv_from(&mut buf).map_err(|e| e.kind()) {
            Ok((len, _)) => String::from_utf8(buf[..len].to_vec())
                .ok()
                .map(|s| Command::try_from(s)),
            Err(io::ErrorKind::WouldBlock) => None,
            Err(e) => break Err(anyhow::anyhow!("UPD socket fault: {:?}", e)),
        };

        if let Some(mut command) = command {
            use Command::*;

            if matches!(command, Ok(Repeat)) {
                command = previous_command.ok_or(CommandParseError::Unknown);
            }

            if let Ok(command) = command {
                previous_command = Some(command);
            }

            let response = match command {
                Ok(Help) => format!(
                    r#"
Accepted command examples:
count   -- get the total number of samples taken.
length  -- get the number of samples taken in the previously completed second.
dips    -- get the number of dips in the previously completed second.
history -- get all the samples in the previously completed second.
stop    -- cause the server program to end.
<enter> -- repeat last command. 
"#
                ),
                Ok(Count) => format!("# samples taken total: {}", sampler.get_total_samples()),
                Ok(Length) => format!("# samples taken last second: {}", sampler.store_size(now)),
                Ok(Dips) => format!("# Dips: {}", sampler.get_dips_count(now)),
                Ok(History) => sampler
                    .history(now)
                    .enumerate()
                    .map(|(i, v)| format!("{v},{}", if i % 10 == 9 { "\n" } else { "" }))
                    .collect::<String>(),
                Ok(Stop) => break Ok(()),
                Ok(Repeat) | Err(CommandParseError::Unknown) => format!("Unkown command."),
            };

            socket.send(response.as_bytes())?;
        }
    }?;

    socket.send("Program terminating.".as_bytes())?;

    sample_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Thread panicked: {:?}", e))??;

    Ok(())
}
