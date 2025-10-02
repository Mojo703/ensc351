
#ifndef _BUILTIN_LED_H_
#define _BUILTIN_LED_H_

#define BUILTIN_LED_RED_PATH "/sys/class/leds/PWR"
#define BUILTIN_LED_GREEN_PATH "/sys/class/leds/ACT"

typedef enum {
    BUILTIN_LED_RED,
    BUILTIN_LED_GREEN,
} BuiltinLED;

typedef enum {
    BUILTIN_LED_ERROR_OPEN = -3,
    BUILTIN_LED_ERROR_CLOSE = -2,
    BUILTIN_LED_ERROR_WRITE = -1,
    BUILTIN_LED_OK = 0
} BuiltinLEDResult;

BuiltinLEDResult builtin_led_init(BuiltinLED led, int* fd_out);
BuiltinLEDResult builtin_led_set_brightness(int fd, int brightness);
BuiltinLEDResult builtin_led_cleanup(int fd);

#endif