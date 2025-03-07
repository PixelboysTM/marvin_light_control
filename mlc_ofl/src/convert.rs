use std::collections::HashMap;
use serde_json::Value;
use mlc_data::fixture::blueprint::{FixtureBlueprint, Metadata, Physical, PixelGroupIdentifier, PixelMatrix};
use mlc_data::project::ToFileName;

pub fn convert(ofl_source: &Value, manufacturer: String) -> Result<FixtureBlueprint, Box<dyn std::error::Error>> {
    let meta = parse_metadata(ofl_source, manufacturer)?;

    let matrix = parse_matrix(&ofl_source["matrix"])?;


    let modes = {
        log::warn!("Mode parsing not yet implemented");
        vec![]
    };


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
        z.resize(array[2],Some(Vec::new()));

        let mut y = Vec::with_capacity(array[1]);
        y.resize(array[1], z);

        let mut x = Vec::with_capacity(array[0]);
        x.resize(array[0], y);
        
        let mut i = 1;
        
        for xs in &mut x {
            for ys in xs {
                for zs in ys {
                    if let Some(v) = zs {
                        v.push(i.to_string());
                        i += 1;
                    }
                }
            }
        }

        Ok(PixelMatrix {
            matrix: x,
        })
    }

    fn parse_pixel_keys(src: &Value) -> Result<PixelMatrix, Box<dyn std::error::Error>> {
        let m = src.as_array().ok_or("Matrix pixelKeys not an array")?.iter().map(|v|
            v.as_array().ok_or("Matrix pixelKeys not an 3d array").map(|a| a.iter().map(|v|
                v.as_array().ok_or("Matrix pixelKeys not an 3d array").map(|a| a.iter().map(|v| v.as_str().map(|s| vec![s.to_string() as PixelGroupIdentifier])).collect::<Vec<_>>())).collect::<Result<Vec<_>, _>>())).collect::<Result<Vec<_>, _>>()?.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(PixelMatrix{
            matrix: m
        })
    }

    fn add_pixel_groups(mut m: PixelMatrix, src: &Value) -> Result<PixelMatrix, Box<dyn std::error::Error>> {
        if src.is_null() {
            return Ok(m);
        }


        fn make_axis_constrained(src: &Value) -> Box<dyn Fn(usize) -> bool> {
            let s = src.as_str().unwrap_or("");

            if s.len() == 0 { Box::new(|_| true) }
            else if s.starts_with("<=") {let n = s[2..].parse::<usize>().unwrap_or(0); Box::new(move |x| x <= n)}
            else if s.starts_with(">=") {let n = s[2..].parse::<usize>().unwrap_or(0); Box::new(move |x| x >= n)}
            else if s.starts_with("<") {let n = s[1..].parse::<usize>().unwrap_or(0); Box::new(move |x| x < n)}
            else if s.starts_with(">") {let n = s[1..].parse::<usize>().unwrap_or(0); Box::new(move |x| x > n)}
            else if s.starts_with("=") {let n = s[1..].parse::<usize>().unwrap_or(0); Box::new(move |x| x == n)}
            else if s == "even" {Box::new(|x| x % 2 == 0)}
            else if s == "odd" {Box::new(|x| x % 2 == 1)}
            else {
                let ss = s.split('n').collect::<Vec<&str>>();
                if ss.len() == 2 && ss[1].starts_with("+") {
                    let x1 = ss[1].parse::<usize>().unwrap_or(0);
                    let x2 = ss[1].parse::<usize>().unwrap_or(0);
                    Box::new(move |x: usize| x % x1 == x2)
                } else if ss.len() == 2 && ss[1] == "" {
                    let n = ss[0].parse::<usize>().unwrap_or(0);
                    Box::new(move |x: usize| x % n == 0)
                } else {
                    log::error!("Invalid matrix pixelGroup axis constrained");
                    Box::new(|_| false)
                }
            }
        }

        fn make_name_constrained(src: &Value) -> Box<dyn Fn(Option<&Vec<PixelGroupIdentifier>>) -> bool> {

            if src.is_null() {
                return Box::new(|_| true);
            }

            let empty = vec![];
            let ss = src.as_array().unwrap_or_else(|| {
                log::error!("Matrix name constrained not an array");
                &empty
            });
            let mut funcs: Vec<Box<dyn Fn(Option<&Vec<PixelGroupIdentifier>>) -> bool>> = vec![];

            for s in ss {
                let s = s.as_str().unwrap_or("");
            let fun = if s.is_empty() { Box::new(move |_: Option<&Vec<PixelGroupIdentifier>>| true) as Box<dyn Fn(Option<&Vec<PixelGroupIdentifier>>) -> bool>  }
            else {
                let regex = regex::Regex::new(s);
                let b = match regex {
                    Ok(regex) => {
                        Box::new(move |idents: Option<&Vec<PixelGroupIdentifier>>| {
                            let r  = if let Some(idents) = idents { idents.iter().map(|ident| regex.is_match(ident)).fold(false, |acc, m| acc || m) } else {
                                false
                            };
                            r
                        })
                    }
                    Err(e) => {
                        log::error!("Invalid matrix pixelGroup named constrained: {}", e);
                        Box::new(move |_: Option<&Vec<PixelGroupIdentifier>>| false) as Box<dyn Fn(Option<&Vec<PixelGroupIdentifier>>) -> bool>
                    }
                };
                b as Box<dyn Fn(Option<&Vec<PixelGroupIdentifier>>) -> bool>
            };
                funcs.push(fun);
            }

            Box::new(move |x| {
                funcs.iter().fold(true, |acc, f| acc && f(x))
            })
        }

        //TODO: Search on clone to not insert based on inserted
        if let Some(map) = src.as_object() {
            for (k, v) in map {
                match v {
                    Value::String(all) if all == "all" => {
                        for x in &mut m.matrix {
                            for y in x {
                                for z in y {
                                    if let Some(l) = z {
                                        l.push(k.clone());
                                    }
                                }
                            }
                        }
                    }
                    Value::Array(a) => {
                        //TODO: Does not work as expected
                        // log::debug!("using pixelGroup array");
                        let a = a.iter().map(|v| v.as_str().ok_or("Wrong pixelGroup type in array").map(|s| s.to_string())).collect::<Result<Vec<_>, _>>()?;
                        for x in &mut m.matrix {
                            for y in x {
                                for z in y {
                                    if let Some(l) = z {
                                        for sa in &a {
                                            if l.contains(sa) {
                                                l.push(k.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Value::Object(obj) => {
                        let xf = make_axis_constrained(&obj.get("x").unwrap_or(&Value::Null));
                        let yf = make_axis_constrained(&obj.get("y").unwrap_or(&Value::Null));
                        let zf = make_axis_constrained(&obj.get("z").unwrap_or(&Value::Null));
                        let name = make_name_constrained(&obj.get("name").unwrap_or(&Value::Null));

                        for (ix, x) in m.matrix.iter_mut().enumerate() {
                            for (iy, y) in x.iter_mut().enumerate() {
                                for (iz, z) in y.iter_mut().enumerate() {
                                    if xf(ix) && yf(iy) && zf(iz) && name(z.as_ref()) {
                                        match z {
                                            None => {log::debug!("pixelGroup matches even tho pixel is None: Obj: {obj:?}");}
                                            Some(l) => {l.push(k.clone());}
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