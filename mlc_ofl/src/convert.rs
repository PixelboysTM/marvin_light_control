use std::collections::HashMap;
use serde_json::{Map, Value};
use mlc_data::{DmxGranularity, DynamicError, DynamicResult, GenericDMXValue, MaybeLinear, Percentage, PercentageDmxExt};
use mlc_data::fixture::blueprint::{Capability, CapabilityKind, Channel, ChannelIdentifier, CommonChannel, FixtureBlueprint, Metadata, Mode, Physical, Pixel, PixelIdentifier, PixelMatrix};
use mlc_data::fixture::blueprint::entities::{Brightness, ShutterEffect};
use mlc_data::project::ToFileName;
use crate::convert::helpers::{parse_beam_angle, parse_brightness, parse_color, parse_color_temperature, parse_distance, parse_dynamic_color, parse_fog_kind, parse_fog_output, parse_horizontal_angle, parse_maybe_linear, parse_optional_bool, parse_optional_maybe_linear, parse_parameter, parse_percentage, parse_rotation_angle, parse_rotation_speed, parse_speed, parse_time, parse_vec, parse_vertical_angle};

mod helpers;

pub fn convert(ofl_source: &Value, manufacturer: String) -> Result<FixtureBlueprint, Box<dyn std::error::Error>> {
    let meta = parse_metadata(ofl_source, manufacturer)?;

    let matrix = parse_matrix(&ofl_source["matrix"])?;


    let modes = parse_modes(matrix.as_ref(), &ofl_source["modes"])?;


    let wheels = parse_wheels(&ofl_source["wheels"])?;


    //TODO: parse templateChannels
    let channels = parse_channels(&ofl_source["availableChannels"])?;

    Ok(FixtureBlueprint {
        meta,
        matrix,
        modes,
        wheels,
        channels
    })
}

fn parse_channels(src: &Value) -> Result<HashMap<ChannelIdentifier, Channel>, Box<dyn std::error::Error>> {
    if src.is_null() { return Ok(HashMap::new()) }

    let obj = src.as_object().ok_or("'availableChannels' if present must be an object")?;

    let mut channels = HashMap::new();
    for (k,v) in obj {
        let channel = parse_channel(v)?;
        channels.insert(k.clone(), channel);
    }

    Ok(channels)
}

fn parse_channel(src: &Value) -> DynamicResult<Channel> {
    let obj = src.as_object().ok_or("channel must be an object")?;

    let granularity = obj.get("fineChannelAliases").and_then(|v| v.as_array()).map(|v| v.iter().map(|a| a.as_str().ok_or("fineChannelAlias must be an string")).collect::<Result<Vec<_>, _>>()).transpose()?.unwrap_or(vec![]);

    let common = parse_common_channel(obj, match granularity.len() { 0 => DmxGranularity::Single, 1 => DmxGranularity::Double, 2 => DmxGranularity::Tripple, _ => DmxGranularity::Single })?;

    match granularity.as_slice() {
        [] => Ok(Channel::Single {
            channel: common
        }),
        [fine] => Ok(Channel::Double {
            channel: common,
            second_channel_name: fine.to_string(),
        }),
        [fine, grain] => Ok(Channel::Tripple {
            channel: common,
            second_channel_name: fine.to_string(),
            third_channel_name: grain.to_string(),
        }),
        _ => Err(format!("Unsupported channel granularity {}", granularity.len()).into())
    }
}

