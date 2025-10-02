// Main program to build the application
// Has main(); does initialization and cleanup and perhaps some basic logic.

#include <stdio.h>
#include <stdbool.h>
#include "hal/builtin_led.h"
#include "hal/mcp320x.h"

int main()
{
    // Init the HAL
    int led_r;
    int led_g;
    if (builtin_led_init(BUILTIN_LED_RED, &led_r) != BUILTIN_LED_OK) return -1;
    if (builtin_led_init(BUILTIN_LED_GREEN, &led_g) != BUILTIN_LED_OK) return -1;

    int adc;
    if (mcp3204_init(&adc) != MCP320x_OK) return -1;


    // Cleanup the HAL
    builtin_led_cleanup(led_r);
    builtin_led_cleanup(led_g);
    mcp3204_cleanup(adc);
}