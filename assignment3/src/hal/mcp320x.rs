/**
 * Hardware interface for the MCP320X line of SPI ADCs.
 */
use std::io;

use linux_embedded_hal::spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

/// The channels that can be polled.
#[derive(Debug, Clone, Copy)]
pub enum Channel {
    CH0 = 0,
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

    /// Get a single raw measurement from the ADC.
    pub fn get(&mut self, channel: Channel) -> io::Result<u16> {
        let tx_buf = Self::get_tx(channel);
        let mut rx_buf = [0; 3];

        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        self.spi.transfer(&mut transfer)?;

        Ok(Self::parse_rx(rx_buf))
    }

    /// Get a single voltage measurement from the ADC. Based on the assumed vref.
    pub fn get_voltage(&mut self, channel: Channel) -> io::Result<f64> {
        self.get(channel)
            .map(|sample| sample as f64 * self.vref / Self::MAX_RAW as f64)
    }

    /// Collect many voltage samples, and calculate the median. Used to counteract noise and bad readings.
    pub fn get_median_voltage(&mut self, channel: Channel, sample_count: usize) -> io::Result<f64> {
        let mut samples: Vec<f64> = (0..sample_count)
            .map(move |_| self.get_voltage(channel))
            .collect::<Result<_, _>>()?;

        let mid = sample_count / 2;
        let (_, median, _) = samples.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap());

        Ok(*median)
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
