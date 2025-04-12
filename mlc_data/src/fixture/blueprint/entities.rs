use serde::{Deserialize, Serialize};
use super::units::{Degree, Hz, Kelvin, Lumen, Meters, MilliSeconds, Seconds, VolumePerMin, BPM, RPM, Percentage};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FogOutput {
    VolumePerMinute(VolumePerMin),
    Percentage(Percentage),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FogKind {
    Fog,
    Haze
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Distance {
    Meters(Meters),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HorizontalAngle {
    Degrees(Degree),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerticalAngle {
    Degrees(Degree),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BeamAngle {
    Degrees(Degree),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Parameter {
    Number(f32),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RotationAngle {
    Degrees(Degree),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RotationSpeed {
    Hz(Hz),
    RPM(RPM),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColorTemperature {
    Kelvin(Kelvin),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
    Amber,
    White,
    WarmWhite,
    ColdWhite,
    UV,
    Lime,
    Indigo
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Brightness {
    Lumen(Lumen),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Time {
    Seconds(Seconds),
    Milliseconds(MilliSeconds),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Speed {
    Hz(Hz),
    Bpm(BPM),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShutterEffect {
    Open,
    Closed,
    Strobe,
    Pulse,
    RampUp,
    RampDown,
    RampUpDown,
    Lightning,
    Spikes,
    Burst
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Preset {
    ColorJump,
    ColorFade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IrisPercent(pub Percentage);