fn parse_common_channel(obj: &Map<String, Value>, granularity: DmxGranularity) -> DynamicResult<CommonChannel> {
    let value_resolution = obj.get("dmxValueResolution").map(|r| r.as_str().ok_or("valueResolution must be a string".to_string()).and_then(|s| match s {
        "8bit" => Ok(DmxGranularity::Single),
        "16bit" => Ok(DmxGranularity::Double),
        "24bit" => Ok(DmxGranularity::Tripple),
        _=> Err(format!("Unsupported dmxValueResolution {}", s))
    })).unwrap_or(Ok(granularity))?;

    let default_value = obj.get("default_value").map(|v| match v {
        Value::Number(number) => Ok(number.as_u64().ok_or("default_value must be unsigned int".to_string()).map(|v| Percentage::from_gen_dmx(GenericDMXValue::create(v as u32), value_resolution))?),
        Value::String(s) if s.ends_with("%") => Ok(s[..s.len() - 1].parse::<f32>().map(Percentage::create)?),
        _ => Err(<&str as Into<DynamicError>>::into("invalid defaultValue")),
    }).unwrap_or(Ok(Percentage::create(0.0)))?;

    let caps_decide = (obj.contains_key("capability"), obj.contains_key("capabilities"));

    let caps = match caps_decide {
        (true, false) => vec![parse_capability(&obj["capability"], true, value_resolution)?],
            (false, true) => {
            let caps = obj["capabilities"].as_array().ok_or("capabilities must be an array")?;
            caps.iter().map(|c| parse_capability(c, false, value_resolution)).collect::<Result<Vec<_>, _>>()?
        },
        _ => {
            log::debug!("No capabilities specified");
            vec![]
        }
     };

    Ok(CommonChannel {
        default_value,
        capabilities: caps
    })
}

fn parse_capability(src: &Value, is_single: bool, granularity: DmxGranularity) -> Result<Capability, Box<dyn std::error::Error>> {
    let obj = src.as_object().ok_or("capability must be an object")?;

    let range = if is_single {
        Percentage::create(0.0)..=Percentage::create(1.0)
    } else {
        let range = src.get("dmxRange").and_then(|v| v.as_array().map(|v| v.iter().map(|d| d.as_u64().ok_or("dmxValue in range must be an unsigned int").map(|d| Percentage::from_gen_dmx(GenericDMXValue::create(d as u32), granularity))).collect::<Result<Vec<_>, _>>())).ok_or("dmxRange must be present")??;
        if range.len() != 2 {
            return Err("dmxRange must contain exactly two values".into());
        }
        range[0]..=range[1]
    };

    let comment = obj.get("comment").map(|c| c.as_str().map(|s| s.to_string()).ok_or("comment must be a string")).transpose()?;

    let kind = parse_capability_kind(obj)?;

    Ok(Capability {
        range,
        comment,
        kind,
        pixel: PixelIdentifier::Master //TODO: Change later for template channels
    })
}

