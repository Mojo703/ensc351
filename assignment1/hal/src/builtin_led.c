#include "hal/builtin_led.h"

BuiltinLEDResult builtin_led_set_brightness(BuiltinLED led, int brightness) {
    char* path;

    if (led == BUILTIN_LED_RED) {
        path = BUILTIN_LED_RED_PATH "/brightness";
    } else {
        path = BUILTIN_LED_GREEN_PATH "/brightness";
    }

    int fd = open(path, O_WRONLY);
    if (fd < 0) {
        perror("Could not open builtin LED peripheral.");
        return BUILTIN_LED_ERROR_OPEN;
    }

    char buf[4];
    int len = snprintf(buf, sizeof(buf), "%d", brightness);
    int write_len = write(fd, buf, len) != len;

    close(fd);

    if (len != write_len) {
        perror("Could not write to builtin LED peripheral.");
        return BUILTIN_LED_ERROR_WRITE;
    }

    return BUILTIN_LED_OK;
}
