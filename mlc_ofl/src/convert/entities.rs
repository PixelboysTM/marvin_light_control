use serde_json::Value;
use mlc_data::{err, ContextResult, misc::ContextError, Percentage};
use mlc_data::fixture::blueprint::{entities::{Distance, FogKind, FogOutput, HorizontalAngle}, units::SignedPercentage};
use mlc_data::fixture::blueprint::entities::{BeamAngle, Brightness, Color, ColorTemperature, DynamicColor, IrisPercent, Parameter, Preset, RotationAngle, RotationSpeed, ShutterEffect, Speed, Time, VerticalAngle};
use crate::convert::parse_helpers::ParseExecutorValue;
use crate::convert::parseable::{SimpleParseable};

fn s_zero() -> SignedPercentage { SignedPercentage::create(0.0) }
fn s_one() -> SignedPercentage { SignedPercentage::create(0.01) }
fn s_hundred() -> SignedPercentage { SignedPercentage::create(1.0) }
fn s_neg_one() -> SignedPercentage { SignedPercentage::create(-0.01) }
fn s_neg_hundred() -> SignedPercentage { SignedPercentage::create(-1.0) }

fn zero() -> Percentage { Percentage::create(0.0) }
fn one() -> Percentage { Percentage::create(0.01) }
fn hundred() -> Percentage { Percentage::create(1.0) }

impl SimpleParseable for FogOutput {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("FogOutput must be a string"))?;

        if s == "off" { Ok(FogOutput::Percentage(zero())) }
        else if s == "weak" { Ok(FogOutput::Percentage(one())) }
        else if s == "strong" { Ok(FogOutput::Percentage(hundred()))}
        else if let Ok(vpm) = value.parse() { Ok(FogOutput::VolumePerMinute(vpm))}
        else if let Ok(p) = value.parse() { Ok(FogOutput::Percentage(p))}
        else { Err(err!("FogOutput can't be parsed")) }
    }
}

impl SimpleParseable for FogKind {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("FogKind must be a string"))?;
        match s {
            "Fog" => Ok(FogKind::Fog),
            "Haze" => Ok(FogKind::Haze),
            _ => Err(err!("FogKind can't be parsed"))
        }
    }
}

impl SimpleParseable for Distance {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Distance must be a string"))?;

        if s == "near" { Ok(Distance::Percentage(s_one())) }
        else if s == "far" { Ok(Distance::Percentage(s_hundred())) }
        else if let Ok(m) = value.parse() { Ok(Distance::Meters(m)) }
        else if let Ok(p) = value.parse() { Ok(Distance::Percentage(p)) }
        else { Err(err!("Distance can't be parsed"))}
    }
}

impl SimpleParseable for HorizontalAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("HorizontalAngle must be a string"))?;

        if s == "left" { Ok(HorizontalAngle::Percentage(s_neg_hundred()))}
        else if s == "right" { Ok(HorizontalAngle::Percentage(s_hundred()))}
        else if s == "center" { Ok(HorizontalAngle::Percentage(s_zero()))}
        else if let Ok(deg) = value.parse() { Ok(HorizontalAngle::Degrees(deg))}
        else if let Ok(p) = value.parse() { Ok(HorizontalAngle::Percentage(p))}
        else { Err(err!("HorizontalAngle can't be parsed"))}
    }
}

impl SimpleParseable for VerticalAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("VerticalAngle must be a string"))?;

        if s == "top" { Ok(VerticalAngle::Percentage(s_neg_hundred()))}
        else if s == "bottom" { Ok(VerticalAngle::Percentage(s_hundred()))}
        else if s == "center" { Ok(VerticalAngle::Percentage(s_zero()))}
        else if let Ok(deg) = value.parse() { Ok(VerticalAngle::Degrees(deg))}
        else if let Ok(p) = value.parse() { Ok(VerticalAngle::Percentage(p))}
        else { Err(err!("VerticalAngle can't be parsed"))}
    }
}

