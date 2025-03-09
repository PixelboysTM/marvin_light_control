use std::fmt::Debug;
use either::Either;
use serde_json::{Map, Value};
use mlc_data::{err, misc::ContextError, ContextResult, MaybeLinear, Percentage};
use mlc_data::fixture::blueprint::entities::{BeamAngle, Brightness, Color, ColorTemperature, Distance, DynamicColor, FogKind, FogOutput, HorizontalAngle, Parameter, Preset, RotationAngle, RotationSpeed, ShutterEffect, Speed, Time, VerticalAngle};
use crate::convert::{ Parseable};
use crate::convert::parse_helpers::{ParseExecutorObj, ParseExecutorValue};
//TODO: Extract unit parsers

impl Parseable for bool {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value.as_bool().ok_or(err!("Value must be a bool"))
    }
}

impl Parseable for String {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value.as_str().ok_or(err!("Value must be a string")).map(|s| s.to_string())
    }
}

impl Parseable for f32 {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value.as_f64().ok_or(err!("Value must be a float")).map(|f| f as f32 )
    }
}

impl Parseable for Value {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        Ok(value.clone())
    }
}

impl<T> Parseable for Option<T> where T: Parseable
{
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        if value.is_null() {
            Ok(None)
        } else {
            Some(T::parse_from_value(value)).transpose()
        }

    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        if obj.contains_key(key) {
            Some(T::parse_from_value(&obj[key])).transpose()
        } else {
            Ok(None)
        }
    }
}

impl<T> Parseable for Option<MaybeLinear<T>> where T: Parseable + Debug + Clone {
    fn parse_from_value(_: &Value) -> ContextResult<Self> {
        Err(err!("MaybeLinear can't parsed from a single value, must be an object"))
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        if let Some (obj) = obj.get(key) {
            Ok(Some(MaybeLinear::Constant(T::parse_from_value(obj)?)))
        } else if let Some (s_obj) = obj.get(&format!("{}Start", key)) {
            let start = T::parse_from_value(s_obj)?;
            let end = T::parse_from_value( obj.get(&format!("{}End", key)).ok_or(err!("if Start is present also End must be there. Key: {key}, Obj: {obj:?}"))?)?;
            Ok(Some(MaybeLinear::Linear { start, end }))
        } else { Ok(None) }
    }
}

pub trait CustomOptionalParser: Sized {
    type Out;
    fn require(self) -> ContextResult<Self::Out>;
}

impl<T> CustomOptionalParser for ContextResult<Option<MaybeLinear<T>>> where T: Parseable + Debug + Clone {
    type Out = MaybeLinear<T>;

    fn require(self) -> ContextResult<Self::Out> {
        self?.ok_or(err!("MaybeLinear is required to be Some"))
    }
}

impl Parseable for ShutterEffect {
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

impl Parseable for Speed {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Speed must be a string"))?;
        match s {
            "fast" => Ok(Speed::Fast),
            "slow" => Ok(Speed::Slow),
            "stop" => Ok(Speed::Stop),
            "slow reverse" => Ok(Speed::SlowReverse),
            "fast reverse" => Ok(Speed::FastReverse),
            hertz if hertz.ends_with("Hz") => { let h = s[..hertz.len() - 2].parse::<f32>().map_err(|e| err!(e))?; Ok(Speed::Hertz(h)) }
            bpm if bpm.ends_with("bpm") => {let b = s[..bpm.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(Speed::Bpm(b))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Speed::Percent(Percentage::create(p)))}
            _ => Err(err!("Speed can't be parsed"))
        }
    }
}

impl Parseable for Time {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Time must be a string"))?;
        match s {
            "instant" => Ok(Time::Instant),
            "short" => Ok(Time::Short),
            "long" => Ok(Time::Long),
            millis if millis.ends_with("ms") => {let ms = s[..millis.len() - 2].parse::<f32>().map_err(|e| err!(e))?; Ok(Time::Milliseconds(ms))}
            seconds if seconds.ends_with("s") => { let secs = s[..seconds.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Time::Seconds(secs)) }
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Time::Percent(Percentage::create(p)))}
            _ => Err(err!("Time can't be parsed"))
        }
    }
}

