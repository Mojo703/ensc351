
#ifndef _BUILTIN_LED_H_
#define _BUILTIN_LED_H_

#define BUILTIN_LED_RED_PATH "/sys/class/leds/PWR"
#define BUILTIN_LED_GREEN_PATH "/sys/class/leds/ACT"

typedef enum {
    BUILTIN_LED_RED,
    BUILTIN_LED_GREEN,
} BuiltinLED;

typedef enum {
    BUILTIN_LED_ERROR_OPEN = -2,
    BUILTIN_LED_ERROR_WRITE = -1,
    BUILTIN_LED_OK = 0;
} BuiltinLEDResult;

// void builtin_led_init();
BuiltinLEDResult builtin_led_set_brightness(BuiltinLED, int);
// void builtin_led_cleanup();

#endif