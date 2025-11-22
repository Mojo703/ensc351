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

    /// Allow controlling the Volume with joystick
    pub fn saturating_add(self, value: f32) -> Self {
        Self((self.0 + value).clamp(0.0, 100.0))
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

impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vol:{}%", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bpm(f64);

impl Bpm {
    const MIN: f64 = 40.0;
    const MAX: f64 = 300.0;

    /// Allow accumulating the encoder values.
    pub fn saturating_add(self, value: f64) -> Self {
        Self((self.0 + value).clamp(Self::MIN, Self::MAX))
    }
}

impl TryFrom<u32> for Bpm {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let value = value as f64;
        (Bpm::MIN..=Bpm::MAX)
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

impl Display for Bpm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}bpm", self.0)
    }
}