impl Parseable for Brightness {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Brightness must be a string"))?;
        match s {
            "off" => Ok(Brightness::Off),
            "dark" => Ok(Brightness::Dark),
            "bright" => Ok(Brightness::Bright),
            lumen if lumen.ends_with("lm") => {let lm = s[..lumen.len() - 2].parse::<f32>().map_err(|e| err!(e))?; Ok(Brightness::Lumen(lm))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Brightness::Percent(Percentage::create(p)))}
            _ => Err(err!("Brightness can't be parsed"))
        }
    }
}

impl Parseable for ColorTemperature {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("ColorTemperature must be a string"))?;
        match s {
            "warm" => Ok(ColorTemperature::Warm),
            "CTO" => Ok(ColorTemperature::CTO),
            "default" => Ok(ColorTemperature::Default),
            "cold" => Ok(ColorTemperature::Cold),
            "CTB" => Ok(ColorTemperature::CTB),
            kelvin if kelvin.ends_with("K") => {let k = s[..kelvin.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(ColorTemperature::Kelvin(k))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(ColorTemperature::Percent(Percentage::create(p)))}
            _ => Err(err!("ColorTemperature can't be parsed"))
        }
    }
}

impl Parseable for RotationAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("RotationAngle must be a string"))?;
        match s {
            deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(RotationAngle::Degrees(d))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(RotationAngle::Percent(Percentage::create(p)))}
            _ => Err(err!("RotationAngle can't be parsed"))
        }
    }
}

impl Parseable for RotationSpeed {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("RotationSpeed must be a string"))?;
        match s {
            "fast CW" => Ok(RotationSpeed::FastCW),
            "slow CW" => Ok(RotationSpeed::SlowCW),
            "stop" => Ok(RotationSpeed::Stop),
            "slow CCW" => Ok(RotationSpeed::SlowCCW),
            "fast CCW" => Ok(RotationSpeed::FastCCW),
            hertz if hertz.ends_with("Hz") => { let h = s[..hertz.len() - 2].parse::<f32>().map_err(|e| err!(e))?; Ok(RotationSpeed::Hertz(h)) }
            rpm if rpm.ends_with("rpm") => {let r = s[..rpm.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(RotationSpeed::RPM(r))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(RotationSpeed::Percent(Percentage::create(p)))}
            _ => Err(err!("RotationSpeed can't be parsed"))
        }
    }
}

impl Parseable for Color {
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

impl<T> Parseable for Vec<T> where T: Parseable {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let v = value.as_array().ok_or(err!("must be an array"))?;
        v.iter().map(T::parse_from_value).collect::<ContextResult<Vec<T>>>()
    }
}

impl Parseable for DynamicColor {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("DynamicColor must be a string"))?;
        if !s.starts_with("#") ||s.len() != 7 { return Err(err!("DynamicColor is not a hey string"));}
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

impl Parseable for Parameter {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        if value.is_number() {
            Ok(Parameter::Number(value.as_f64().map(|n| n as f32).expect("Tested that it is a number")))
        } else {
            let s = value.as_str().ok_or(err!("Parameter must be a string if it is not a number"))?;
            match s {
                "off" => Ok(Parameter::Off),
                "low" => Ok(Parameter::Low),
                "high" => Ok(Parameter::High),
                "slow" => Ok(Parameter::Slow),
                "fast" => Ok(Parameter::Fast),
                "small" => Ok(Parameter::Small),
                "big" => Ok(Parameter::Big),
                "instant" => Ok(Parameter::Instant),
                "short" => Ok(Parameter::Short),
                "long" => Ok(Parameter::Long),
                percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Parameter::Percentage(Percentage::create(p)))}
                _ => Err(err!("Parameter can't be parsed: '{}'", value))

            }
        }
    }
}

impl Parseable for Percentage {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Percentage must be a string"))?;
        if let Some(s) = s.strip_suffix("%") {
            let p =s.parse::<f32>().map_err(|e| err!(e))?; Ok(Percentage::create(p))
        } else {
            Err(err!("Parameter can't be parsed"))
        }
    }
}


