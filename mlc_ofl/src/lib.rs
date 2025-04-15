use crate::convert::convert;
use mlc_data::fixture::blueprint::FixtureBlueprint;
use mlc_data::misc::ContextError;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

mod convert;

const OFL_URL: &str = "https://open-fixture-library.org/download.ofl";

pub async fn create_lib(path: &Path, pretty: bool) -> Result<(), Box<dyn std::error::Error>> {
    let time = Instant::now();

    let zip = reqwest::get(OFL_URL)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip))?;

    let parsing = Instant::now();

    let mut blueprints = vec![];

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if !file.is_file() || file.name() == "manufacturers.json" {
            continue;
        }
        let s = file
            .name()
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let [manu, name, ..] = s.as_slice() else {
            log::error!("File structure not valid: {:?}", file.name());
            continue;
        };
        log::info!("Parsing: {}:{}", manu, name);
        let data: String = String::from_utf8(file.bytes().collect::<Result<Vec<u8>, _>>()?)?;

        #[cfg(debug_assertions)]
        {
            if format!("{}:{}", manu, name) == "" {
                log::debug!("Halt");
            }
        }

        let blueprint = convert(&serde_json::from_str(&data)?, manu.to_string())
            .map_err(ContextError::to_generic)?;
        blueprints.push(blueprint);
    }

    let parsing_time = parsing.elapsed();

    log::info!("Loaded {} blueprints in {parsing_time:?}", blueprints.len());
    log::info!("Writing Blueprints to disk...");
    let out_file = File::create(path)?;

    if pretty {
        serde_json::to_writer_pretty(out_file, &blueprints)?;
    } else {
        serde_json::to_writer(out_file, &blueprints)?;
    }

    let elapsed = time.elapsed();

    log::info!(
        "Done writing Blueprints to disk! Overall time: {:?}",
        elapsed
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct OflLibrary {
    state: Arc<Mutex<OflState>>,
    library_path: Arc<PathBuf>,
}

impl OflLibrary {
    pub fn create(path: PathBuf) -> Self {
        OflLibrary {
            state: Arc::new(Mutex::new(OflState::Uninitialized)),
            library_path: Arc::new(path),
        }
    }

    pub async fn init<C>(&self, status_callback: Option<C>)
    where
        C: Fn(String),
    {
        if let Some(c) = &status_callback {
            c("Waiting for OflLibrary lock!".to_string());
        }
        while *self.state.lock().await != OflState::Idle
            && *self.state.lock().await != OflState::Uninitialized
        {
            tokio::task::yield_now().await;
        }

        *self.state.lock().await = OflState::Loading;

        if let Some(c) = &status_callback {
            c("Downloading data...".to_string());
        }

        let _ = create_lib(&self.library_path, false).await;

        *self.state.lock().await = OflState::Idle;

        if let Some(c) = &status_callback {
            c("Written ofl data!".to_string());
        }
    }

    pub async fn read<C>(
        &self,
        status_callback: Option<C>,
    ) -> Result<Vec<FixtureBlueprint>, Box<dyn std::error::Error>>
    where
        C: Fn(String),
    {
        if *self.state.lock().await == OflState::Uninitialized {
            self.init(status_callback.as_ref().map(|c| |s| c(s)))
            .await;
        }

        while *self.state.lock().await != OflState::Idle {
            tokio::task::yield_now().await;
        }

        *self.state.lock().await = OflState::Reading;
        if let Some(c) = &status_callback {
            c("Reading ofl data...".to_string());
        }

        let data = tokio::fs::read(self.library_path.as_ref()).await?;
        let data: Vec<FixtureBlueprint> = serde_json::from_slice(&data)?;

        *self.state.lock().await = OflState::Idle;
        if let Some(c) = &status_callback {
            c("Done loading ofl data!".to_string());
        }
        Ok(data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OflState {
    Uninitialized,
    Loading,
    Idle,
    Reading,
}
