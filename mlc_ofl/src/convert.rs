use std::collections::HashMap;
use serde_json::Value;
use mlc_data::fixture::blueprint::{ChannelIdentifier, FixtureBlueprint, Metadata, Mode, Physical, Pixel, PixelMatrix};
use mlc_data::project::ToFileName;

pub fn convert(ofl_source: &Value, manufacturer: String) -> Result<FixtureBlueprint, Box<dyn std::error::Error>> {
    let meta = parse_metadata(ofl_source, manufacturer)?;

    let matrix = parse_matrix(&ofl_source["matrix"])?;


    let modes = parse_modes(matrix.as_ref(), &ofl_source["modes"])?;


    let wheels = {
        log::warn!("Wheel parsing not yet implemented");
        None
    };


    let channels = {
        log::warn!("Channel parsing not yet implemented");
        HashMap::new()
    };

    Ok(FixtureBlueprint {
        meta,
        matrix,
        modes,
        wheels,
        channels
    })
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

    fn parse_channel(m: Option<&PixelMatrix>,src: &Value) -> Result<Vec<ChannelIdentifier>, Box<dyn std::error::Error>> {
        if src.is_null() { return Ok(vec![ChannelIdentifier::default()]) }
        if let Some(str) = src.as_str() {return Ok(vec![str.to_string()]) }

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
                Value::Array(arr ) => arr.iter().map(|v| v.as_str().map(|s| s.to_string())).collect::<Option<Vec<_>>>().map(|v| RepeatFor::Keys(v)),
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

        //TODO: Gen channels

        let channels = match channel_order {
            ChannelOrder::PerPixel => {
                pixel_keys.iter().flat_map(|pixel| {
                    template_channels.iter().map(|template| {
                        match template {
                            Some(t ) => t.replace("$pixelKey",  pixel),
                            None => String::new()
                        }
                    })
                }).collect::<Vec<_>>()
            }
            ChannelOrder::PerChannel => {
                template_channels.iter().flat_map(|template| {
                    pixel_keys.iter().map(move |pixel| {
                        match template {
                            None => String::new(),
                            Some(t) => t.replace("$pixelKey", pixel)
                        }
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