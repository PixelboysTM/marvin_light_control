use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::convert::convert;

mod convert;

const OFL_URL: &str = "https://open-fixture-library.org/download.ofl";

pub async fn create_lib(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let zip = reqwest::get(OFL_URL).await?.error_for_status()?.bytes().await?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip))?;

    let mut blueprints = vec![];

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if !file.is_file() { continue; }
        let s = file.name().split('/').map(|s| s.to_string()).collect::<Vec<_>>();
        let [manu, name, ..] = s.as_slice() else { log::error!("File structure not valid: {:?}", file.name()); continue; };
        log::info!("Parsing: {}:{}", manu, name);
        let data: String = String::from_utf8( file.bytes().collect::<Result<Vec<u8>, _>>()?)?;

        let blueprint = convert(&serde_json::from_str(&data)?, manu.to_string())?;
        blueprints.push(blueprint);
    }

    log::info!("Loaded {} blueprints", blueprints.len());
    log::info!("Writing Blueprints to disk...");
    let out_file = File::create(path)?;

    serde_json::to_writer_pretty(&out_file, &blueprints)?;

    log::info!("Done writing Blueprints to disk!");

    Ok(())
}
