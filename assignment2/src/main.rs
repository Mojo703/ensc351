mod encoder;
mod mcp320x;
mod pwm;
mod sampler;

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
        Ok(match value.to_lowercase().trim() {
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
    const PWM_PATH: &'static str = "/dev/hat/pwm/GPIO16";
    const UDP_ADDR: &'static str = "0.0.0.0:12345";
    const UDP_BUF_SIZE: usize = 1024;
    const REPORT_PERIOD: time::Duration = time::Duration::from_secs(1);

    let socket = net::UdpSocket::bind(UDP_ADDR)?;
    socket.set_nonblocking(true)?;
    let mut sampler = Sampler::new();
    let mut led = pwm::PWM::new(PWM_PATH);
    let mut encoder = encoder::Encoder::start(4, 50)?;

    let (sample_tx, sample_rx) = sync::mpsc::channel();
    let (sample_kill_tx, sample_kill_rx) = sync::mpsc::channel::<()>();

    let sample_thread = thread::spawn::<_, anyhow::Result<()>>(move || {
        let mut adc = mcp320x::MCP320X::new("/dev/spidev0.0", 3.3)?;

        let period = time::Duration::from_millis(1);
        let mut previous = None;
        loop {
            let now = time::Instant::now();

            if previous.is_some_and(|prev| now - prev < period) {
                continue;
            }

            let voltage: f64 = adc.get_voltage(mcp320x::Channel::CH0)?;
            sample_tx.send(sampler::Sample::new(voltage, now))?;
            previous = Some(now);

            match sample_kill_rx.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => break Ok(()),
            };
        }
    });

    let mut last_report = None;
    let mut previous_command = None;
    led.set_enable(true)?;

    loop {
        let now = time::Instant::now();

        let pwm_freq = pwm::Frequency::hz((encoder.get_offset() as u64 / 4) * 10);
        led.set(pwm_freq, 0.5)?;

        match sample_rx.try_recv() {
            Ok(sample) => sampler.add_sample(sample, now),
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                break Err(anyhow::anyhow!("Sample channel disconnected."));
            }
        }

        if last_report.is_none_or(|last_report| now - last_report > REPORT_PERIOD) {
            let sample_count = sampler.history_size(now);
            let avg = sampler.get_avg();
            let dips = sampler.get_dips_count(now);
            let jitter = sampler.get_jitter_info(now);

            if let Some((avg, jitter)) = avg.zip(jitter) {
                let history: Vec<f64> = sampler.history(now).collect();

                println!(
                    "#Smpl/s = {sample_count:<4} Flash @ {pwm_freq:<7} avg = {avg:<4.3}V dips={dips:<3} {jitter}",
                );

                println!(
                    "{}",
                    (0..10)
                        .map(|i| i * (history.len() - 1) / 9)
                        .map(|i| format!("{i:>4}:{:<5.3} ", history[i]))
                        .collect::<String>()
                );
            }

            last_report = Some(now);
        }

        let mut buf = [0u8; UDP_BUF_SIZE];
        let response = match socket.recv_from(&mut buf).map_err(|e| e.kind()) {
            Ok((len, rx_addr)) => Some((rx_addr, String::from_utf8(buf[..len].to_vec()).ok())),
            Err(io::ErrorKind::WouldBlock) => None,
            Err(e) => break Err(anyhow::anyhow!("UPD socket fault: {:?}", e)),
        };

        match handle_commands(&mut sampler, &mut previous_command, response, now) {
            CommandsResult::Invalid => {}
            CommandsResult::Exit(addr) => {
                socket.send_to(format!("Program terminating.\n").as_bytes(), addr)?;
                break Ok(());
            }
            CommandsResult::Response(addr, text) => {
                socket.send_to(text.as_bytes(), addr)?;
            }
            CommandsResult::ManyResponse(addr, lines) => {
                for line in lines {
                    socket.send_to(line.as_bytes(), addr)?;
                }
            }
        }
    }?;

    sample_kill_tx.send(())?;
    sample_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Sample thread panicked: {:?}", e))??;
    led.set_enable(false)?;
    encoder.end()?;

    Ok(())
}

enum CommandsResult {
    Invalid,
    Response(net::SocketAddr, String),
    ManyResponse(net::SocketAddr, Vec<String>),
    Exit(net::SocketAddr),
}

fn handle_commands(
    sampler: &mut Sampler,
    previous_command: &mut Option<Command>,
    response: Option<(net::SocketAddr, Option<String>)>,
    now: time::Instant,
) -> CommandsResult {
    let Some((rx_addr, command_text)) = response else {
        return CommandsResult::Invalid;
    };

    let Some(command_text) = command_text else {
        return CommandsResult::Invalid;
    };

    let mut command = Command::try_from(command_text.clone());

    if matches!(command, Ok(Command::Repeat)) {
        command = previous_command.ok_or(CommandParseError::Unknown);
    }

    if let Ok(command) = command {
        previous_command.replace(command);
    }

    match command {
        Ok(Command::Help) => CommandsResult::Response(
            rx_addr,
            format!(
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
        ),
        Ok(Command::Count) => CommandsResult::Response(
            rx_addr,
            format!("# samples taken total: {}\n", sampler.get_total_samples()),
        ),
        Ok(Command::Length) => CommandsResult::Response(
            rx_addr,
            format!(
                "# samples taken last second: {}\n",
                sampler.history_size(now)
            ),
        ),
        Ok(Command::Dips) => CommandsResult::Response(
            rx_addr,
            format!("# Dips: {}\n", sampler.get_dips_count(now)),
        ),
        Ok(Command::History) => {
            let history: Vec<f64> = sampler.history(now).collect();

            CommandsResult::ManyResponse(
                rx_addr,
                history
                    .chunks(10)
                    .map(|samples| {
                        samples
                            .iter()
                            .map(|voltage| format!("{voltage:0.3},"))
                            .collect::<String>()
                            + "\n"
                    })
                    .collect(),
            )
        }
        Ok(Command::Stop) => CommandsResult::Exit(rx_addr),
        Ok(Command::Repeat) | Err(CommandParseError::Unknown) => CommandsResult::Response(
            rx_addr,
            format!("Unkown command: \"{}\".\n", command_text.trim()),
        ),
    }
}
