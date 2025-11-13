/**
 * Hardware interface for the MCP320X line of SPI ADCs.
 */
use std::{fmt::Display, io};

use linux_embedded_hal::spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

/// The channels that can be polled.
#[derive(Debug, Clone, Copy)]
pub enum Channel {
    CH0 = 0,
    CH1 = 1,
    CH2 = 2,
    CH3 = 3,
    CH4 = 4,
    CH5 = 5,
    CH6 = 6,
    CH7 = 7,
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::CH0 => "CH0",
            Self::CH1 => "CH1",
            Self::CH2 => "CH2",
            Self::CH3 => "CH3",
            Self::CH4 => "CH4",
            Self::CH5 => "CH5",
            Self::CH6 => "CH6",
            Self::CH7 => "CH7",
        };
        writeln!(f, "{}", text)
    }
}

/// MCP320X connected over SPI, with an assumed Vref.
pub struct MCP320X {
    spi: Spidev,
    vref: f64,
}

impl MCP320X {
    pub const MAX_RAW: u16 = 4096;

    /// Open a connection with an MCP320X over SPI.
    pub fn new<P: AsRef<std::path::Path>>(path: P, vref: f64) -> io::Result<Self> {
        let mut spi = Spidev::open(path)?;
        spi.configure(&SpidevOptions {
            spi_mode: Some(SpiModeFlags::SPI_MODE_0),
            bits_per_word: Some(8),
            max_speed_hz: Some(500000),
            lsb_first: Some(false),
        })?;

        Ok(Self { spi, vref })
    }

    /// Get a single measurement from the ADC. Scaled to [0.0, 1.0)
    pub fn get_single(&mut self, channel: Channel) -> io::Result<f64> {
        let tx_buf = Self::get_tx(channel);
        let mut rx_buf = [0; 3];

        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        self.spi.transfer(&mut transfer)?;

        Ok(Self::parse_rx(rx_buf) as f64 / Self::MAX_RAW as f64)
    }

    /// Get the median of multiple measurements from the ADC. Scaled to [0.0, 1.0)
    pub fn get_median(&mut self, channel: Channel, sample_count: usize) -> io::Result<f64> {
        let mut samples: Vec<_> = (0..sample_count)
            .map(move |_| self.get_single(channel))
            .collect::<Result<_, _>>()?;

        let mid = sample_count / 2;
        let (_, median, _) = samples.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap());

        Ok(*median)
    }

    /// Get a single voltage measurement from the ADC. Based on the assumed vref.
    pub fn get_voltage_single(&mut self, channel: Channel) -> io::Result<f64> {
        self.get_single(channel).map(|sample| sample * self.vref)
    }

    /// Collect many voltage samples, and calculate the median. Used to counteract noise and bad readings.
    pub fn get_voltage_median(&mut self, channel: Channel, sample_count: usize) -> io::Result<f64> {
        self.get_median(channel, sample_count)
            .map(|sample| sample * self.vref)
    }

    /// Get the packet to transmit.
    fn get_tx(channel: Channel) -> [u8; 3] {
        let channel = channel as u8;
        [
            (0x1 << 2) | (0x1 << 1) | (channel >> 2), // [0b0000_0(start)(single)(d2)]
            (channel << 6),                           // [0b(D1)(D0)xx_xxxx]
            0x0,                                      // [0bxxxx_xxxx]
        ]
    }

    /// Parse the received packet for the raw measurement
    fn parse_rx(rx: [u8; 3]) -> u16 {
        // rx = { [0bZZZZ_ZZZZ], [0bZZZ(null)_(B11)(B10)(B9)(B8)], [0b(B7)(B6)(B5)(B4)_(B3)(B2)(B1)(B0)] }
        let rx1 = (rx[1] & 0x0F) as u16;
        let rx2 = rx[2] as u16;
        (rx1 << 8) | rx2
    }
}
