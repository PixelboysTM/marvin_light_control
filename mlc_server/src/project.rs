use crate::ServiceImpl;
use mlc_communication::remoc::rtc;
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::project_selection::ProjectSelectionService;
use mlc_data::uuid::Uuid;
use mlc_data::{
    fixture::blueprint::FixtureBlueprint,
    project::{ProjectMetadata, ProjectType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub project_information: ProjectMetadata,
    pub fixture_blueprints: Vec<FixtureBlueprint>,
}

#[rtc::async_trait]
impl ProjectSelectionService for ServiceImpl {
    async fn create(&self, name: String, kind: ProjectType) -> Result<(), CallError> {
        todo!()
    }

    async fn list(&self) -> Result<Vec<ProjectMetadata>, CallError> {
        todo!()
    }

    async fn open(&self, id: Uuid) -> Result<bool, CallError> {
        todo!()
    }

    async fn delete(&self, id: Uuid) -> Result<(), CallError> {
        todo!()
    }
}

impl Project {
    pub fn new() -> Self {
        Self {
            project_information: ProjectMetadata {
                name: "Default invalid project".into(),
                last_saved: chrono::Local::now(),
                created_at: chrono::Local::now(),
                file_name: "".into(),
                project_type: ProjectType::default(),
                id: Uuid::new_v4(),
            },
            fixture_blueprints: vec![],
        }
    }
}
