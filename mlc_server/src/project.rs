use std::path::PathBuf;
use mlc_data::{fixture::blueprint::FixtureBlueprint, project::{ProjectMetadata, ProjectType}, uuid};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use mlc_communication::remoc::rtc;
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::project_selection::{ProjectSelectionService, ProjectSelectionServiceError};
use mlc_data::uuid::Uuid;
use crate::ServiceImpl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub metadata: ProjectMetadata,
    pub blueprints: Vec<FixtureBlueprint>,
}

fn to_pl_err(e: tokio::io::Error) -> ProjectSelectionServiceError {
    ProjectSelectionServiceError::ProjectListError(format!("{e:?}"))
}

#[rtc::async_trait]
impl ProjectSelectionService for ServiceImpl {
    async fn create(&self, name: String, kind: ProjectType) -> Result<(), ProjectSelectionServiceError> {
        todo!()
    }

    async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError> {

        let mut projects = vec![];
        if let Ok(path) = get_base_app_dir().join("projects").canonicalize() {
            if !path.exists() {
                tokio::fs::create_dir_all(path.clone()).await.map_err(to_pl_err)?;
            }

            let mut files = tokio::fs::read_dir(path).await.map_err(to_pl_err)?;

            'file_iter: loop {
                if let Some(file) = files.next_entry().await.map_err(to_pl_err)? {
                    let file_name = file.file_name().to_string_lossy().to_string();
                    if file.file_type().await.map_err(to_pl_err)?.is_file(){
                        let mut content = tokio::fs::File::open(file.path()).await.map_err(to_pl_err)?;
                        let mut s = String::new();
                        content.read_to_string(&mut s).await.map_err(to_pl_err)?;
                        let meta: ProjectMetadata =
                        if file_name.ends_with(ProjectType::Json.dotted_extension()) {
                            json5::from_str(&s).map_err(|e| ProjectSelectionServiceError::ProjectListError(format!("Couldn't read Metadata {e}")))?
                        } else if file_name.ends_with(ProjectType::Binary.dotted_extension()) {
                            bson::from_slice(s.as_bytes()).map_err(|e| ProjectSelectionServiceError::ProjectListError(format!("Couldn't read Metadata {e}")))?
                        } else {
                            continue;
                        };

                        projects.push(meta);

                    }
                } else {
                    break 'file_iter;
                }
            }

        }else { 
            log::error!("Couldn't get base project dir");
        }

        Ok(projects)
    }

    async fn open(&self, id: Uuid) -> Result<bool, CallError> {
        todo!()
    }

    async fn delete(&self, id: Uuid) -> Result<(), CallError> {
        todo!()
    }
}

impl Project {
    fn new() -> Self {
        Self {
            metadata: ProjectMetadata {
                name: "Default invalid project".into(),
                last_saved: chrono::Local::now(),
                created_at: chrono::Local::now(),
                file_name: "".into(),
                project_type: ProjectType::default(),
                id: Uuid::new_v4(),
            },
            blueprints: vec![],
        }
    }
}

pub fn create_default_project() -> Project {
    Project::new()
}


fn get_base_app_dir() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("de", "timfritzen", "marvin_light_control")
        .expect("Could not get project directory");
    project_dirs.data_dir().to_path_buf()
}