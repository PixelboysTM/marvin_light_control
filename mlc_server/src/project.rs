use crate::ServiceImpl;
use mlc_communication::remoc::rtc;
use mlc_communication::services::project_selection::{ProjectIdent, ProjectSelectionService, ProjectSelectionServiceError};
use mlc_data::{fixture::blueprint::FixtureBlueprint, project::{ProjectMetadata, ProjectType}, DynamicResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use chrono::Local;
use log::{error, info, warn};
use mlc_data::project::{ProjectSettings, ToFileName};
use crate::project::project_loader::Plm;

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

async fn get_valid_project_dir() -> DynamicResult<PathBuf> {
    let projects_dir = get_base_app_dir().join("projects");
    tokio::fs::create_dir_all(&projects_dir).await.map_err(to_pl_err)?;
    Ok(projects_dir)
}

async fn make_save_file_name(name: &str, kind: &ProjectType) -> DynamicResult<String> {
    let projects_dir = get_valid_project_dir().await?;

    let base = name.to_project_file_name();

    fn complete(name: &str, kind: &ProjectType) -> String {
        format!("{}.{}", name, kind.extension())
    }

    if !projects_dir.join(complete(&base, kind)).exists() {
        return Ok(base);
    }

    for i in 0.. {
        let indexed = format!("{base}_{i}");

        if projects_dir.join(complete(&indexed, kind)).exists() {
            continue;
        }

        return Ok(indexed);
    }

    Err(format!("Could not create save file name for name: {name}").into())
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
        p.metadata.file_name =  make_save_file_name(&p.metadata.name, &kind).await.map_err(|e| ProjectSelectionServiceError::ProjectCreateError(e.to_string()))?;

        let identifier = p.metadata.file_name.clone();
        p.save().await.map_err(to_pc_err)?;

        Ok(identifier)
    }

    async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError> {
        let mut projects = vec![];

        let projects_dir = get_valid_project_dir().await.map_err(|e| ProjectSelectionServiceError::ProjectListError(e.to_string()))?;

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
                            if let Some(loader) = Plm::for_file(&file.path()) {
                                let data = tokio::fs::read(file.path()).await.map_err(to_pl_err)?;
                                let meta = loader.load_metadata(data);
                                match meta {
                                    Ok(mut meta) => {
                                        meta.project_type = loader.kind();
                                        meta.file_name = file_name.split('.').next().expect("Must be").to_string();
                                        projects.push(meta);
                                    }
                                    Err(e) => {
                                        warn!("Error loading project metadata: {}", e);
                                    }
                                }
                            } else {
                                warn!("No suitable loader found for: {file_name}");
                            }
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
        let projects_dir = get_valid_project_dir().await.map_err(|e| ProjectSelectionServiceError::ProjectOpenError(e.to_string()))?;


        for loader in Plm::loaders() {
            let path = projects_dir.join(format!("{}.{}", &ident, loader.kind().extension()));
            if path.exists() && path.is_file() {
                let content = tokio::fs::read(path).await.map_err(to_po_err)?;
                let mut p = loader.load_project(content).map_err(|e| ProjectSelectionServiceError::ProjectOpenError(format!("{e:?}")))?;

                p.metadata.project_type = loader.kind();
                p.metadata.file_name = ident.clone();

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



        let projects_dir = get_valid_project_dir().await.map_err(|e| e.to_string())?;

        let path = projects_dir.join(format!("{}.{}", &identifier, kind.extension()));


        let loader = Plm::for_kind(&kind).ok_or(format!("No saver found for {kind:?}"))?;
        let data = loader.store_project(self).map_err(|e| format!("{e:}"))?;
        tokio::fs::write(path, data).await.map_err(|e| format!("{e:?}"))?;


        self.metadata.file_name = identifier;
        self.metadata.project_type = kind;

        Ok(())
    }
}