use crate::ServiceImpl;
use mlc_communication::remoc::rtc;
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::project_selection::{
    ProjectSelectionService, ProjectSelectionServiceError,
};
use mlc_data::uuid::Uuid;
use mlc_data::{
    fixture::blueprint::FixtureBlueprint,
    project::{ProjectMetadata, ProjectType},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{Local, Utc};
use log::error;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub metadata: ProjectMetadata,
    pub blueprints: Vec<FixtureBlueprint>,
}

fn to_pl_err(e: tokio::io::Error) -> ProjectSelectionServiceError {
    error!("tokio io error: {:?}", e);
    ProjectSelectionServiceError::ProjectListError(format!("{e:?}"))
}

fn to_pc_err<E: std::error::Error>(e: E) -> ProjectSelectionServiceError {
    error!("project create error: {:?}", e);
    ProjectSelectionServiceError::ProjectCreateError(format!("{e:?}"))
}

fn to_po_err<E: std::error::Error>(e: E) -> ProjectSelectionServiceError {
    error!("project open error: {:?}", e);
    ProjectSelectionServiceError::ProjectOpenError(format!("{e:?}"))
}

#[rtc::async_trait]
impl ProjectSelectionService for ServiceImpl {
    async fn create(
        &self,
        name: String,
        kind: ProjectType,
    ) -> Result<Uuid, ProjectSelectionServiceError> {

        let mut p = create_default_project();
        p.metadata.name = name;
        p.metadata.created_at = Local::now();
        p.metadata.last_saved = Local::now();


        let projects_dir = get_base_app_dir().join("projects");
        tokio::fs::create_dir_all(&projects_dir).await.map_err(to_pc_err)?;

        let path = projects_dir.join(format!("{}.{}", p.metadata.id, kind.extension()));

        let bytes = match kind {
            ProjectType::Json => {
                json5::to_string(&p).map_err(to_pc_err)?.into_bytes()
            }
            ProjectType::Binary => {
                bson::to_vec(&p).map_err(to_pc_err)?
            }
        };
        tokio::fs::write(path, bytes).await.map_err(to_pc_err)?;

        Ok(p.metadata.id)
    }

    async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError> {
        let mut projects = vec![];

        let projects_dir = get_base_app_dir().join("projects");
        tokio::fs::create_dir_all(&projects_dir).await.map_err(to_pl_err)?;

        match projects_dir.canonicalize() {
            Ok(path) => {
                if !path.exists() {
                    tokio::fs::create_dir_all(path.clone())
                        .await
                        .map_err(to_pl_err)?;
                }

                let mut files = tokio::fs::read_dir(path).await.map_err(to_pl_err)?;

                'file_iter: loop {
                    if let Some(file) = files.next_entry().await.map_err(to_pl_err)? {
                        let file_name = file.file_name().to_string_lossy().to_string();
                        if file.file_type().await.map_err(to_pl_err)?.is_file() {
                            let mut content = tokio::fs::File::open(file.path())
                                .await
                                .map_err(to_pl_err)?;
                            let mut meta: ProjectMetadata = if file_name
                                .ends_with(ProjectType::Json.dotted_extension())
                            {
                                let mut s = String::new();
                                content.read_to_string(&mut s).await.map_err(to_pl_err)?;
                                let mut m: ProjectMetadata = json5::from_str(&s).map_err(|e| {
                                    ProjectSelectionServiceError::ProjectListError(format!(
                                        "Couldn't read Metadata {e}"
                                    ))
                                })?;
                                m.project_type = ProjectType::Json;
                                m
                            } else if file_name.ends_with(ProjectType::Binary.dotted_extension()) {
                                let mut buffer = Vec::new();
                                content.read_to_end(&mut buffer).await.map_err(to_pl_err)?;
                                let mut m: ProjectMetadata = bson::from_slice(&buffer).map_err(|e| {
                                    ProjectSelectionServiceError::ProjectListError(format!(
                                        "Couldn't read Binary Metadata {e:}"
                                    ))
                                })?;
                                m.project_type = ProjectType::Binary;
                                m
                            } else {
                                continue;
                            };

                            meta.file_name = file_name.to_string();
                            projects.push(meta);
                        }
                    } else {
                        break 'file_iter;
                    }
                }
            }
            Err(e) => {
                log::error!("Couldn't get base project dir: {e}");
            }
        }

        Ok(projects)
    }

    async fn open(&self, id: Uuid) -> Result<bool, ProjectSelectionServiceError> {
        let projects_dir = get_base_app_dir().join("projects");

        for format in ProjectType::all() {
            let path = projects_dir.join(format!("{id}.{}", format.extension()));
            if path.exists() && path.is_file() {
                let mut content = tokio::fs::File::open(path)
                    .await
                    .map_err(to_po_err)?;
                let mut bytes = Vec::<u8>::new();
                content.read_to_end(&mut bytes).await.map_err(to_po_err)?;

                let p: Project = match format {
                    ProjectType::Json => {
                        json5::from_str(&String::from_utf8(bytes).map_err(to_po_err)?).map_err(to_po_err)?
                    }
                    ProjectType::Binary => {
                        bson::from_slice(&bytes).map_err(to_po_err)?
                    }
                };

                *self.project.write().await = p; //TODO: Adapt any other services that might need it effect baking endpoint mapping etc.

                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn delete(&self, id: Uuid) -> Result<(), ProjectSelectionServiceError> {
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
