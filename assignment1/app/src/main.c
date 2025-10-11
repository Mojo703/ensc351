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
#define JOYSTICK_DEADZONE 500

// LED ready signal on time in milliseconds
#define READY_DELAY_MS 250

// Random pause length in milliseconds
#define PAUSE_MIN_MS 500
#define PAUSE_MAX_MS 3000

#define TIMEOUT_MS 5000

/// @brief Get the program time in milliseconds.
/// @return milliseconds.
long time_ms()
{
    struct timespec ts;
    timespec_get(&ts, TIME_UTC);
    return (long long)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
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

    do
    {
        res = nanosleep(&ts, &ts);
    } while (res && errno == EINTR);

    return res;
}

enum JoystickState
{
    JOYSTICK_UP,
    JOYSTICK_DOWN,
    JOYSTICK_LEFT,
    JOYSTICK_RIGHT,
    JOYSTICK_CENTER,
};

const char *get_JoystickState_name(enum JoystickState state)
{
    switch (state)
    {
    case JOYSTICK_UP:
        return "Up";
    case JOYSTICK_DOWN:
        return "Down";
    case JOYSTICK_LEFT:
        return "Left";
    case JOYSTICK_RIGHT:
        return "Right";
    case JOYSTICK_CENTER:
        return "Center";
    }
    return "Unknown";
}

enum JoystickState get_joystick(int adc)
{
    int sample_count = 8;

    u_int16_t x_pos;
    u_int16_t y_pos;
    mcp320x_get_median(adc, MCP320x_CH0, sample_count, &y_pos);
    mcp320x_get_median(adc, MCP320x_CH1, sample_count, &x_pos);

    int dx = 2048 - (int)x_pos;
    int dy = 2048 - (int)y_pos;

    if (abs(dx) > abs(dy))
    {
        if (dx > JOYSTICK_DEADZONE)
            return JOYSTICK_RIGHT;
        if (dx < -JOYSTICK_DEADZONE)
            return JOYSTICK_LEFT;
    }
    else
    {
        if (dy > JOYSTICK_DEADZONE)
            return JOYSTICK_DOWN;
        if (dy < -JOYSTICK_DEADZONE)
            return JOYSTICK_UP;
    }

    return JOYSTICK_CENTER;
}

/// @brief Run the reaction time loop.
/// @param adc
/// @param led_g
/// @param led_r
/// @param best_time A reference to the best reaction time so far.
/// @return `true` if the game should continue. `false` if the user chose to quit.
bool time_reaction(int adc, int led_g, int led_r, long *best_time)
{
    // Pick the random target
    enum JoystickState target;
    if (rand() % 2 == 0)
    {
        target = JOYSTICK_UP;
        printf("Press UP!\n");
        builtin_led_set_brightness(led_g, 1);
    }
    else
    {
        target = JOYSTICK_DOWN;
        printf("Press DOWN!\n");
        builtin_led_set_brightness(led_r, 1);
    }

    // Start the reaction time loop.
    long long start_time = time_ms();
    while (true)
    {
        enum JoystickState current = get_joystick(adc);

        long reaction_time = time_ms() - start_time;

        if (current == JOYSTICK_CENTER)
            continue;

        // Reset the LEDs
        builtin_led_set_brightness(led_g, 0);
        builtin_led_set_brightness(led_r, 0);

        printf("You pressed %s.\n", get_JoystickState_name(current));

        // Handle more than 5 secods delay, or left and right joysticks.
        if (reaction_time > TIMEOUT_MS)
        {
            printf("No reaction within 5000ms; quitting!\n");
            return false;
        }
        if (current == JOYSTICK_LEFT || current == JOYSTICK_RIGHT)
        {
            printf("User selected to quit.\n");
            return false;
        }

        if (current == target)
        {
            printf("Correct!\nYour reaction time was %ldms. ", reaction_time);

            if (*best_time == -1 || reaction_time < *best_time)
            {
                printf("You have set a new best time.\n");
                *best_time = reaction_time;
            }
            else
            {
                printf("Best so far was %ldms.\n", *best_time);
            }

            // Flash the green LED
            for (int j = 0; j < 5; j++)
            {
                builtin_led_set_brightness(led_g, 1);
                msleep(100);
                builtin_led_set_brightness(led_g, 0);
                msleep(100);
            }
        }
        else
        {
            printf("Incorrect.\n");

            // Flash the red LED
            for (int j = 0; j < 5; j++)
            {
                builtin_led_set_brightness(led_r, 1);
                msleep(100);
                builtin_led_set_brightness(led_r, 0);
                msleep(100);
            }
        }

        return true;
    }
}

/// @brief The game loop.
/// @param adc
/// @param led_g
/// @param led_r
void game(int adc, int led_g, int led_r)
{
    printf(WELCOME_MESSAGE);

    time_t best_time = -1;
    bool game_running = true;
    while (game_running)
    {
        printf("Get Ready...\n");

        for (int i = 0; i < 4; i++)
        {
            builtin_led_set_brightness(led_g, 1);
            msleep(READY_DELAY_MS);
            builtin_led_set_brightness(led_g, 0);
            builtin_led_set_brightness(led_r, 1);
            msleep(READY_DELAY_MS);
            builtin_led_set_brightness(led_r, 0);
        }

        // If necessary, tell the user to let go of the joystick
        if (get_joystick(adc) != JOYSTICK_CENTER)
            printf("Please let go of joystick.\n");

        // Wait for the user to let go of the joystick
        while (get_joystick(adc) != JOYSTICK_CENTER)
        {
            ;
        }

        // Pause for a random period.
        int pause_length_ms = (rand() % (PAUSE_MAX_MS - PAUSE_MIN_MS)) + PAUSE_MIN_MS;
        msleep(pause_length_ms);

        // If the user is holding the joystick, restart the game.
        if (get_joystick(adc) != JOYSTICK_CENTER)
        {
            printf("too soon.\n");
            continue;
        }

        game_running = time_reaction(adc, led_g, led_r, &best_time);
    }
}

void led_test(int led_g, int led_r)
{
    for (int index = 0; index < 20; index++)
    {
        builtin_led_set_brightness(led_r, index & 1);
        builtin_led_set_brightness(led_g, (index >> 1) & 1);
        msleep(300);
    }
}

void joystick_test(int adc)
{
    for (int index = 0; index < 30; index++)
    {
        unsigned short ch0, ch1;
        mcp320x_get(adc, 0, &ch0);
        mcp320x_get(adc, 1, &ch1);
        enum JoystickState state = get_joystick(adc);
        printf("CH0: %d, CH1: %d, Joystick: %s\n", ch0, ch1, get_JoystickState_name(state));
        msleep(300);
    }
}

int main()
{
    // Init the HAL
    int led_r;
    int led_g;
    if (builtin_led_init(BUILTIN_LED_RED, &led_r) != BUILTIN_LED_OK)
        return -1;
    if (builtin_led_init(BUILTIN_LED_GREEN, &led_g) != BUILTIN_LED_OK)
        return -1;

    int adc;
    if (mcp320x_init(&adc) != MCP320x_OK)
        return -1;

    // Init the Pseudo random numbers
    srand(time(NULL));

    // led_test(led_g, led_r);
    // joystick_test(adc);

    // Start the game
    game(adc, led_g, led_r);

    // Set the LEDs to Off
    builtin_led_set_brightness(led_r, 0);
    builtin_led_set_brightness(led_g, 0);

    // Cleanup the HAL
    builtin_led_cleanup(led_r);
    builtin_led_cleanup(led_g);
    mcp320x_cleanup(adc);

    return 0;
}
