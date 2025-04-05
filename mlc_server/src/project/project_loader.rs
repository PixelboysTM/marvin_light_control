use mlc_data::DynamicResult;
use mlc_data::project::{ProjectMetadata, ProjectType};
use crate::project::Project;


pub fn get_loaders() -> [Box<dyn ProjectLoader>; 2] {
    [Box::new(Json5Loader), Box::new(BsonLoader)]
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