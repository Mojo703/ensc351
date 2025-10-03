#include "hal/mcp320x.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/spi/spidev.h>

void create_header(uint8_t *tx, MCP320xChannel channel)
{
    // tx[0] = [0b0000_0(start)(single)(d2)]
    tx[0] = (0x1 << 2) | (0x1 << 1) | (channel >> 2);

    // tx[1] = [0b(D1)(D0)xx_xxxx]
    tx[1] = (channel << 6);

    // tx[2] = [0bxxxx_xxxx]
    tx[2] = 0x0;
}

uint16_t get_adc_value(uint8_t *rx)
{
    // rx = { [0bZZZZ_ZZZZ], [0bZZZ(null)_(B11)(B10)(B9)(B8)], [0b(B7)(B6)(B5)(B4)_(B3)(B2)(B1)(B0)] }
    // value = [0b0000_(B11)(B10)(B9)(B8)_(B7)(B6)(B5)(B4)_(B3)(B2)(B1)(B0)]
    return ((rx[1] & 0x0F) << 8) | rx[2];
}

MCP320xResult mcp320x_init(int *fd_out)
{
    int fd = open(MCP320x_PATH, O_RDWR);
    if (fd < 0)
    {
        perror("Could not open SPI");
        return MCP320x_SPI_ERROR;
    }

    // Set the Mode for TX and RX.
    uint8_t wr_mode = SPI_MODE_0; // SPI_MODE_0 = (TX on rising edge, RX on falling edge)
    if (ioctl(fd, SPI_IOC_WR_MODE, &wr_mode) < 0)
        return MCP320x_SPI_ERROR;

    // Set to MSB first.
    uint8_t lsb = 0; // 0 = MSB first, 1 = LSB first
    if (ioctl(fd, SPI_IOC_WR_LSB_FIRST, &lsb) < 0)
        return MCP320x_SPI_ERROR;

    uint8_t bits = MCP320x_BITS_PER_WORD;
    if (ioctl(fd, SPI_IOC_WR_BITS_PER_WORD, &bits) < 0)
        return MCP320x_SPI_ERROR;
    if (ioctl(fd, SPI_IOC_RD_BITS_PER_WORD, &bits) < 0)
        return MCP320x_SPI_ERROR;

    uint32_t speed = MCP320x_SPI_FREQUENCY;
    if (ioctl(fd, SPI_IOC_WR_MAX_SPEED_HZ, &speed) < 0)
        return MCP320x_SPI_ERROR;
    if (ioctl(fd, SPI_IOC_RD_MAX_SPEED_HZ, &speed) < 0)
        return MCP320x_SPI_ERROR;

    *fd_out = fd;

    return MCP320x_OK;
}

MCP320xResult mcp320x_cleanup(int fd)
{
    if (close(fd) < 0)
    {
        perror("Could not close SPI");
        return MCP320x_SPI_ERROR;
    }

    return MCP320x_OK;
}

MCP320xResult mcp320x_get(int fd, MCP320xChannel channel, uint16_t *value_out)
{
    uint8_t tx[MCP320x_TRANSMIT_LENGTH] = {0};
    uint8_t rx[MCP320x_TRANSMIT_LENGTH] = {0};

    create_header(tx, channel);

    struct spi_ioc_transfer tr = {
        .tx_buf = (unsigned long)tx,
        .rx_buf = (unsigned long)rx,
        .len = MCP320x_TRANSMIT_LENGTH,
    };

    if (ioctl(fd, SPI_IOC_MESSAGE(1), &tr) < 0)
    {
        perror("Could not send spi message");
        return MCP320x_SPI_ERROR;
    }

    *value_out = get_adc_value(rx);

    return MCP320x_OK;
}
