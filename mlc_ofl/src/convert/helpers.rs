use std::fmt::Debug;
use serde_json::{Map, Value};
use mlc_data::{DynamicResult, MaybeLinear, Percentage};
use mlc_data::fixture::blueprint::entities::{BeamAngle, Brightness, Color, ColorTemperature, Distance, DynamicColor, FogKind, FogOutput, HorizontalAngle, Parameter, RotationAngle, RotationSpeed, Speed, Time, VerticalAngle};

//TODO: Convert into ParseFromValue and ParseFromObject traits 

pub fn parse_optional_bool(obj: &Map<String, Value>, key: &str) -> DynamicResult<Option<bool>> {
    Ok(obj.get(key).map(|v| v.as_bool().ok_or("must be a bool")).transpose()?)
}

pub fn parse_maybe_linear<T: Debug + Clone, F>(obj: &Map<String, Value>, key: &str, val_parse: F) -> DynamicResult<MaybeLinear<T>> where F: Fn(&Value) -> DynamicResult<T> {
    parse_optional_maybe_linear(obj, key, val_parse)?.ok_or(format!("{} must be present", key).into())
}

pub fn parse_optional_maybe_linear<T: Debug + Clone, F>(obj: &Map<String, Value>, key: &str, val_parse: F) -> DynamicResult<Option<MaybeLinear<T>>> where F: Fn(&Value) -> DynamicResult<T> {
    if let Some (obj) = obj.get(key) {
        Ok(Some(MaybeLinear::Constant(val_parse(obj)?)))
    } else if let Some (s_obj) = obj.get(&format!("{}Start", key)) {
        let start = val_parse(s_obj)?;
        let end = val_parse( obj.get(&format!("{}End", key)).ok_or(format!("if Start is present also End must be there. Key: {key}, Obj: {obj:?}"))?)?;
        Ok(Some(MaybeLinear::Linear { start, end }))
    } else { Ok(None) }
}

pub fn parse_speed(obj: &Value) -> DynamicResult<Speed> {
    let s = obj.as_str().ok_or("Speed must be a string")?;
    match s {
        "fast" => Ok(Speed::Fast),
        "slow" => Ok(Speed::Slow),
        "stop" => Ok(Speed::Stop),
        "slow reverse" => Ok(Speed::SlowReverse),
        "fast reverse" => Ok(Speed::FastReverse),
        hertz if hertz.ends_with("Hz") => { let h = s[..hertz.len() - 2].parse::<f32>()?; Ok(Speed::Hertz(h)) }
        bpm if bpm.ends_with("bpm") => {let b = s[..bpm.len() - 3].parse::<f32>()?; Ok(Speed::Bpm(b))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(Speed::Percent(Percentage::create(p)))}
        _ => Err("Speed can't be parsed".into())
    }
}

pub fn parse_time(obj: &Value) -> DynamicResult<Time> {
    let s = obj.as_str().ok_or("Time must be a string")?;
    match s {
        "instant" => Ok(Time::Instant),
        "short" => Ok(Time::Short),
        "long" => Ok(Time::Long),
        millis if millis.ends_with("ms") => {let ms = s[..millis.len() - 2].parse::<f32>()?; Ok(Time::Milliseconds(ms))}
        seconds if seconds.ends_with("s") => { let secs = s[..seconds.len() - 1].parse::<f32>()?; Ok(Time::Seconds(secs)) }
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(Time::Percent(Percentage::create(p)))}
        _ => Err("Time can't be parsed".into())
    }
}

pub fn parse_brightness(obj: &Value) -> DynamicResult<Brightness> {
    let s = obj.as_str().ok_or("Brightness must be a string")?;
    match s {
        "off" => Ok(Brightness::Off),
        "dark" => Ok(Brightness::Dark),
        "bright" => Ok(Brightness::Bright),
        lumen if lumen.ends_with("lm") => {let lm = s[..lumen.len() - 2].parse::<f32>()?; Ok(Brightness::Lumen(lm))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(Brightness::Percent(Percentage::create(p)))}
        _ => Err("Brightness can't be parsed".into())
    }
}

pub fn parse_color_temperature(obj: &Value) -> DynamicResult<ColorTemperature> {
    let s = obj.as_str().ok_or("ColorTemperature must be a string")?;
    match s {
        "warm" => Ok(ColorTemperature::Warm),
        "CTO" => Ok(ColorTemperature::CTO),
        "default" => Ok(ColorTemperature::Default),
        "cold" => Ok(ColorTemperature::Cold),
        "CTB" => Ok(ColorTemperature::CTB),
        kelvin if kelvin.ends_with("K") => {let k = s[..kelvin.len() - 1].parse::<f32>()?; Ok(ColorTemperature::Kelvin(k))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(ColorTemperature::Percent(Percentage::create(p)))}
        _ => Err("ColorTemperature can't be parsed".into())
    }
}

pub fn parse_rotation_angle(obj: &Value) -> DynamicResult<RotationAngle> {
    let s = obj.as_str().ok_or("RotationAngle must be a string")?;
    match s {
        deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>()?; Ok(RotationAngle::Degrees(d))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(RotationAngle::Percent(Percentage::create(p)))}
        _ => Err("RotationAngle can't be parsed".into())
    }
}

pub fn parse_rotation_speed(obj: &Value) -> DynamicResult<RotationSpeed> {
    let s = obj.as_str().ok_or("RotationSpeed must be a string")?;
    match s {
        "fast CW" => Ok(RotationSpeed::FastCW),
        "slow CW" => Ok(RotationSpeed::SlowCW),
        "stop" => Ok(RotationSpeed::Stop),
        "slow CCW" => Ok(RotationSpeed::SlowCCW),
        "fast CCW" => Ok(RotationSpeed::FastCCW),
        hertz if hertz.ends_with("Hz") => { let h = s[..hertz.len() - 2].parse::<f32>()?; Ok(RotationSpeed::Hertz(h)) }
        rpm if rpm.ends_with("rpm") => {let r = s[..rpm.len() - 3].parse::<f32>()?; Ok(RotationSpeed::RPM(r))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(RotationSpeed::Percent(Percentage::create(p)))}
        _ => Err("RotationSpeed can't be parsed".into())
    }
}