fn parse_capability_kind(obj: &Map<String, Value>) -> DynamicResult<CapabilityKind> {
    let kind = obj.get("type").and_then(|v| v.as_str()).ok_or("capability type must be a string")?;

    let cap = match kind {
        "NoFunction" => CapabilityKind::NoFunction,
        "ShutterStrobe" => CapabilityKind::ShutterStrobe {
            sound_controlled: parse_optional_bool(obj, "soundControlled")?.unwrap_or(false),
            random_timing: parse_optional_bool(obj, "randomTiming")?.unwrap_or(false),
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?,
            duration: parse_optional_maybe_linear(obj, "duration", parse_time)?,
            effect: match obj.get("shutterEffect").and_then(|v| v.as_str()).ok_or("shutterEffect must be a string and present")? {
                "Open" => ShutterEffect::Open,
                "Closed" => ShutterEffect::Closed,
                "Strobe" => ShutterEffect::Strobe,
                "Pulse" => ShutterEffect::Pulse,
                "RampUp" => ShutterEffect::RampUp,
                "RampDown" => ShutterEffect::RampDown,
                "RampUpDown" => ShutterEffect::RampUpDown,
                "Lightning" => ShutterEffect::Lightning,
                "Spikes" => ShutterEffect::Spikes,
                "Burst" => ShutterEffect::Burst,
                _=> return Err(format!("Unsupported shutterEffect {}", kind).into())
            },
        },
        "StrobeSpeed" => CapabilityKind::StrobeSpeed {
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?.ok_or("Speed must be present")?,
        },
        "StrobeDuration" => CapabilityKind::StrobeDuration {
            duration: parse_optional_maybe_linear(obj, "duration", parse_time)?.ok_or("Speed must be present")?,
        },
        "Intensity" => CapabilityKind::Intensity {
          brightness: parse_optional_maybe_linear(obj, "brightness", parse_brightness)?.unwrap_or(MaybeLinear::Linear {
              start: Brightness::Off,
              end: Brightness::Bright,
          }),
        },
        "ColorIntensity" => CapabilityKind::ColorIntensity {
            brightness: parse_optional_maybe_linear(obj, "brightness", parse_brightness)?.unwrap_or(MaybeLinear::Linear {
                start: Brightness::Off,
                end: Brightness::Bright,
            }),
            color: parse_color(obj.get("color").ok_or("color must be present")?)?,
        },
        "ColorPreset" => CapabilityKind::ColorPreset {
            colors: parse_optional_maybe_linear(obj, "colors", |v| parse_vec(v, parse_dynamic_color))?.ok_or("colorPreset must have colors")?,
            color_temperature: parse_optional_maybe_linear(obj, "colorTemperature", parse_color_temperature)?,
        },
        "Pan" => CapabilityKind::Pan {
            angle: parse_optional_maybe_linear(obj, "angle", parse_rotation_angle)?.ok_or("Rotation angle must be present")?,
        },
        "PanContinuous" => CapabilityKind::PanContinuous {
            speed: parse_optional_maybe_linear(obj, "speed", parse_rotation_speed)?.ok_or("Rotation speed must be present")?,
        },
        "Tilt" => CapabilityKind::Tilt {
            angle: parse_optional_maybe_linear(obj, "angle", parse_rotation_angle)?.ok_or("Rotation angle must be present")?,
        },
        "TiltContinuous" => CapabilityKind::TiltContinuous {
            speed: parse_optional_maybe_linear(obj, "speed", parse_rotation_speed)?.ok_or("Rotation speed must be present")?,
        },
        "PanTiltSpeed" => CapabilityKind::PanTiltSpeed {
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?.ok_or("speed must be present")?,
            duration: parse_optional_maybe_linear(obj, "duration", parse_time)?,
        },
        "WheelSlot" => CapabilityKind::WheelSlot {
            wheel: obj.get("wheel").map(|v| v.as_str().map(|s| s.to_string()).ok_or("wheel must be a string")).transpose()?,
            slot_number: parse_optional_maybe_linear(obj, "slotNumber", |v| v.as_f64().map(|u| u as f32).ok_or("slotNumber must be a unsigned number".into()))?.ok_or("slotNumber must be present")?,
        },
        "WheelShake" => CapabilityKind::WheelShake,
        "WheelSlotRotation" => CapabilityKind::WheelSlotRotation,
        "WheelRotation" => CapabilityKind::WheelRotation,
        "Effect" => CapabilityKind::Effect {
            name: obj.get("effectName").and_then(|v| v.as_str().map(|s| s.to_string())).ok_or("Effect must have name")?,
            preset: obj.get("effectPreset").and_then(|v| v.as_str().map(|s| s.to_string())).ok_or("Effect must have preset")?,
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?,
            duration: parse_optional_maybe_linear(obj, "duration", parse_time)?,
            parameter: parse_optional_maybe_linear(obj, "parameter", parse_parameter)?,
            sound_controlled: parse_optional_bool(obj, "soundControlled")?.unwrap_or(false),
            sound_sensitivity: parse_optional_maybe_linear(obj, "soundSensitivity", parse_percentage)?,
        },
        "EffectSpeed" => CapabilityKind::EffectSpeed {
            speed: parse_maybe_linear(obj, "speed", parse_speed)?,
        },
        "EffectDuration" => CapabilityKind::EffectDuration {
            duration: parse_maybe_linear(obj, "duration", parse_time)?,
        },
        "EffectParameter" => CapabilityKind::EffectParameter {
            parameter: parse_maybe_linear(obj, "effectParameter", parse_parameter)?,
        },
        "SoundSensitivity" => CapabilityKind::SoundSensitivity {
            sensitivity: parse_maybe_linear(obj, "soundSensitivity", parse_percentage)?,
        },
        "BeamAngle" => CapabilityKind::BeamAngle {
            angle: parse_maybe_linear(obj, "angle", parse_beam_angle)?,
        },
        "BeamPosition" => CapabilityKind::BeamPosition {
            horizontal_angle: parse_optional_maybe_linear(obj, "horizontalAngle", parse_horizontal_angle)?,
            vertical_angle: parse_optional_maybe_linear(obj, "verticalAngle", parse_vertical_angle)?,
        },
        "Focus" => CapabilityKind::Focus {
            distance: parse_maybe_linear(obj, "distance", parse_distance)?,
        },
        "Zoom" => CapabilityKind::Zoom {
            angle: parse_maybe_linear(obj, "angle", parse_beam_angle)?,
        },
        "Iris" => CapabilityKind::Iris {
            open_percent: parse_maybe_linear(obj, "openPercent", parse_percentage)?,
        },
        "IrisEffect" => CapabilityKind::IrisEffect {
            name: obj.get("effectName").and_then(|v| v.as_str().map(|s| s.to_string())).ok_or("IrisEffect must have name")?,
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?,
        },
        "Frost" => CapabilityKind::Frost {
            intensity: parse_maybe_linear(obj, "frostIntensity", parse_percentage)?,
        },
        "FrostEffect" => CapabilityKind::FrostEffect {
            name: obj.get("effectName").and_then(|v| v.as_str().map(|s| s.to_string())).ok_or("FrostEffect must have name")?,
            speed: parse_optional_maybe_linear(obj, "speed", parse_speed)?,
        },
        "Prism" => CapabilityKind::Prism {
            speed: parse_optional_maybe_linear(obj, "speed", parse_rotation_speed)?,
            angle: parse_optional_maybe_linear(obj, "angle", parse_rotation_angle)?,
        },
        "PrismRotation" => CapabilityKind::PrismRotation {
            speed: parse_optional_maybe_linear(obj, "speed", parse_rotation_speed)?,
            angle: parse_optional_maybe_linear(obj, "angle", parse_rotation_angle)?,
        },
        "BladeInsertion" => CapabilityKind::BladeInsertion,
        "BladeRotation" => CapabilityKind::BladeRotation,
        "BladeSystemRotation" => CapabilityKind::BladeSystemRotation,
        "Fog" => CapabilityKind::Fog {
            kind: parse_fog_kind(obj.get("fogType").unwrap_or(&Value::String("Fog".to_string())))?,
            output: parse_optional_maybe_linear(obj, "fogOutput", parse_fog_output)?,
        },
        "FogOutput" => CapabilityKind::FogOutput {
            output: parse_maybe_linear(obj, "fogOutput", parse_fog_output)?,
        },
        "FogType" => CapabilityKind::FogType {
            kind: parse_fog_kind(obj.get("fogType").ok_or("fogType must be present")?)?,
        },
        "Rotation" => CapabilityKind::Rotation {
            speed: parse_optional_maybe_linear(obj, "speed", parse_rotation_speed)?,
            angle: parse_optional_maybe_linear(obj, "angle", parse_rotation_angle)?,
        },
        "Speed" => CapabilityKind::Speed {
            speed: parse_maybe_linear(obj, "speed", parse_speed)?,
        },
        "Time" => CapabilityKind::Time {
            time: parse_maybe_linear(obj, "time", parse_time)?,
        },
        "Maintenance" => CapabilityKind::Maintenance {
            parameter: parse_optional_maybe_linear(obj, "parameter", parse_parameter)?,
            hold: if let Some(v) = obj.get("hold") { Some(parse_time(v)?) } else { None },
        },
        "Generic" => CapabilityKind::Generic,
        _ => return Err("Unknown capability type".into()),
    };
    Ok(cap)
}