impl SimpleParseable for BeamAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("BeamAngle must be a string"))?;

        if s == "closed" { Ok(BeamAngle::Percentage(zero())) }
        else if s == "narrow" { Ok(BeamAngle::Percentage(one())) }
        else if s == "wide" { Ok(BeamAngle::Percentage(hundred())) }
        else if let Ok(deg) = value.parse() { Ok(BeamAngle::Degrees(deg)) }
        else if let Ok(p) = value.parse() { Ok(BeamAngle::Percentage(p)) }
        else { Err(err!("BeamAngle can't be parsed"))}
    }
}

impl SimpleParseable for Parameter {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        if value.is_number() {
            Ok(Parameter::Number(value.as_f64().map(|n| n as f32).expect("Tested that it is a number")))
        } else {
            let s = value.as_str().ok_or(err!("Parameter must be a string if it is not a number"))?;

            if s == "off" { Ok(Parameter::Percentage(s_zero()))}
            else if s == "low" { Ok(Parameter::Percentage(s_one()))}
            else if s == "high" { Ok(Parameter::Percentage(s_hundred()))}
            else if s == "slow" { Ok(Parameter::Percentage(s_one()))}
            else if s == "fast" { Ok(Parameter::Percentage(s_hundred()))}
            else if s == "small" { Ok(Parameter::Percentage(s_one()))}
            else if s == "big" { Ok(Parameter::Percentage(s_hundred()))}
            else if s == "instant" { Ok(Parameter::Percentage(s_zero()))}
            else if s == "short" { Ok(Parameter::Percentage(s_one()))}
            else if s == "long" { Ok(Parameter::Percentage(s_hundred()))}
            else if let Ok(p) = value.parse(){ Ok(Parameter::Percentage(p))}
            else { Err(err!("Parameter can't be parsed: {}", value))}
        }
    }
}

impl SimpleParseable for RotationAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        if let Ok(deg) = value.parse() { Ok(RotationAngle::Degrees(deg)) }
        else if let Ok(p) = value.parse() { Ok(RotationAngle::Percent(p))}
        else { Err(err!("RotationAngle can't be parsed: {}", value))}
    }
}

impl SimpleParseable for RotationSpeed {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("RotationSpeed must be a string"))?;

        if s == "fast CW" { Ok(RotationSpeed::Percent(s_hundred()))  }
        else if s == "slow CW" { Ok(RotationSpeed::Percent(s_one()))  }
        else if s == "stop" { Ok(RotationSpeed::Percent(s_zero()))  }
        else if s == "slow CCW" { Ok(RotationSpeed::Percent(s_neg_one()))  }
        else if s == "fast CCW" { Ok(RotationSpeed::Percent(s_neg_hundred()))  }
        else if let Ok(hertz) = value.parse() { Ok(RotationSpeed::Hz(hertz))  }
        else if let Ok(rpm) = value.parse() { Ok(RotationSpeed::RPM(rpm))  }
        else if let Ok(p) = value.parse() { Ok(RotationSpeed::Percent(p))  }
        else { Err(err!("RotationSpeed can't be parsed"))}
    }
}

impl SimpleParseable for ColorTemperature {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("ColorTemperature must be a string"))?;

        if s == "warm" || s == "CTO" { Ok(ColorTemperature::Percent(s_neg_hundred()))}
        else if s == "default" { Ok(ColorTemperature::Percent(s_zero()))}
        else if s == "cold" || s == "CTB" { Ok(ColorTemperature::Percent(s_hundred()))}
        else if let Ok(kelvin) = value.parse() { Ok(ColorTemperature::Kelvin(kelvin))}
        else if let Ok(p) = value.parse() { Ok(ColorTemperature::Percent(p))}
        else { Err(err!("ColorTemperature can't be parsed"))}
    }
}

impl SimpleParseable for Color {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Color must be a string"))?;

        match s {
            "Red" => Ok(Color::Red),
            "Green" => Ok(Color::Green),
            "Blue" => Ok(Color::Blue),
            "Cyan" => Ok(Color::Cyan),
            "Magenta" => Ok(Color::Magenta),
            "Yellow" => Ok(Color::Yellow),
            "Amber" => Ok(Color::Amber),
            "White" => Ok(Color::White),
            "Warm White" => Ok(Color::WarmWhite),
            "Cold White" => Ok(Color::ColdWhite),
            "UV" => Ok(Color::UV),
            "Lime" => Ok(Color::Lime),
            "Indigo" => Ok(Color::Indigo),
            _ => Err(err!("Color can't be parsed"))
        }
    }
}

