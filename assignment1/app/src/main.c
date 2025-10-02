// Main program to build the application
// Has main(); does initialization and cleanup and perhaps some basic logic.

#include <stdio.h>
#include <stdbool.h>
#include <time.h>
#include <stdlib.h>
#include <errno.h>
#include "hal/builtin_led.h"
#include "hal/mcp320x.h"

#define WELCOME_MESSAGE "Get ready for the reaction time game. Wait for the signal, and press up or down on the joystick.\n(Press left or right to exit)\n"
#define JOYSTICK_DEADZONE 200

// LED ready signal on time in milliseconds
#define READY_DELAY_MS 250

// Random pause length in milliseconds
#define PAUSE_MIN_MS 500
#define PAUSE_MAX_MS 3000

#define TIMEOUT_MS 50000

enum JoystickState {
    JOYSTICK_UP,
    JOYSTICK_DOWN,
    JOYSTICK_LEFT,
    JOYSTICK_RIGHT,
    JOYSTICK_CENTER,
};

enum JoystickState get_joystick(int adc) {
    u_int16_t x_pos;
    u_int16_t y_pos;
    mcp320x_get(adc, MCP320x_CH0, &x_pos);
    mcp320x_get(adc, MCP320x_CH1, &y_pos);

    int dx = 2048 - (int)x_pos;
    int dy = 2048 - (int)y_pos;

    if (abs(dx) > abs(dy)) {
        if (dx > JOYSTICK_DEADZONE) {
            return JOYSTICK_LEFT;
        }
        if (dx < -JOYSTICK_DEADZONE) {
            return JOYSTICK_RIGHT;
        }
    } else {
        if (dy > JOYSTICK_DEADZONE) {
            return JOYSTICK_UP;
        }
        if (dy < -JOYSTICK_DEADZONE) {
            return JOYSTICK_DOWN;
        }
    }

    return JOYSTICK_CENTER;
}


/// @brief Sleep for a specified number of milliseconds.
/// @param msec number of milliseconds.
/// @return status.
int msleep(long msec)
{
    struct timespec ts;
    int res;

    if (msec < 0)
    {
        errno = EINVAL;
        return -1;
    }

    ts.tv_sec = msec / 1000;
    ts.tv_nsec = (msec % 1000) * 1000000;

    do {
        res = nanosleep(&ts, &ts);
    } while (res && errno == EINTR);

    return res;
}

int main()
{
    // Init the HAL
    int led_r;
    int led_g;
    if (builtin_led_init(BUILTIN_LED_RED, &led_r) != BUILTIN_LED_OK) return -1;
    if (builtin_led_init(BUILTIN_LED_GREEN, &led_g) != BUILTIN_LED_OK) return -1;

    int adc;
    if (mcp320x_init(&adc) != MCP320x_OK) return -1;

    // Init the Pseudo random numbers
    srand(time(NULL));

    time_t best_time = -1;
    
    // Start the game
    printf(WELCOME_MESSAGE);

    bool game_running = true;
    while (game_running) {
        printf("Get Ready...\n");

        for (int i = 0; i < 4; i ++) {
            builtin_led_set_brightness(led_g, 1);
            msleep(READY_DELAY_MS);
            builtin_led_set_brightness(led_g, 0);
            builtin_led_set_brightness(led_r, 1);
            msleep(READY_DELAY_MS);
            builtin_led_set_brightness(led_r, 0);
        }

        // If necessary, tell the user to let go of the joystick
        if (get_joystick(adc) != JOYSTICK_CENTER) printf("Please let go of joystick.\n");

        // Wait for the user to let go of the joystick
        while (get_joystick(adc) != JOYSTICK_CENTER) { ; }

        // Pause for a random period.
        int pause_length_ms = (rand() % (PAUSE_MAX_MS - PAUSE_MIN_MS)) + PAUSE_MIN_MS;
        msleep(pause_length_ms);

        // If the user is holding the joystick, restart the game.
        if (get_joystick(adc) != JOYSTICK_CENTER) {
            printf("too soon.");
            continue;
        }

        // Pick the random target
        enum JoystickState target;
        if (rand() % 2 == 0){
            target = JOYSTICK_UP;
            printf("Press UP!");
            builtin_led_set_brightness(led_g, 1);
        } else {
            target = JOYSTICK_DOWN;
            printf("Press DOWN!");
            builtin_led_set_brightness(led_r, 1);
        }

        // Start the reaction time loop.
        time_t start_time = time(NULL);
        while (true) {
            enum JoystickState current = get_joystick(adc);

            time_t reaction_time = time(NULL) - start_time;
            
            if (current == JOYSTICK_CENTER) continue;
            
            // Handle more than 5 secods delay, or left and right joysticks.
            if (reaction_time > TIMEOUT_MS || current == JOYSTICK_LEFT || current == JOYSTICK_RIGHT) {
                printf("User selected to quit.");
                game_running = false;
                break;
            }

            if (current == target) {
                printf("Correct!\n  Your reaction time was %ldms. ", reaction_time);

                if (best_time == -1 || reaction_time < best_time) {
                    printf("You have set a new best time.\n");
                } else {
                    printf("Best so far was %ldms.\n", best_time);
                }

                // Flash the green LED
                for (int j = 0; j < 5; j ++) {
                    builtin_led_set_brightness(led_g, 1);
                    msleep(100);
                    builtin_led_set_brightness(led_g, 0);
                    msleep(100);
                }
            } else {
                printf("Incorrect.\n");

                // Flash the red LED
                for (int j = 0; j < 5; j ++) {
                    builtin_led_set_brightness(led_r, 1);
                    msleep(100);
                    builtin_led_set_brightness(led_r, 0);
                    msleep(100);
                }
            }
        }
    }

    // Set the LEDs to Off
    builtin_led_set_brightness(led_r, 0);
    builtin_led_set_brightness(led_g, 0);

    // Cleanup the HAL
    builtin_led_cleanup(led_r);
    builtin_led_cleanup(led_g);
    mcp320x_cleanup(adc);

    return 0;
}
