#include "hal/builtin_led.h"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

BuiltinLEDResult builtin_led_init(BuiltinLED led, int* fd_out) {
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

    *fd_out = fd;

    return BUILTIN_LED_OK;
}

BuiltinLEDResult builtin_led_set_brightness(int fd, int brightness) {
    char buf[4];
    int len = snprintf(buf, sizeof(buf), "%d", brightness);

    if (write(fd, buf, len) != len) {
        perror("Could not write to builtin LED peripheral.");
        return BUILTIN_LED_ERROR_WRITE;
    }

    return BUILTIN_LED_OK;
}

BuiltinLEDResult builtin_led_cleanup(int fd) {
    if (close(fd) < 0) {
        return BUILTIN_LED_ERROR_CLOSE;
    }
    
    return BUILTIN_LED_OK;
}