fn parse_wheels(src: &Value) -> Result<Option<Vec<()>>, Box<dyn std::error::Error>> {
    if src.is_null() { return Ok(None); }

    log::warn!("Wheel parsing not yet implemented");
    Ok(Some(vec![]))

}

fn parse_modes(m: Option<&PixelMatrix>,src: &Value) -> Result<Vec<Mode>, Box<dyn std::error::Error>> {
    if src.is_null() { return Ok(Vec::new()); }

    let modes = src.as_array().ok_or("modes is not an array")?;

    modes.iter().map(|v| parse_mode(m, v) ).collect::<Result<Vec<_>, _>>()
}

fn gen_each_pixel(f: usize, s: usize, t: usize) -> Vec<String> {
    let mut v = Vec::new();
    for fi in 1..=f {
        for si in 1..=s {
            for ti in 1..=t {
                v.push(format!("({}, {}, {})", ti, si, fi));
            }
        }
    }
    v
}



fn parse_mode(m: Option<&PixelMatrix>,src: &Value) -> Result<Mode, Box<dyn std::error::Error>> {
    let obj = src.as_object().ok_or("mode is not an object")?;

    fn parse_channel(m: Option<&PixelMatrix>,src: &Value) -> Result<Vec<Option<ChannelIdentifier>>, Box<dyn std::error::Error>> {
        if src.is_null() { return Ok(vec![None]) }
        if let Some(str) = src.as_str() {return Ok(vec![Some(str.to_string())]) }

        let obj = src.as_object().ok_or("mode must be one of null, string or object")?;
        obj.get("insert").and_then(|s| s.as_str()).and_then(|s| if s == "matrixChannels" {Some(())} else {None}).ok_or("object mode must contain 'insert: matrixChannels'")?;

        let m = m.ok_or("template channels require a matrix")?;

        enum ChannelOrder {
            PerPixel,
            PerChannel
        }

        let channel_order = obj.get("channelOrder").and_then(|s| s.as_str()).and_then(|s| match s {
            "perPixel" => Some(ChannelOrder::PerPixel),
            "perChannel" => Some(ChannelOrder::PerChannel),
            _ => None
        }).ok_or("channelOrder must be one of perPixel, perChannel")?;

        let template_channels = obj.get("templateChannels").and_then(|s| s.as_array()).ok_or("templateChannels must be array")?.iter().map(|v| {
            match v {
                Value::Null => Ok(None),
                Value::String(str) => Ok(Some(str.clone())),
                _ =>  Err("templateChannels must be string")
            }
        }).collect::<Result<Vec<_>, _>>()?;

        enum RepeatFor {
            Keys(Vec<String>),
            EachPixelABC,
            EachPixelXYZ,
        	EachPixelXZY,
        	EachPixelYXZ,
        	EachPixelYZX,
        	EachPixelZXY,
        	EachPixelZYX,
            EachPixelGroup,
        }

        let repeat_for = obj.get("repeatFor").and_then(|v| {
            match v {
                Value::String(s) if s == "eachPixelABC" => Some(RepeatFor::EachPixelABC),
                Value::String(s) if s == "eachPixelXYZ" => Some(RepeatFor::EachPixelXYZ),
                Value::String(s) if s == "eachPixelXZY" => Some(RepeatFor::EachPixelXZY),
                Value::String(s) if s == "eachPixelYXZ" => Some(RepeatFor::EachPixelYXZ),
                Value::String(s) if s == "eachPixelYZX" => Some(RepeatFor::EachPixelYZX),
                Value::String(s) if s == "eachPixelZXY" => Some(RepeatFor::EachPixelZXY),
                Value::String(s) if s == "eachPixelZYX" => Some(RepeatFor::EachPixelZYX),
                Value::String(s) if s == "eachPixelGroup" => Some(RepeatFor::EachPixelGroup),
                Value::Array(arr ) => arr.iter().map(|v| v.as_str().map(|s| s.to_string())).collect::<Option<Vec<_>>>().map(RepeatFor::Keys),
                _ => None
            }
        }).ok_or("repeatFor is invalid")?;



        let dims = m.dimensions();
        let pixel_keys = match repeat_for {
            RepeatFor::EachPixelABC => {
                let mut ps = m.pixels.iter().flatten().flatten().flatten().map(|p| p.key.clone()).collect::<Vec<_>>();
                ps.sort();
                ps
            }
            RepeatFor::EachPixelGroup => m.groups.clone(),
            RepeatFor::EachPixelXYZ => gen_each_pixel(dims[0], dims[1], dims[2]),
            RepeatFor::EachPixelXZY => gen_each_pixel(dims[0], dims[2], dims[1]),
            RepeatFor::EachPixelYXZ => gen_each_pixel(dims[1], dims[0], dims[2]),
            RepeatFor::EachPixelYZX => gen_each_pixel(dims[1], dims[2], dims[0]),
            RepeatFor::EachPixelZXY => gen_each_pixel(dims[2], dims[0], dims[1]),
            RepeatFor::EachPixelZYX => gen_each_pixel(dims[2], dims[1], dims[0]),
            RepeatFor::Keys(keys) => keys
        };

        let channels = match channel_order {
            ChannelOrder::PerPixel => {
                pixel_keys.iter().flat_map(|pixel| {
                    template_channels.iter().map(|template| {
                        template.as_ref().map(|t| t.replace("$pixelKey", pixel))
                    })
                }).collect::<Vec<_>>()
            }
            ChannelOrder::PerChannel => {
                template_channels.iter().flat_map(|template| {
                    pixel_keys.iter().map(move |pixel| {
                        template.as_ref().map(|t| t.replace("$pixelKey", pixel))
                    })
                }).collect::<Vec<_>>()
            }
        };

        Ok(channels)
    }

    let name =  obj.get("name").ok_or("mode needs a name")?.as_str().ok_or("mode name is not a string")?.to_string();
    let channels = obj.get("channels").ok_or("mode needs a channels")?.as_array().ok_or("mode channels is not an array")?.iter().map(|c| parse_channel(m, c)).collect::<Result<Vec<_>, _>>()?.into_iter()
        .flatten().collect::<Vec<_>>();



    Ok(Mode {
        channels,
        name
    })
}

