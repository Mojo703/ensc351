use std::time::{Duration, Instant};

/**
 * Hardware interface for a button. With debounce and repeat events.
 */
/// Encoder direction pulse
#[derive(Debug, Clone, Copy)]
pub enum Event {
    Pressed,
    Repeat,
}

#[derive(Debug, Clone, Copy, Default)]
enum State {
    Pressed(Instant),
    Repeat(Instant),
    #[default]
    Up,
}

/// Quadrature encoder that is polled on a seperate thread.
pub struct Button {
    debounce: Duration,
    timeout: Duration,
    repeat_timeout: Duration,

    pin: gpiod::Lines<gpiod::Input>,

    state: State,
}

impl Button {
    /// Create an encoder that is polled on a seperate thread. It has a limited range of allowed values.
    pub fn new(
        pin: gpiod::Lines<gpiod::Input>,
        debounce: Duration,
        timeout: Duration,
        repeat_timeout: Duration,
    ) -> std::io::Result<Self> {
        Ok(Self {
            debounce,
            timeout,
            repeat_timeout,
            pin,
            state: State::default(),
        })
    }

    pub fn update(&mut self, now: Instant) -> Option<Event> {
        let pressed = self
            .pin
            .get_values([false])
            .expect("Button pins must be defined correclty for get_values.")[0];

        self.update_state(pressed, now)
    }

    fn update_state(&mut self, pressed: bool, now: Instant) -> Option<Event> {
        let (next, event) = match self.state {
            State::Up => {
                if pressed {
                    (State::Pressed(now), Some(Event::Pressed))
                } else {
                    (State::Up, None)
                }
            }
            State::Pressed(prev) => {
                let delta = now - prev;
                if pressed || delta < self.debounce {
                    if delta > self.timeout {
                        (State::Repeat(now), Some(Event::Pressed))
                    } else {
                        (State::Pressed(prev), None)
                    }
                } else {
                    (State::Up, None)
                }
            }
            State::Repeat(prev) => {
                let delta = now - prev;
                if pressed || delta < self.debounce {
                    if delta > self.repeat_timeout {
                        (State::Repeat(now), Some(Event::Repeat))
                    } else {
                        (State::Repeat(prev), None)
                    }
                } else {
                    (State::Up, None)
                }
            }
        };

        self.state = next;

        event
    }
}
