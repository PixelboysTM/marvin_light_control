use crate::ServiceImpl;
use mlc_communication::remoc::rtc;
use mlc_communication::services::project_selection::{ProjectIdent, ProjectSelectionService, ProjectSelectionServiceError};
use mlc_data::{
    fixture::blueprint::FixtureBlueprint,
    project::{ProjectMetadata, ProjectType},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use chrono::Local;
use log::{error, info};
use tokio::io::AsyncReadExt;
use mlc_data::project::{ProjectSettings, ToFileName};
use crate::project::project_loader::get_loaders;

mod project_loader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub metadata: ProjectMetadata,
    pub blueprints: Vec<FixtureBlueprint>,
    pub settings: ProjectSettings,
}

fn to_pl_err(e: tokio::io::Error) -> ProjectSelectionServiceError {
    error!("tokio io error: {:?}", e);
    ProjectSelectionServiceError::ProjectListError(format!("{e:?}"))
}

fn to_pc_err(e: String) -> ProjectSelectionServiceError {
    ProjectSelectionServiceError::ProjectCreateError(e)
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
    ) -> Result<ProjectIdent, ProjectSelectionServiceError> {

        let mut p = create_default_project();
        p.metadata.name = name;
        p.metadata.created_at = Local::now();
        p.metadata.project_type = kind;
        p.metadata.file_name =  p.metadata.name.to_project_file_name();

        let identifier = p.metadata.file_name.clone();
        p.save().await.map_err(to_pc_err)?;

        //
        //
        // let projects_dir = get_base_app_dir().join("projects");
        // tokio::fs::create_dir_all(&projects_dir).await.map_err(to_pc_err)?;
        //
        // let path = projects_dir.join(format!("{}.{}", &identifier, kind.extension()));
        //
        // let bytes = {
        //     let loaders = get_loaders();
        //     let mut v = Vec::new();
        //     for loader in loaders {
        //         if loader.kind() == kind {
        //             v = loader.store_project(&p).map_err(|e| ProjectSelectionServiceError::ProjectCreateError(format!("{e:}")))?;
        //
        //         }
        //     }
        //
        //     if v.is_empty() {
        //         return Err(ProjectSelectionServiceError::ProjectListError("No suitable loader found".into()));
        //     }
        //
        //     v
        // };
        // tokio::fs::write(path, bytes).await.map_err(to_pc_err)?;

        Ok(identifier)
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

                            meta.file_name = file_name.split('.').next().expect("Must be").to_string();
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

    async fn open(&self, ident: ProjectIdent) -> Result<bool, ProjectSelectionServiceError> {
        let projects_dir = get_base_app_dir().join("projects");

        for format in ProjectType::all() {
            let path = projects_dir.join(format!("{ident}.{}", format.extension()));
            if path.exists() && path.is_file() {
                let mut content = tokio::fs::File::open(path)
                    .await
                    .map_err(to_po_err)?;
                let mut bytes = Vec::<u8>::new();
                content.read_to_end(&mut bytes).await.map_err(to_po_err)?;

                let (mut p, k): (Project, ProjectType) = match format {
                    ProjectType::Json => {
                        (json5::from_str(&String::from_utf8(bytes).map_err(to_po_err)?).map_err(to_po_err)?, ProjectType::Json)
                    }
                    ProjectType::Binary => {
                        (bson::from_slice(&bytes).map_err(to_po_err)?, ProjectType::Binary)
                    }
                    ProjectType::Invalid => {
                        unreachable!()
                    }
                };

                p.metadata.name = ident;
                p.metadata.project_type = k;
                {
                    *self.project.write().await = p;
                    *self.valid_project.write().await = true;
                }
                self.adapt_notifier.notify_waiters();

                return Ok(true);
            }
        }

        info!("Project with ident: {ident} not found");
        Ok(false)
    }

    async fn delete(&self, _ident: ProjectIdent) -> Result<(), ProjectSelectionServiceError> {
        unimplemented!()
    }
}

impl Project {
    fn new() -> Self {
        Self {
            metadata: ProjectMetadata {
                name: "Default invalid project".into(),
                last_saved: Local::now(),
                created_at: Local::now(),
                file_name: "".into(),
                project_type: ProjectType::default(),
            },
            blueprints: vec![],
            settings: ProjectSettings {
                autosave: Some(Duration::from_secs(30 * 60)),
                save_on_quit: true,
            }
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


impl Project {
    pub async fn save(&mut self) -> Result<(), String> {
        let identifier = self.metadata.file_name.clone();
        let kind = self.metadata.project_type.clone();

        self.metadata.last_saved = Local::now();
        self.metadata.file_name = "".to_owned();
        self.metadata.project_type = ProjectType::Invalid;



        let projects_dir = get_base_app_dir().join("projects");
        tokio::fs::create_dir_all(&projects_dir).await.map_err(|e| format!("{e:?}"))?;

        let path = projects_dir.join(format!("{}.{}", &identifier, kind.extension()));

        let bytes = {
            let loaders = get_loaders();
            let mut v = Vec::new();
            for loader in loaders {
                if loader.kind() == kind {
                    v = loader.store_project(self).map_err(|e| format!("{e:}"))?;

                }
            }

            if v.is_empty() {
                return Err("No suitable saver found".into());
            }

            v
        };
        tokio::fs::write(path, bytes).await.map_err(|e| format!("{e:?}"))?;


        self.metadata.file_name = identifier;
        self.metadata.project_type = kind;

        Ok(())
    }
}