use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume(f32);

impl Volume {
    /// Returns volume as f32 in [0.0, 100.0]
    pub fn as_percentage(self) -> f32 {
        self.0
    }

    /// Returns volume as f32 in [0.0, 1.0]
    pub fn as_scale(self) -> f32 {
        self.as_percentage() / 100.0
    }
}

impl TryFrom<u32> for Volume {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let value = value as f32;
        (0.0..=100.0)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(())
    }
}

impl From<Volume> for u32 {
    fn from(value: Volume) -> Self {
        value.0.round() as u32
    }
}

/// Allow controlling the Volume with joystick
impl std::ops::Add<f32> for Volume {
    type Output = Volume;

    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bpm(f64);

impl TryFrom<u32> for Bpm {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let value = value as f64;
        (40.0..=300.0)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(())
    }
}

impl From<Bpm> for f64 {
    fn from(value: Bpm) -> Self {
        value.0
    }
}

impl From<Bpm> for u32 {
    fn from(value: Bpm) -> Self {
        value.0.round() as u32
    }
}

/// Allow accumulating the encoder values.
impl std::ops::Add<i32> for Bpm {
    type Output = Bpm;

    fn add(self, rhs: i32) -> Self::Output {
        let rhs = rhs as f64;
        Bpm(self.0 + rhs)
    }
}

impl Display for Bpm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bpm", self.0)
    }
}
