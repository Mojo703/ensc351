/**
 * Hardware interface for linux PWM peripherals.
 */
use std::fmt::Display;
use std::io::{self, Write};
use std::{fs, path, time};

/// Physical frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Open a file temporarily and write the value.
fn write_sysfs<P: AsRef<path::Path>>(path: P, value: &[u8]) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    file.write_all(value)?;
    std::thread::sleep(time::Duration::from_millis(1));
    Ok(())
}

/// PWM interface for linux. Allows control of the output frequency.
pub struct Pwm {
    path: path::PathBuf,

    previous: Option<Frequency>,
    is_enabled: bool,
}

impl Pwm {
    pub fn new<P: Into<path::PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            previous: None,
            is_enabled: false,
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        write_sysfs(self.path.join("period"), b"1000000\n")?;
        write_sysfs(self.path.join("duty_cycle"), b"500000\n")?;
        write_sysfs(self.path.join("enable"), b"0\n")?;
        Ok(())
    }

    pub fn set(&mut self, frequency: Frequency) -> io::Result<()> {
        if frequency.as_hz() == 0 {
            self.set_enable(false)?;
        } else if self.previous.is_none_or(|prev| prev != frequency) {
            let period = 1_000_000_000 / frequency.as_hz();
            let duty = period / 2;

            write_sysfs(self.path.join("duty_cycle"), b"250000")?;
            write_sysfs(self.path.join("period"), b"500000")?;
            write_sysfs(self.path.join("period"), format!("{}", period).as_bytes())?;
            write_sysfs(self.path.join("duty_cycle"), format!("{}", duty).as_bytes())?;
            self.set_enable(true)?;

            self.previous = Some(frequency);
        };

        Ok(())
    }

    pub fn set_enable(&mut self, enable: bool) -> io::Result<()> {
        if self.is_enabled == enable {
            return Ok(());
        }
        self.is_enabled = enable;
        write_sysfs(self.path.join("enable"), if enable { b"1" } else { b"0" })
    }
}
