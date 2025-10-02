
#ifndef _BUILTIN_LED_H_
#define _BUILTIN_LED_H_

typedef enum {
    BUILTIN_LED_RED,
    BUILTIN_LED_GREEN,
} BuiltinLED;

void builtin_led_init();
void builtin_led_set_brightness(BuiltinLED, float);
void builtin_led_cleanup();

#endif