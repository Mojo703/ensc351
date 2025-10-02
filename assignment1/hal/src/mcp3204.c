#include "hal/mcp3204.h"
#include <stdio.h>


MCP3204Result mcp3204_get(int* value) {
    uint8_t tx[] = { 0x0 };
    uint8_t rx[2] = { 0x0 };

    int fd = open("/dev/spidev0.0", O_RDWR);

    if (fd < 0) {
        perror("open");
        return MCP3204_SPI_ERROR;
    }

    struct spi_ioc_transfer tr = {
        .tx_buf = tx,
        .rx_buf = rx,
        .len = sizeof(tx) + sizeof(rx),
        .speed_hz = MCP3204_SPI_FREQUENCY,
        .bits_per_word = MCP3204_BITS_PER_WORD,
        .delay_usecs = 0,
    };

    if (ioctl(fd, SPI_IOC_MESSAGE(1), &tr) < 1) {
        perror("Can't send spi message");
        return MCP3204_SPI_ERROR;
    }

    *value = (rx[0] | (rx[1] << 8)) & 0xFFF;

    return MCP3204_OK;
}