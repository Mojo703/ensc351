use std::io;

use linux_embedded_hal::spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SPI fault: {0}")]
    SPI(#[from] io::Error),
}

pub enum Channel {
    CH0 = 0,
}

pub struct MCP320X {
    spi: Spidev,
    vref: f64,
}

impl MCP320X {
    pub fn new<P: AsRef<std::path::Path>>(path: P, vref: f64) -> Result<Self, Error> {
        let mut spi = Spidev::open(path)?;
        spi.configure(&SpidevOptions {
            spi_mode: Some(SpiModeFlags::SPI_MODE_0),
            bits_per_word: Some(8),
            max_speed_hz: Some(500000),
            lsb_first: Some(false),
        })?;

        Ok(Self { spi, vref })
    }

    pub fn get(&mut self, channel: Channel) -> Result<u16, Error> {
        let tx_buf = Self::get_tx(channel);
        let mut rx_buf = [0; 3];

        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        self.spi.transfer(&mut transfer)?;

        Ok(Self::parse_rx(rx_buf))
    }

    pub fn get_voltage(&mut self, channel: Channel) -> Result<f64, Error> {
        self.get(channel)
            .map(|sample| sample as f64 * self.vref / 4096.0)
    }

    fn get_tx(channel: Channel) -> [u8; 3] {
        let channel = channel as u8;
        [
            (0x1 << 2) | (0x1 << 1) | (channel >> 2), // [0b0000_0(start)(single)(d2)]
            (channel << 6),                           // [0b(D1)(D0)xx_xxxx]
            0x0,                                      // [0bxxxx_xxxx]
        ]
    }

    fn parse_rx(rx: [u8; 3]) -> u16 {
        // rx = { [0bZZZZ_ZZZZ], [0bZZZ(null)_(B11)(B10)(B9)(B8)], [0b(B7)(B6)(B5)(B4)_(B3)(B2)(B1)(B0)] }
        let rx1 = (rx[1] & 0x0F) as u16;
        let rx2 = rx[2] as u16;
        (rx1 << 8) | rx2
    }
}
