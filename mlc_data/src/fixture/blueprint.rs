use std::collections::HashMap;
use std::ops::RangeInclusive;
use serde::{Deserialize, Serialize};
use crate::{MaybeLinear, Percentage};
use entities::*;

pub mod entities;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureBlueprint {
    #[serde(flatten)]
    pub meta: Metadata,
    pub channels: HashMap<ChannelIdentifier, Channel>,
    pub modes: Vec<Mode>,
    pub matrix: Option<PixelMatrix>,
    pub wheels: Option<Vec<()>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub identifier: String,
    pub manufacturer: String,
    pub physical: Physical,
}

pub type PixelGroupIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelMatrix {
    pub matrix: Vec<Vec<Vec<Option<Vec<PixelGroupIdentifier>>>>>
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PixelIdentifier {
    Pixel(usize, usize, usize),
    Group(PixelGroupIdentifier),
    #[default]
    Master
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Physical {
     /// width, height, depth (in mm)
    pub dimensions: Option<[f32; 3]>,
     /// in kg
    pub weight: f32,
    /// in Watt
    pub power_consumption: f32,
    pub power_connectors: String,
    pub dmx_connector: String,
    pub bulb: String,
    pub lens: String,
}

pub type ChannelIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Channel {
    Single {
        #[serde(flatten)]
        channel: CommonChannel,
    },
    Double {
        #[serde(flatten)]
        channel: CommonChannel,
        second_channel_name: ChannelIdentifier,
    },
    Tripple {
        #[serde(flatten)]
        channel: CommonChannel,
        second_channel_name: ChannelIdentifier,
        third_channel_name: ChannelIdentifier,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonChannel {
    #[serde(default = "default_percentage")]
    default_value: Percentage,
    capabilities: Vec<Capability>
}

fn default_percentage() -> Percentage {
    Percentage::create(0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    range: RangeInclusive<Percentage>,
    #[serde(default)]
    pixel: PixelIdentifier,
    comment: Option<String>,
    #[serde(flatten)]
    kind: CapabilityKind,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    NoFunction,
    Generic,
    ShutterStrobe {
        effect: ShutterEffect,
        sound_controlled: bool,
        speed: Option<MaybeLinear<Speed>>,
        duration: Option<MaybeLinear<Time>>,
        random_timing: bool,
    },
    StrobeSpeed {
        speed: MaybeLinear<Speed>,
    },
    StrobeDuration {
        duration: MaybeLinear<Time>,
    },
    Intensity {
        brightness: MaybeLinear<Brightness>,
    },
    ColorIntensity {
        brightness: MaybeLinear<Brightness>,
        color: Color,
    },
    ColorPreset {
        colors: Vec<MaybeLinear<DynamicColor>>,
        color_temperature: Option<MaybeLinear<ColorTemperature>>
    },
    ColorTemperature {
        temperature: MaybeLinear<ColorTemperature>,
    },
    Pan {
        angle: MaybeLinear<RotationAngle>
    },
    PanContinuous {
        speed: MaybeLinear<RotationSpeed>,
    },
    Tilt {
        angle: MaybeLinear<RotationAngle>
    },
    TiltContinuous {
        speed: MaybeLinear<RotationSpeed>,
    },
    PanTiltSpeed {
        speed: MaybeLinear<Speed>,
        duration: Option<MaybeLinear<Time>>
    },
    WheelSlot {
        wheel: String,
        slot_number: MaybeLinear<u32>,
    },
    //TODO: Implement
    WheelShake,
    WheelSlotRotation,
    WheelRotation,
    Effect {
        name: String,
        preset: String,
        speed: Option<MaybeLinear<Speed>>,
        duration: Option<MaybeLinear<Time>>,
        parameter: Option<MaybeLinear<Parameter>>,
        sound_controlled: bool,
        sound_sensitivity: Option<MaybeLinear<Percentage>>,
    },
    EffectSpeed {
        speed: MaybeLinear<Speed>,
    },
    EffectDuration {
        duration: MaybeLinear<Time>,
    },
    EffectParameter {
        parameter: MaybeLinear<Parameter>,
    },
    SoundSensitivity {
        sensitivity: MaybeLinear<Percentage>,
    },
    BeamAngle {
        angle: MaybeLinear<BeamAngle>
    },
    BeamPosition {
        horizontal_angle: Option<MaybeLinear<HorizontalAngle>>,
        vertical_angle: Option<MaybeLinear<VerticalAngle>>,
    },
    Focus {
        distance: MaybeLinear<Distance>,
    },
    Zoom {
        angle: MaybeLinear<BeamAngle>
    },
    Iris {
        open_percent: MaybeLinear<Percentage>,
    },
    IrisEffect {
        name: String,
        speed: Option<MaybeLinear<Speed>>
    },
    Frost {
        intensity: MaybeLinear<Percentage>,
    },
    FrostEffect {
        name: String,
        speed: Option<MaybeLinear<Speed>>
    },
    Prism {
        speed: Option<MaybeLinear<RotationSpeed>>,
        angle: Option<MaybeLinear<RotationAngle>>
    },
    PrismRotation {
        speed: Option<MaybeLinear<RotationSpeed>>,
        angle: Option<MaybeLinear<RotationAngle>>
    },
    //TODO: Implement
    BladeInsertion,
    BladeRotation,
    BladeSystemRotation,
    Fog {
        kind: FogKind,
        output: Option<MaybeLinear<FogOutput>>,
    },
    FogOutput {
        output: MaybeLinear<FogOutput>,
    },
    FogType {
        kind: FogKind
    },
    Rotation {
       speed: Option<MaybeLinear<RotationSpeed>>,
        angle: Option<MaybeLinear<RotationAngle>>,
    },
    Speed {
        speed: MaybeLinear<Speed>,
    },
    Time {
        time: MaybeLinear<Time>,
    },
    Maintenance {
        parameter: Option<MaybeLinear<Parameter>>,
        hold: Option<Time>
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    name: String,
    channel: Vec<ChannelIdentifier>,
}