impl Parseable for BeamAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("BeamAngle must be a string"))?;
        match s {
            "closed" => Ok(BeamAngle::Closed),
            "narrow" => Ok(BeamAngle::Narrow),
            "wide" => Ok(BeamAngle::Wide),
            deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(BeamAngle::Degrees(d))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(BeamAngle::Percentage(Percentage::create(p)))}
            _ => Err(err!("BeamAngle can't be parsed"))
        }
    }
}

impl Parseable for HorizontalAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("HorizontalAngle must be a string"))?;
        match s {
            "left" => Ok(HorizontalAngle::Left),
            "right" => Ok(HorizontalAngle::Right),
            "center" => Ok(HorizontalAngle::Center),
            deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(HorizontalAngle::Degrees(d))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(HorizontalAngle::Percentage(Percentage::create(p)))}
            _ => Err(err!("HorizontalAngle can't be parsed"))
        }
    }
}

impl Parseable for VerticalAngle {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("VerticalAngle must be a string"))?;
        match s {
            "top" => Ok(VerticalAngle::Top),
            "bottom" => Ok(VerticalAngle::Bottom),
            "center" => Ok(VerticalAngle::Center),
            deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>().map_err(|e| err!(e))?; Ok(VerticalAngle::Degrees(d))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(VerticalAngle::Percentage(Percentage::create(p)))}
            _ => Err(err!("VerticalAngle can't be parsed"))
        }
    }
}

impl Parseable for Distance {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Distance must be a string"))?;
        match s {
            "near" => Ok(Distance::Near),
            "far" => Ok(Distance::Far),
            meters if meters.ends_with("m") => {let m = s[..meters.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Distance::Meters(m))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Distance::Percentage(Percentage::create(p)))}
            _ => Err(err!("Distance can't be parsed"))
        }
    }
}

impl Parseable for FogKind {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("FogKind must be a string"))?;
        match s {
            "Fog" => Ok(FogKind::Fog),
            "Haze" => Ok(FogKind::Haze),
            _ => Err(err!("FogKind can't be parsed"))
        }
    }
}

impl Parseable for FogOutput {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("FogOutput must be a string"))?;
        match s {
            "off" => Ok(FogOutput::Off),
            "weak" => Ok(FogOutput::Weak),
            "strong" => Ok(FogOutput::Strong),
            vpm if vpm.ends_with("m\\^3/min") => {let v = s[..vpm.len() - 8].parse::<f32>().map_err(|e| err!(e))?; Ok(FogOutput::VolumePerMinute(v))}
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(FogOutput::Percentage(Percentage::create(p)))}
            _ => Err(err!("FogOutput can't be parsed"))
        }
    }
}

impl Parseable for Preset {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Preset must be a string"))?;
        match s {
            "ColorJump" => Ok(Preset::ColorJump),
            "ColorFade" => Ok(Preset::ColorFade),
            _ => Err(err!("Preset can't be parsed"))
        }
    }
}

/// Both types are being parsed th one that succeeds is returned, if both succeed the left one is returned, if both fail an error is returned.
impl<L, R> Parseable for Option<Either<L, R>> where L: Parseable, R: Parseable {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let left: ContextResult<L> = value.parse();
        let right: ContextResult<R> = value.parse();
        decide(left, right)
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        let split = key.split(' ').collect::<Vec<_>>();
        if split.len() != 2 {
            return Err(err!("key for Either must be a whitespace seperated list of two values ('<leftKey> <rightKey>') got: '{}'", key));
        }

        let left: ContextResult<L> = obj.parse(split[0]);
        let right: ContextResult<R> = obj.parse(split[1]);

        decide(left, right)
    }

}

fn decide<L, R>(left: ContextResult<L>, right: ContextResult<R>) -> ContextResult<Option<Either<L, R>>> {
    match (left, right) {
        (Ok(l), _) => Ok(Some(Either::Left(l))),
        (Err(_), Ok(r)) => Ok(Some(Either::Right(r))),
        _ => Ok(None),
    }
}

impl<L, R> CustomOptionalParser for ContextResult<Option<Either<L, R>>> where L: Parseable, R: Parseable {
    type Out = Either<L, R>;

    fn require(self) -> ContextResult<Self::Out> {
        self?.ok_or(err!("Either is required but none of the values could be parsed"))
    }
}