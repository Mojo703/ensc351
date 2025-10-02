#ifndef _MCP320x_H_
#define _MCP320x_H_

#define MCP320x_PATH "/dev/spidev0.0"
#define MCP320x_SPI_FREQUENCY 1000000
#define MCP320x_BITS_PER_WORD 8

typedef enum {
    MCP320x_SPI_ERROR = -1,
    MCP320x_OK = 0,
} MCP320xResult;

typedef enum {
    // MCP3204 and MCP3208
    MCP320x_CH0 = 0x000,
    MCP320x_CH1 = 0x001,
    MCP320x_CH2 = 0x010,
    MCP320x_CH3 = 0x011,
    // MCP3208
    MCP320x_CH4 = 0x100,
    MCP320x_CH5 = 0x101,
    MCP320x_CH6 = 0x110,
    MCP320x_CH7 = 0x111,
} MCP320xChannel;

/// @brief Create a reference to the peripheral. `fd` exists iff result is OK.
/// @return result.
MCP320xResult mcp320x_init(int* fd_out);

/// @brief Get the 12 bit value for a specific channel.
/// @param fd the reference to the peripheral.
/// @param channel The channel index (0 upto 7 for MCP3208).
/// @param value_out The place to put the value.
/// @return result.
MCP320xResult mcp320x_get(int fd, MCP320xChannel channel, unsigned short* value_out);

/// @brief Clean up the peripheral reference.
/// @param fd
/// @return 
MCP320xResult mcp320x_cleanup(int fd);

#endif