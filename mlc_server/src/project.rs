use mlc_data::{
    fixture::blueprint::FixtureBlueprint,
    project::{ProjectInformation, ProjectType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub project_information: ProjectInformation,
    pub fixture_blueprints: Vec<FixtureBlueprint>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            project_information: ProjectInformation {
                name: "Default invalid project".into(),
                last_saved: chrono::Local::now(),
                file_name: "".into(),
                project_type: ProjectType::default(),
            },
            fixture_blueprints: vec![],
        }
    }
}