fn parse_metadata(src: &Value, manufacturer: String) -> Result<Metadata, Box<dyn std::error::Error>> {
    let name = src["name"].as_str().ok_or("Fixture contains no name")?.to_string();
    let identifier = make_identifier(&name, &manufacturer);
    let physical = parse_physical(&src["physical"])?;
    Ok(Metadata {
        manufacturer,
        name,
        physical,
        identifier
    })
}

fn make_identifier(name: &str, manufacturer: &str) -> String {
    format!("{}:{}", manufacturer.to_project_file_name(), name.to_project_file_name())
}

fn parse_physical(src: &Value) -> Result<Physical, Box<dyn std::error::Error>> {
    if matches!(src, Value::Null) {
        return Ok(Physical {
            dimensions: None,
            bulb: String::new(),
            weight: 0.0,
            power_consumption: 0.0,
            dmx_connector: String::new(),
            lens: String::new(),
            power_connectors: String::new(),
        });
    }

    let dimension_vec = src["dimensions"].as_array().map(|v| v.iter().map(|val| val.as_f64().ok_or("Dimension was not a number").map(|u| u as f32 )).collect::<Result<Vec<_>, _>>()).transpose()?;
    let dimensions = if let Some (ds) = dimension_vec {
          if ds.len() == 3 {
              Some([ds[0], ds[1], ds[2]])
          } else { Err("Physical Dimensions were not 3D")?; None }
    } else {
        None
    };

    Ok(Physical {
        dimensions,
        weight: src["weight"].as_f64().map(|f| f as f32).unwrap_or(0.0),
        lens: src["lens"].as_str().unwrap_or("").to_string(),
        bulb: src["bulb"].as_str().unwrap_or("").to_string(),
        power_consumption: src["power"].as_f64().map(|f| f as f32).unwrap_or(0.0),
        dmx_connector: src["DMXconnector"].as_str().unwrap_or("").to_string(),
        power_connectors: src["powerConnectors"].as_str().unwrap_or("").to_string(),
    })
}