impl SimpleParseable for DynamicColor {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("DynamicColor must be a string"))?;
        if !s.starts_with("#") ||s.len() != 7 { return Err(err!("DynamicColor is not a hex string"));}
        let r = u8::from_str_radix(&s[1..=2], 16).map_err(|e| err!(e))?;
        let g = u8::from_str_radix(&s[3..=4], 16).map_err(|e| err!(e))?;
        let b = u8::from_str_radix(&s[5..=6], 16).map_err(|e| err!(e))?;
        Ok(DynamicColor{
            r,
            g,
            b
        })
    }
}

impl SimpleParseable for Brightness {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Brightness must be a string"))?;

        if s == "off" { Ok(Brightness::Percent(zero()))}
        else if s == "dark" { Ok(Brightness::Percent(one()))}
        else if s == "bright" { Ok(Brightness::Percent(hundred()))}
        else if let Ok(lm) = value.parse() { Ok(Brightness::Lumen(lm)) }
        else if let Ok(p) = value.parse() { Ok(Brightness::Percent(p))}
        else { Err(err!("Brightness can't be parsed"))}
    }
}

impl SimpleParseable for Time {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Time must be a string"))?;

        if s == "instant" { Ok(Time::Percent(zero())) }
        else if s == "short" { Ok(Time::Percent(one()))}
        else if s == "long" { Ok(Time::Percent(hundred()))}
        else if let Ok(ms) = value.parse() { Ok(Time::Milliseconds(ms))}
        else if let Ok(secs) = value.parse() { Ok(Time::Seconds(secs))}
        else if let Ok(p) = value.parse() { Ok(Time::Percent(p))}
        else { Err(err!("Time can't be parsed"))}
    }
}

impl SimpleParseable for Speed {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Speed must be a string"))?;

        if s == "fast" { Ok(Speed::Percent(s_hundred())) }
        else if s == "slow" { Ok(Speed::Percent(s_one()))}
        else if s == "stop" { Ok(Speed::Percent(s_zero()))}
        else if s == "slow reverse" { Ok(Speed::Percent(s_neg_one()))}
        else if s == "fast reverse" { Ok(Speed::Percent(s_neg_hundred()))}
        else if let Ok(hertz) = value.parse() { Ok(Speed::Hz(hertz))}
        else if let Ok(bpm) = value.parse() { Ok(Speed::Bpm(bpm))}
        else if let Ok(p) = value.parse() { Ok(Speed::Percent(p))}
        else { Err(err!("Speed can't be parsed"))}
    }
}

impl SimpleParseable for ShutterEffect {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("shutterEffect must be a string"))?;
        match s {
            "Open" => Ok(ShutterEffect::Open),
            "Closed" => Ok(ShutterEffect::Closed),
            "Strobe" => Ok(ShutterEffect::Strobe),
            "Pulse" => Ok(ShutterEffect::Pulse),
            "RampUp" => Ok(ShutterEffect::RampUp),
            "RampDown" => Ok(ShutterEffect::RampDown),
            "RampUpDown" => Ok(ShutterEffect::RampUpDown),
            "Lightning" => Ok(ShutterEffect::Lightning),
            "Spikes" => Ok(ShutterEffect::Spikes),
            "Burst" => Ok(ShutterEffect::Burst),
            _ => Err(err!("shutterEffect can't be parsed"))
        }
    }
}

impl SimpleParseable for Preset {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Preset must be a string"))?;
        match s {
            "ColorJump" => Ok(Preset::ColorJump),
            "ColorFade" => Ok(Preset::ColorFade),
            _ => Err(err!("Preset can't be parsed"))
        }
    }
}

impl SimpleParseable for IrisPercent {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("IrisPercent must be a string"))?;

        if s == "open" { Ok(IrisPercent(hundred()))}
        else if s == "closed" { Ok(IrisPercent(zero()))}
        else if let Ok(p) = value.parse() { Ok(IrisPercent(p))}
        else { Err(err!("IrisPercent can't be parsed"))}
    }
}