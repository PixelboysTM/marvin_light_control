use serde::{Deserialize, Serialize};
use super::units::{Degree, Hz, Kelvin, Lumen, Meters, MilliSeconds, SignedPercentage, Seconds, VolumePerMin, BPM, RPM, Percentage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FogOutput {
    VolumePerMinute(VolumePerMin),
    Percentage(Percentage),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FogKind {
    Fog,
    Haze
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Distance {
    Meters(Meters),
    Percentage(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HorizontalAngle {
    Degrees(Degree),
    Percentage(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerticalAngle {
    Degrees(Degree),
    Percentage(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeamAngle {
    Degrees(Degree),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parameter {
    Number(f32),
    Percentage(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationAngle {
    Degrees(Degree),
    Percent(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationSpeed {
    Hz(Hz),
    RPM(RPM),
    Percent(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorTemperature {
    Kelvin(Kelvin),
    Percent(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Brightness {
    Lumen(Lumen),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Time {
    Seconds(Seconds),
    Milliseconds(MilliSeconds),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Speed {
    Hz(Hz),
    Bpm(BPM),
    Percent(SignedPercentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Preset {
    ColorJump,
    ColorFade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisPercent(pub Percentage);