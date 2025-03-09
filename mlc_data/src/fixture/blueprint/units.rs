use serde::{Deserialize, Serialize};
use crate::bounded::BoundedValue;
use crate::bounded::bounds::{NegOne, One};

pub type SignedPercentage = BoundedValue<f32, NegOne, One>;
pub use crate::Percentage; // TODO: Merge into just an f32

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hz(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BPM(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RPM(pub f32);


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seconds(pub f32);


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilliSeconds(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Meters(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Lumen(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Kelvin(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VolumePerMin(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Degree(pub f32);