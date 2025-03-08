use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::time::Instant;
use crate::convert::convert;

mod convert;

const OFL_URL: &str = "https://open-fixture-library.org/download.ofl";

pub async fn create_lib(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let time = Instant::now();

    let zip = reqwest::get(OFL_URL).await?.error_for_status()?.bytes().await?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip))?;
    
    let parsing = Instant::now();

    let mut blueprints = vec![];

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if !file.is_file() || file.name() == "manufacturers.json" { continue; }
        let s = file.name().split('/').map(|s| s.to_string()).collect::<Vec<_>>();
        let [manu, name, ..] = s.as_slice() else { log::error!("File structure not valid: {:?}", file.name()); continue; };
        log::info!("Parsing: {}:{}", manu, name);
        let data: String = String::from_utf8( file.bytes().collect::<Result<Vec<u8>, _>>()?)?;

        let blueprint = convert(&serde_json::from_str(&data)?, manu.to_string())?;
        blueprints.push(blueprint);
    }
    
    let parsing_time = parsing.elapsed();

    log::info!("Loaded {} blueprints in {parsing_time:?}", blueprints.len());
    log::info!("Writing Blueprints to disk...");
    let out_file = File::create(path)?;

    serde_json::to_writer_pretty(&out_file, &blueprints)?;

    let elapsed = time.elapsed();

    log::info!("Done writing Blueprints to disk! Overall time: {:?}", elapsed);

    Ok(())
}
