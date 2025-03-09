use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use bounded::{
    BoundedValue,
    bounds::{DynamicU8, DynamicU16, DynamicU32, One, Zero},
};
use crate::misc::ContextError;


pub mod bounded;
pub mod fixture;
pub mod project;
pub mod misc;

pub type DynamicError = Box<dyn std::error::Error>;
pub type DynamicResult<T> = Result<T, DynamicError>;
pub type ContextResult<T> = Result<T, ContextError>;


pub type Percentage = BoundedValue<f32, Zero, One>;
pub type TrippleDMXValue = BoundedValue<
    u32,
    DynamicU32<{ DmxGranularity::Tripple.min() }>,
    DynamicU32<{ DmxGranularity::Tripple.max() }>,
>;
pub type SingleDMXValue = BoundedValue<
    u8,
    DynamicU8<{ DmxGranularity::Single.min() as u8 }>,
    DynamicU8<{ DmxGranularity::Single.max() as u8 }>,
>;
pub type DoubleDMXValue = BoundedValue<
    u16,
    DynamicU16<{ DmxGranularity::Double.min() as u16 }>,
    DynamicU16<{ DmxGranularity::Double.max() as u16 }>,
>;
pub type GenericDMXValue = TrippleDMXValue;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum DmxGranularity {
    Single,
    Double,
    Tripple,
}

impl DmxGranularity {
    pub const fn min(&self) -> u32 {
        0
    }

    pub const fn max(&self) -> u32 {
        match self {
            DmxGranularity::Single => u8::MAX as u32,
            DmxGranularity::Double => u16::MAX as u32,
            DmxGranularity::Tripple => 2_u32.pow(24) - 1,
        }
    }
}

pub trait PercentageDmxExt {
    fn from_gen_dmx(dmx: GenericDMXValue, granularity: DmxGranularity) -> Self;
    fn from_single_dmx(dmx: SingleDMXValue) -> Self;
    fn from_double_dmx(dmx: DoubleDMXValue) -> Self;
    fn from_tripple_dmx(dmx: TrippleDMXValue) -> Self;

    fn to_gen_dmx(&self, granularity: DmxGranularity) -> GenericDMXValue;
    fn to_single_dmx(&self) -> SingleDMXValue;
    fn to_double_dmx(&self) -> DoubleDMXValue;
    fn to_tripple_dmx(&self) -> TrippleDMXValue;
}

impl PercentageDmxExt for Percentage {
    fn from_gen_dmx(dmx: GenericDMXValue, granularity: DmxGranularity) -> Self {
        let val = dmx.take() as f32 / granularity.max() as f32;
        Self::create(val)
    }

    fn from_single_dmx(dmx: SingleDMXValue) -> Self {
        let val = dmx.take() as f32 / SingleDMXValue::max() as f32;
        Self::create(val)
    }

    fn from_double_dmx(dmx: DoubleDMXValue) -> Self {
        let val = dmx.take() as f32 / DoubleDMXValue::max() as f32;
        Self::create(val)
    }

    fn from_tripple_dmx(dmx: TrippleDMXValue) -> Self {
        let val = dmx.take() as f32 / TrippleDMXValue::max() as f32;
        Self::create(val)
    }

    fn to_gen_dmx(&self, granularity: DmxGranularity) -> GenericDMXValue {
        GenericDMXValue::create((self.take() * granularity.max() as f32) as u32)
    }

    fn to_single_dmx(&self) -> SingleDMXValue {
        SingleDMXValue::create((self.take() * SingleDMXValue::max() as f32) as u8)
    }

    fn to_double_dmx(&self) -> DoubleDMXValue {
        DoubleDMXValue::create((self.take() * DoubleDMXValue::max() as f32) as u16)
    }

    fn to_tripple_dmx(&self) -> TrippleDMXValue {
        TrippleDMXValue::create((self.take() * TrippleDMXValue::max() as f32) as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaybeLinear<T> where T: Debug + Clone {
    Constant(T),
    Linear{
        start: T,
        end: T,
    }
}