pub fn parse_color(obj: &Value) -> DynamicResult<Color> {
    let s = obj.as_str().ok_or("Color must be a string")?;

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
        _ => Err("Color can't be parsed".into())
    }
}

pub fn parse_vec<T, F>(obj: &Value, val_fun: F) -> DynamicResult<Vec<T>> where F: Fn(&Value) -> DynamicResult<T> {
    let v = obj.as_array().ok_or("must be an array")?;
    v.iter().map(|v| val_fun(v)).collect::<DynamicResult<Vec<T>>>()
}

pub fn parse_dynamic_color(obj: &Value) -> DynamicResult<DynamicColor> {
    let s = obj.as_str().ok_or("DynamicColor must be a string")?;
    if !s.starts_with("#") ||s.len() != 7 { return Err("DynamicColor is not a hey string".into());}
    let r = u8::from_str_radix(&s[1..=2], 16)?;
    let g = u8::from_str_radix(&s[3..=4], 16)?;
    let b = u8::from_str_radix(&s[5..=6], 16)?;
    Ok(DynamicColor{
        r,
        g,
        b
    })
}

pub fn parse_parameter(obj: &Value) -> DynamicResult<Parameter> {
    if obj.is_number() {
        Ok(Parameter::Number(obj.as_f64().map(|n| n as f32).expect("Tested that it is a number")))
    } else {
        let s = obj.as_str().ok_or("Parameter must be a string if it is not a number")?;
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
            percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(Parameter::Percentage(Percentage::create(p)))}
            _ => Err("Parameter can't be parsed".into())

        }
    }
}

pub fn parse_percentage(obj: &Value) -> DynamicResult<Percentage> {
    let s = obj.as_str().ok_or("Percentage must be a string")?;
    if s.ends_with("%") {
        let p =s[..s.len() - 1].parse::<f32>()?; Ok(Percentage::create(p))
    } else {
        Err("Parameter can't be parsed".into())
    }
}

pub fn parse_beam_angle(obj: &Value) -> DynamicResult<BeamAngle> {
    let s = obj.as_str().ok_or("BeamAngle must be a string")?;
    match s {
        "closed" => Ok(BeamAngle::Closed),
        "narrow" => Ok(BeamAngle::Narrow),
        "wide" => Ok(BeamAngle::Wide),
        deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>()?; Ok(BeamAngle::Degrees(d))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(BeamAngle::Percentage(Percentage::create(p)))}
        _ => Err("BeamAngle can't be parsed".into())
    }
}

pub fn parse_horizontal_angle(obj: &Value) -> DynamicResult<HorizontalAngle> {
    let s = obj.as_str().ok_or("HorizontalAngle must be a string")?;
    match s {
        "left" => Ok(HorizontalAngle::Left),
        "right" => Ok(HorizontalAngle::Right),
        "center" => Ok(HorizontalAngle::Center),
        deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>()?; Ok(HorizontalAngle::Degrees(d))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(HorizontalAngle::Percentage(Percentage::create(p)))}
        _ => Err("HorizontalAngle can't be parsed".into())
    }
}

pub fn parse_vertical_angle(obj: &Value) -> DynamicResult<VerticalAngle> {
    let s = obj.as_str().ok_or("VerticalAngle must be a string")?;
    match s {
        "top" => Ok(VerticalAngle::Top),
        "bottom" => Ok(VerticalAngle::Bottom),
        "center" => Ok(VerticalAngle::Center),
        deg if deg.ends_with("deg") => {let d = s[..deg.len() - 3].parse::<f32>()?; Ok(VerticalAngle::Degrees(d))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(VerticalAngle::Percentage(Percentage::create(p)))}
        _ => Err("VerticalAngle can't be parsed".into())
    }
}

pub fn parse_distance(obj: &Value) -> DynamicResult<Distance> {
    let s = obj.as_str().ok_or("Distance must be a string")?;
    match s {
        "near" => Ok(Distance::Near),
        "far" => Ok(Distance::Far),
        meters if meters.ends_with("m") => {let m = s[..meters.len() - 1].parse::<f32>()?; Ok(Distance::Meters(m))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(Distance::Percentage(Percentage::create(p)))}
        _ => Err("Distance can't be parsed".into())
    }
}

pub fn parse_fog_kind(obj: &Value) -> DynamicResult<FogKind> {
    let s = obj.as_str().ok_or("FogKind must be a string")?;
    match s {
        "Fog" => Ok(FogKind::Fog),
        "Haze" => Ok(FogKind::Haze),
        _ => Err("FogKind can't be parsed".into())
    }
}

pub fn parse_fog_output(obj: &Value) -> DynamicResult<FogOutput> {
    let s = obj.as_str().ok_or("FogOutput must be a string")?;
    match s {
        "off" => Ok(FogOutput::Off),
        "weak" => Ok(FogOutput::Weak),
        "strong" => Ok(FogOutput::Strong),
        vpm if vpm.ends_with("m\\^3/min") => {let v = s[..vpm.len() - 8].parse::<f32>()?; Ok(FogOutput::VolumePerMinute(v))}
        percentage if percentage.ends_with("%") => {let p =s[..percentage.len() - 1].parse::<f32>()?; Ok(FogOutput::Percentage(Percentage::create(p)))}
        _ => Err("FogOutput can't be parsed".into())
    }
}
