use serde_json::Value;
use mlc_data::{err, ContextResult, misc::ContextError};
use mlc_data::fixture::blueprint::units::{Degree, Hz, Kelvin, Lumen, Meters, MilliSeconds, Percentage, Seconds, VolumePerMin, BPM, RPM};
use crate::convert::parseable::{SimpleParseable};

impl SimpleParseable for Percentage {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("SignedPercentage must be a string"))?;

        match s {
            "off" => Ok(Percentage(0.0)),
            "low" => Ok(Percentage(0.01)),
            "high" => Ok(Percentage(1.0)),
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>().map_err(|e| err!(e))?; Ok(Percentage(p / 100.0))}
            _ => Err(err!("SignedPercentage can't be parsed"))
        }
    }
}

impl SimpleParseable for Hz {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Hz must be a string"))?;

        if let Some(v) = s.strip_suffix("Hz") {
            Ok(Hz(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Hz can't be parsed"))
        }
    }
}

impl SimpleParseable for BPM {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("BPM must be a string"))?;

        if let Some(v) = s.strip_suffix("bpm") {
            Ok(BPM(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("BPM can't be parsed"))
        }
    }
}

impl SimpleParseable for RPM {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("RPM must be a string"))?;

        if let Some(v) = s.strip_suffix("rpm") {
            Ok(RPM(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("RPM can't be parsed"))
        }
    }
}

impl SimpleParseable for Seconds {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Seconds must be a string"))?;

        if let Some(v) = s.strip_suffix("s") {
            Ok(Seconds(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Seconds can't be parsed"))
        }
    }
}

impl SimpleParseable for MilliSeconds {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Milliseconds must be a string"))?;

        if let Some(v) = s.strip_suffix("ms") {
            Ok(MilliSeconds(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Milliseconds can't be parsed"))
        }
    }
}

impl SimpleParseable for Meters {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Meters must be a string"))?;

        if let Some(v) = s.strip_suffix("m") {
            Ok(Meters(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Meters can't be parsed"))
        }
    }
}

impl SimpleParseable for Lumen {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Lumen must be a string"))?;

        if let Some(v) = s.strip_suffix("lm") {
            Ok(Lumen(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Lumen can't be parsed"))
        }
    }
}

impl SimpleParseable for Kelvin {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Kelvin must be a string"))?;

        if let Some(v) = s.strip_suffix("K") {
            Ok(Kelvin(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Kelvin can't be parsed"))
        }
    }
}

impl SimpleParseable for VolumePerMin {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("VolumePerMin must be a string"))?;

        if let Some(v) = s.strip_suffix("m^3/min") {
            Ok(VolumePerMin(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("VolumePerMin can't be parsed"))
        }
    }
}

impl SimpleParseable for Degree {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let s = value.as_str().ok_or(err!("Degree must be a string"))?;

        if let Some(v) = s.strip_suffix("deg") {
            Ok(Degree(v.parse::<f32>().map_err(|e| err!(e))?))
        } else {
            Err(err!("Degree can't be parsed"))
        }
    }
}