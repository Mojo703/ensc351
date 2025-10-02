
#ifndef _MCP3204_H_
#define _MCP3204_H_

#define MCP3204_SPI_FREQUENCY 1000000
#define MCP3204_BITS_PER_WORD 8

typedef enum {
    MCP3204_SPI_ERROR = -1,
    MCP3204_OK = 0,
} MCP3204Result

// void mcp3204_init(void);
MCP3204Result mcp3204_get(int*);
// void mcp3204_cleanup(void);

#endif