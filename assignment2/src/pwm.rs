use std::fmt::Display;
use std::io::{self, Write};
use std::{fs, path, time};

#[derive(Debug, Clone, Copy)]
pub struct Frequency(u64);

impl Frequency {
    pub fn hz(value: u64) -> Self {
        Self(value)
    }

    pub fn as_hz(self) -> u64 {
        self.0
    }
}

impl Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}Hz", self.as_hz())
    }
}

fn write_sysfs<P: AsRef<path::Path>>(path: P, value: &[u8]) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    file.write_all(value)?;
    std::thread::sleep(time::Duration::from_millis(1));
    Ok(())
}

pub struct Pwm {
    path: path::PathBuf,

    period: Option<time::Duration>,
    duty_cycle: Option<time::Duration>,
    enable: Option<bool>,
}

impl Pwm {
    pub fn new<P: Into<path::PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            period: None,
            duty_cycle: None,
            enable: None,
        }
    }

    pub fn set(&mut self, frequency: Frequency, duty_cycle: f64) -> io::Result<()> {
        let period = 1_000_000_000 / frequency.as_hz();
        let duty_cycle = (1_000_000_000.0 * duty_cycle).round() as u64 / frequency.as_hz();

        let period = time::Duration::from_nanos(period);
        let duty_cycle = time::Duration::from_nanos(duty_cycle);

        if self.period.is_some_and(|current| current == period)
            && self.duty_cycle.is_some_and(|current| current == duty_cycle)
        {
            return Ok(());
        }

        write_sysfs(self.path.join("duty_cycle"), b"250000")?;
        write_sysfs(self.path.join("period"), b"500000")?;
        write_sysfs(
            self.path.join("period"),
            format!("{}", period.as_nanos()).as_bytes(),
        )?;
        write_sysfs(
            self.path.join("duty_cycle"),
            format!("{}", duty_cycle.as_nanos()).as_bytes(),
        )?;

        self.duty_cycle = Some(duty_cycle);
        self.period = Some(period);

        Ok(())
    }

    pub fn set_enable(&mut self, enable: bool) -> io::Result<()> {
        if self.enable.is_some_and(|current| current == enable) {
            return Ok(());
        }
        self.enable = Some(enable);
        write_sysfs(self.path.join("enable"), if enable { b"1" } else { b"0" })
    }
}
