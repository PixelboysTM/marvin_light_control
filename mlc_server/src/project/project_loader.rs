use crate::project::Project;
use lazy_static::lazy_static;
use mlc_data::project::{ProjectMetadata, ProjectType};
use mlc_data::DynamicResult;
use std::ffi::OsString;
use std::path::Path;
use std::str::FromStr;

pub type Plm = ProjectLoaderManager;
pub type BoxedLoader = Box<dyn ProjectLoader + Send + Sync>;

pub struct ProjectLoaderManager;

lazy_static! {
    static ref LOADERS: Vec<BoxedLoader> = vec![Box::new(JsonLoader), Box::new(BsonLoader)];
}

impl ProjectLoaderManager {
    pub fn loaders() -> &'static [BoxedLoader] {
        &LOADERS
    }

    pub fn for_file(path: &Path) -> Option<&BoxedLoader> {
        Self::loaders().iter().find(|&loader| {
            path.extension()
                .unwrap_or(&OsString::from_str(".").expect("Must be"))
                == loader.kind().extension()
        })
    }

    pub fn for_kind(kind: &ProjectType) -> Option<&BoxedLoader> {
        Self::loaders()
            .iter()
            .find(|&loader| loader.kind() == *kind)
    }
}

pub trait ProjectLoader {
    fn kind(&self) -> ProjectType;

    fn load_metadata(&self, data: Vec<u8>) -> DynamicResult<ProjectMetadata>;
    fn load_project(&self, data: Vec<u8>) -> DynamicResult<Project>;

    fn store_project(&self, data: &Project) -> DynamicResult<Vec<u8>>;
}

pub struct Json5Loader;

impl ProjectLoader for Json5Loader {
    fn kind(&self) -> ProjectType {
        ProjectType::Json
    }

    fn load_metadata(&self, data: Vec<u8>) -> DynamicResult<ProjectMetadata> {
        Ok(json5::from_str(&String::from_utf8(data)?)?)
    }

    fn load_project(&self, data: Vec<u8>) -> DynamicResult<Project> {
        Ok(json5::from_str(&String::from_utf8(data)?)?)
    }

    fn store_project(&self, data: &Project) -> DynamicResult<Vec<u8>> {
        Ok(json5::to_string(data)?.into_bytes())
    }
}

pub struct JsonLoader;

impl ProjectLoader for JsonLoader {
    fn kind(&self) -> ProjectType {
        ProjectType::Json
    }

    fn load_metadata(&self, data: Vec<u8>) -> DynamicResult<ProjectMetadata> {
        Ok(serde_json::from_str(&String::from_utf8(data)?)?)
    }

    fn load_project(&self, data: Vec<u8>) -> DynamicResult<Project> {
        Ok(serde_json::from_str(&String::from_utf8(data)?)?)
    }

    fn store_project(&self, data: &Project) -> DynamicResult<Vec<u8>> {
        Ok(serde_json::to_string(data)?.into_bytes())
    }
}
pub struct BsonLoader;

impl ProjectLoader for BsonLoader {
    fn kind(&self) -> ProjectType {
        ProjectType::Binary
    }

    fn load_metadata(&self, data: Vec<u8>) -> DynamicResult<ProjectMetadata> {
        Ok(bson::from_slice(&data)?)
    }

    fn load_project(&self, data: Vec<u8>) -> DynamicResult<Project> {
        Ok(bson::from_slice(&data)?)
    }

    fn store_project(&self, data: &Project) -> DynamicResult<Vec<u8>> {
        Ok(bson::to_vec(data)?)
    }
}
