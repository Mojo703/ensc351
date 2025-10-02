#include "hal/mcp320x.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/spi/spidev.h>

MCP3204Result mcp3204_init(int* fd_out) {
    int fd = open(MCP320x_PATH, O_RDWR);
    if (fd < 0) {
        perror("Could not open SPI");
        return MCP320x_SPI_ERROR;
    }

    *fd_out = fd;

    return MCP320x_OK;
}

MCP3204Result mcp3204_cleanup(int fd) {
    if (close(fd) < 0) {
        perror("Could not close SPI");
        return MCP320x_SPI_ERROR;
    }

    return MCP320x_OK;
}

MCP3204Result mcp3204_get(int fd, MCP3204Channel channel, uint16_t* value_out) {
    uint8_t tx_0 = (0x1 << 7 | 0x1 << 6) | channel << 3; // { START[0], SINGLE/DIFF[0], CHANNEL[2, 1, 0] }

    uint8_t tx[] = { tx_0 };
    uint8_t rx[2] = { 0x0, 0x0 };

    struct spi_ioc_transfer tr = {
        .tx_buf = *tx,
        .rx_buf = *rx,
        .len = sizeof(tx) + sizeof(rx),
        .speed_hz = MCP320x_SPI_FREQUENCY,
        .bits_per_word = MCP320x_BITS_PER_WORD,
        .delay_usecs = 0,
    };

    if (ioctl(fd, SPI_IOC_MESSAGE(1), &tr) < 0) {
        perror("Could not send spi message");
        return MCP320x_SPI_ERROR;
    }

    *value_out = (rx[0] | (rx[1] << 8)) & 0xFFF;

    return MCP320x_OK;
}

/*
fn read(&mut self, mode: Mode, address: u8) -> Result<Reading> {
        // START[1] + MODE[1] + ADDR[1/3] + SAMPLE[1] + NULL[1] + DATA[10-13]
        let size = 1 + 1 + self.channels.bit_size() + 1 + 1 + self.resolution.0;
        let bytes = (f32::from(size) / 8f32).ceil() as u8;

        let command: u32 = 1u32 << u32::from(size - 1)
            | ((mode as u32) << u32::from(size - 2))
            | ((u32::from(address)) << (self.resolution.0 + 2)) as u32;

        let mut tx: Vec<u8> = Vec::with_capacity(bytes as usize);
        for i in (0..bytes).rev() {
            let shift = u32::from(8u8 * i);
            tx.push(((command & (0b_1111_1111u32 << shift)) >> shift) as u8);
        }

        let mut rx: Vec<u8> = Vec::with_capacity(bytes as usize);
        for _ in 0..bytes {
            rx.push(0);
        }

        self.spi.transfer(&mut rx.as_mut_slice(), &tx.as_slice())?;

        let mut result: u32 = 0;
        for (i, byte) in rx.iter().enumerate() {
            result |= (u32::from(*byte) << (u32::from(bytes - 1 - i as u8) * 8)) as u32;
        }

        debug_assert_eq!(result >> u32::from(self.resolution.0), 0);

        Ok(Reading::new(result as u16, self.resolution.range().1))
    }
*/