fn parse_matrix(src: &Value) -> Result<Option<PixelMatrix>, Box<dyn std::error::Error>> {
    if matches!(src, Value::Null) {
        return Ok(None);
    }

    fn parse_pixel_count(src: &Value) -> Result<PixelMatrix, Box<dyn std::error::Error>> {
        let array = src.as_array().ok_or("Matrix pixelCount not an array")?.iter().map(|v| v.as_u64().ok_or("Matrix pixelCount entry not an integer").map(|u| u as usize)).collect::<Result<Vec<_>, _>>()?;
        if array.len() != 3 {
            Err("Matrix pixelCount were not 3")?;
        }

        let mut z = Vec::with_capacity(array[2]);
        z.resize(array[2],Some(Pixel{ key: String::new(), groups: vec![]}));

        let mut y = Vec::with_capacity(array[1]);
        y.resize(array[1], z);

        let mut x = Vec::with_capacity(array[0]);
        x.resize(array[0], y);
        
        let mut i = 1;
        
        for xs in &mut x {
            for ys in xs {
                for zs in ys.iter_mut().flatten() {
                        zs.key = i.to_string();
                        i += 1;
                }
            }
        }

        Ok(PixelMatrix {
            pixels: x,
            groups: vec![]
        })
    }

    fn parse_pixel_keys(src: &Value) -> Result<PixelMatrix, Box<dyn std::error::Error>> {
        let m = src.as_array().ok_or("Matrix pixelKeys not an array")?.iter().map(|v|
            v.as_array().ok_or("Matrix pixelKeys not an 3d array").map(|a| a.iter().map(|v|
                v.as_array().ok_or("Matrix pixelKeys not an 3d array").map(|a| a.iter().map(|v| v.as_str().map(|s|Pixel { key: s.to_string(), groups: vec![]})).collect::<Vec<_>>())).collect::<Result<Vec<_>, _>>())).collect::<Result<Vec<_>, _>>()?.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(PixelMatrix{
            pixels: m,
            groups: vec![]
        })
    }

    fn add_pixel_groups(mut m: PixelMatrix, src: &Value) -> Result<PixelMatrix, Box<dyn std::error::Error>> {
        if src.is_null() {
            return Ok(m);
        }


        fn make_axis_constrained(src: &Value) -> Box<dyn Fn(usize) -> bool> {
            let s = src.as_str().unwrap_or("");

            if s.is_empty() { Box::new(|_| true) }
            else if let Some(st) = s.strip_prefix("<=") {let n = st.parse::<usize>().unwrap_or(0); Box::new(move |x| x <= n)}
            else if let Some(st) = s.strip_prefix(">=") {let n = st.parse::<usize>().unwrap_or(0); Box::new(move |x| x >= n)}
            else if let Some(st) = s.strip_prefix("<") {let n = st.parse::<usize>().unwrap_or(0); Box::new(move |x| x < n)}
            else if let Some(st) = s.strip_prefix(">") {let n = st.parse::<usize>().unwrap_or(0); Box::new(move |x| x > n)}
            else if let Some(st) = s.strip_prefix("=") {let n = st.parse::<usize>().unwrap_or(0); Box::new(move |x| x == n)}
            else if s == "even" {Box::new(|x| x % 2 == 0)}
            else if s == "odd" {Box::new(|x| x % 2 == 1)}
            else {
                let ss = s.split('n').collect::<Vec<&str>>();
                if ss.len() == 2 && ss[1].starts_with("+") {
                    let x1 = ss[1].parse::<usize>().unwrap_or(0);
                    let x2 = ss[1].parse::<usize>().unwrap_or(0);
                    Box::new(move |x: usize| x % x1 == x2)
                } else if ss.len() == 2 && ss[1].is_empty() {
                    let n = ss[0].parse::<usize>().unwrap_or(0);
                    Box::new(move |x: usize| x % n == 0)
                } else {
                    log::error!("Invalid matrix pixelGroup axis constrained");
                    Box::new(|_| false)
                }
            }
        }

        type ConsFunc = Box<dyn Fn(Option<&Pixel>) -> bool>;
        fn make_name_constrained(src: &Value) -> ConsFunc {

            if src.is_null() {
                return Box::new(|_| true);
            }

            let empty = vec![];
            let ss = src.as_array().unwrap_or_else(|| {
                log::error!("Matrix name constrained not an array");
                &empty
            });
            let mut funcs: Vec<ConsFunc> = vec![];

            for s in ss {
                let s = s.as_str().unwrap_or("");
            let fun = if s.is_empty() { Box::new(move |_: Option<&Pixel>| true) as ConsFunc  }
            else {
                let regex = regex::Regex::new(s);
                let b = match regex {
                    Ok(regex) => {
                        Box::new(move |idents: Option<&Pixel>| {
                            if let Some(Pixel { key: pixel_key, ..}) = idents { regex.is_match(pixel_key) } else {
                                false
                            }
                        })
                    }
                    Err(e) => {
                        log::error!("Invalid matrix pixelGroup named constrained: {}", e);
                        Box::new(move |_: Option<&Pixel>| false) as ConsFunc
                    }
                };
                b as ConsFunc
            };
                funcs.push(fun);
            }

            Box::new(move |x| {
                funcs.iter().all(|f| f(x))
            })
        }

        if let Some(map) = src.as_object() {
            for (k, v) in map {
                m.groups.push(k.clone());
                match v {
                    Value::String(all) if all == "all" => {
                        for x in &mut m.pixels {
                            for y in x {
                                for z in y.iter_mut().flatten() {
                                    z.groups.push(k.clone());
                                }
                            }
                        }
                    }
                    Value::Array(a) => {
                        let a = a.iter().map(|v| v.as_str().ok_or("Wrong pixelGroup type in array").map(|s| s.to_string())).collect::<Result<Vec<_>, _>>()?;
                        for x in &mut m.pixels {
                            for y in x {
                                for z in y.iter_mut().flatten() {
                                    for sa in &a {
                                        if z.key == *sa {
                                            z.groups.push(k.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Value::Object(obj) => {
                        let xf = make_axis_constrained(obj.get("x").unwrap_or(&Value::Null));
                        let yf = make_axis_constrained(obj.get("y").unwrap_or(&Value::Null));
                        let zf = make_axis_constrained(obj.get("z").unwrap_or(&Value::Null));
                        let name = make_name_constrained(obj.get("name").unwrap_or(&Value::Null));

                        for (ix, x) in m.pixels.iter_mut().enumerate() {
                            for (iy, y) in x.iter_mut().enumerate() {
                                for (iz, z) in y.iter_mut().enumerate() {
                                    if xf(ix) && yf(iy) && zf(iz) && name(z.as_ref()) {
                                        match z {
                                            None => {log::debug!("pixelGroup matches even tho pixel is None: Obj: {obj:?}");}
                                            Some(l) => {l.groups.push(k.clone());}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => { return Err("Invalid pixelGroup type".into()); },
                }

            }
                Ok(m)
        } else {
            log::error!("Matrix pixelGroups not an object: {:?}", src);
            Ok(m)
        }
    }


    let decide = (src["pixelCount"].is_null(), src["pixelKeys"].is_null());

    match decide { (true, true) | (false, false) => {
        Err("either 'pixelCount' or 'pixelKeys' is required".into())
    }
        (true, false) => {
            Ok(Some(add_pixel_groups(parse_pixel_keys(&src["pixelKeys"])?, &src["pixelGroups"])?))
        }
        (false, true) => {
            Ok(Some(add_pixel_groups(parse_pixel_count(&src["pixelCount"])?, &src["pixelGroups"])?))
        }
    }

}

#[cfg(test)]
mod test {
    use crate::convert::gen_each_pixel;

    #[test]
    fn test_gen_each_pixel() {
        let dims = [2,2,2];
        let g = gen_each_pixel(dims[0], dims[1], dims[2]);
        let r = vec!["(1, 1, 1)", "(2, 1, 1)", "(1, 2, 1)", "(2, 2, 1)", "(1, 1, 2)", "(2, 1, 2)", "(1, 2, 2)", "(2, 2, 2)"];
        assert_eq!(r, g);
    }
}