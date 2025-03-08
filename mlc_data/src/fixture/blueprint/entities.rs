use serde::{Deserialize, Serialize};
use crate::Percentage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FogOutput {
    VolumePerMinute(f32),
    Percentage(Percentage),
    Off,
    Weak,
    Strong,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FogKind {
    Fog,
    Haze
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Distance {
    Meters(f32),
    Percentage(Percentage),
    Near,
    Far,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HorizontalAngle {
    Degrees(f32),
    Percentage(Percentage),
    Left,
    Center,
    Right
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerticalAngle {
    Degrees(f32),
    Percentage(Percentage),
    Top,
    Center,
    Bottom
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeamAngle {
    Degrees(f32),
    Percentage(Percentage),
    Closed,
    Narrow,
    Wide
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parameter {
    Number(f32),
    Percentage(Percentage),
    Off,
    Low,
    High,
    Slow,
    Fast,
    Small,
    Big,
    Instant,
    Short,
    Long
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationAngle {
    Degrees(f32),
    Percent(Percentage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationSpeed {
    Hertz(f32),
    RPM(f32),
    Percent(Percentage),
    FastCW,
    SlowCW,
    Stop,
    SlowCCW,
    FastCCW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorTemperature {
    Kelvin(f32),
    Percent(Percentage),
    Warm,
    CTO,
    Default,
    Cold,
    CTB
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
    Lumen(f32),
    Percent(Percentage),
    Off,
    Dark,
    Bright
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Time {
    Seconds(f32),
    Milliseconds(f32),
    Percent(Percentage),
    Instant,
    Short,
    Long
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Speed {
    Hertz(f32),
    Bpm(f32),
    Percent(Percentage),
    Fast,
    Slow,
    Stop,
    SlowReverse,
    FastReverse,
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