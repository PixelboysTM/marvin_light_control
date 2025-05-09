use crate::misc::{AdaptScopes, ShutdownPhase};
use crate::project::project_loader::Plm;
use crate::universe::{RuntimeCommand, UniverseUpdate};
use crate::ServiceImpl;
use chrono::Local;
use mlc_communication::remoc::rch::mpsc::{Receiver, Sender};
use mlc_communication::remoc::rtc;
use mlc_communication::services::general::{Info, ProjectInfo};
use mlc_communication::services::project::{
    FixtureBlueprintHead, ProjectService, ProjectServiceError,
};
use mlc_communication::services::project_selection::{
    ProjectIdent, ProjectSelectionService, ProjectSelectionServiceError,
};
use mlc_data::endpoints::{EndpointConfig, EndpointMapping, EndpointSpeed};
use mlc_data::misc::ErrIgnore;
use mlc_data::project::universe::{
    FixtureAddress, FixtureUniverse, UniverseAddress, UniverseId, UniverseSlot, UNIVERSE_SIZE,
};
use mlc_data::project::{ProjectSettings, ToFileName};
use mlc_data::{
    fixture::blueprint::FixtureBlueprint,
    project::{ProjectMetadata, ProjectType},
    DynamicResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::select;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};
use tracing::{error, info, warn};

mod project_loader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub metadata: ProjectMetadata,
    pub blueprints: Vec<FixtureBlueprint>,
    pub settings: ProjectSettings,
    pub universes: Vec<FixtureUniverse>,
    pub endpoint_mapping: EndpointMapping,
}

#[rtc::async_trait]
impl ProjectService for ServiceImpl {
    async fn list_available_fixture_blueprints(
        &self,
    ) -> Result<Vec<FixtureBlueprintHead>, ProjectServiceError> {
        let heads = self
            .ofl_library
            .read(Some(|s| {
                info!("OflLoadMsg: {s}");
                self.status.send(s).ignore();
            }))
            .await
            .map_err(|e| ProjectServiceError::BlueprintListFailed(e.to_string()))?
            .into_iter()
            .map(|fb| FixtureBlueprintHead {
                meta: fb.meta,
                modes: fb.modes.into_iter().map(|m| m.name).collect(),
                num_channels: fb.channels.len() as u32,
            })
            .collect::<Vec<_>>();

        Ok(heads)
    }

    async fn import_fixture_blueprints(
        &self,
        identifiers: Vec<String>,
    ) -> Result<(), ProjectServiceError> {
        let mut blueprints = self
            .ofl_library
            .read(Some(|s| {
                info!("OflLoadMsg: {s}");
                self.status.send(s).ignore();
            }))
            .await
            .map_err(|e| ProjectServiceError::BlueprintListFailed(e.to_string()))?
            .into_iter()
            .filter(|b| identifiers.contains(&b.meta.identifier))
            .collect::<Vec<_>>();

        if blueprints.len() != identifiers.len() {
            self.send_info(Info::Warning {
                title: "Blueprints not found".to_string(),
                msg: "Not all specified blueprints could be found".to_string(),
            });
        }
        let mut p = self.project.write().await;

        p.blueprints
            .retain(|b| !identifiers.contains(&b.meta.identifier));
        p.blueprints.append(&mut blueprints);
        p.blueprints
            .sort_by(|b1, b2| b1.meta.identifier.cmp(&b2.meta.identifier));

        self.send_info(ProjectInfo::BlueprintsChanged.into());

        Ok(())
    }

    async fn list_blueprints(&self) -> Result<Vec<FixtureBlueprint>, ProjectServiceError> {
        Ok(self.project.read().await.blueprints.clone())
    }

    async fn universe_list(&self) -> Result<Vec<UniverseId>, ProjectServiceError> {
        Ok(self
            .project
            .read()
            .await
            .universes
            .iter()
            .enumerate()
            .map(|(i, _)| i as u16 + 1)
            .collect::<Vec<_>>())
    }

    async fn universe_sub(
        &self,
        universe: UniverseId,
    ) -> Result<
        (
            Receiver<(UniverseAddress, u8)>,
            Sender<(UniverseAddress, u8)>,
        ),
        ProjectServiceError,
    > {
        let (tx_1, rx_1) =
            mlc_communication::remoc::rch::mpsc::channel::<(UniverseAddress, u8), _>(100);
        let (tx_2, mut rx_2) =
            mlc_communication::remoc::rch::mpsc::channel::<(UniverseAddress, u8), _>(100);

        let controller = self.universe_runtime.clone();

        let shutdown = self.shutdown.clone();
        tokio::task::spawn(async move {
            let mut sub = controller.subscribe_universe(universe);
            loop {
                select! {
                    _ = shutdown.wait(ShutdownPhase::Phase1) => break,
                    Ok(update) = sub.recv() => {
                        match update {
                            UniverseUpdate::Single{ update } => {
                                tx_1.send((update.0.address(), update.1)).await.debug_ignore();
                            }
                            UniverseUpdate::Many{ updates } => {
                                for update in updates {
                                    tx_1.send((update.0.address(), update.1)).await.debug_ignore();
                                }
                            }
                            UniverseUpdate::Entire{ values, .. } => {
                                for (i, update) in values.into_iter().enumerate() {
                                    tx_1.send((UniverseAddress::create(i + 1), update)).await.debug_ignore();
                                }
                            }
                        }
                    }
                    Ok(ch) = rx_2.recv() => {
                        if let Some(update) = ch {
                            controller.cmd(RuntimeCommand::UpdateData(UniverseUpdate::Single {
                                update: (FixtureAddress::new(universe, update.0), update.1),
                            }))
                        }
                    }
                }
            }
        });

        Ok((rx_1, tx_2))
    }

    async fn get_settings(&self) -> Result<ProjectSettings, ProjectServiceError> {
        let p = self.validate_project().await?;
        Ok(p.settings.clone())
    }

    async fn update_settings(&self, settings: ProjectSettings) -> Result<(), ProjectServiceError> {
        let mut p = self.validate_project_mut().await?;
        p.settings = settings;
        self.adapt_notifier.notify(AdaptScopes::SETTINGS);
        self.send_info(ProjectInfo::SettingsChanged.into());
        Ok(())
    }

    async fn get_meta(&self) -> Result<ProjectMetadata, ProjectServiceError> {
        let p = self.validate_project().await?;
        Ok(p.metadata.clone())
    }
}

impl ServiceImpl {
    pub async fn project_valid(&self) -> bool {
        *self.valid_project.read().await
    }

    pub async fn validate_project(
        &self,
    ) -> Result<RwLockReadGuard<'_, Project>, ProjectServiceError> {
        if !self.project_valid().await {
            Err(ProjectServiceError::InvalidProject)
        } else {
            Ok(self.project.read().await)
        }
    }

    pub async fn validate_project_mut(
        &self,
    ) -> Result<RwLockWriteGuard<'_, Project>, ProjectServiceError> {
        if !self.project_valid().await {
            Err(ProjectServiceError::InvalidProject)
        } else {
            Ok(self.project.write().await)
        }
    }
}

fn to_pl_err(e: tokio::io::Error) -> ProjectSelectionServiceError {
    error!("tokio io error: {:?}", e);
    ProjectSelectionServiceError::ProjectListError(format!("{e:?}"))
}

fn to_pc_err(e: String) -> ProjectSelectionServiceError {
    ProjectSelectionServiceError::ProjectCreateError(e)
}

fn to_po_err<E: error::Error>(e: E) -> ProjectSelectionServiceError {
    error!("project open error: {:?}", e);
    ProjectSelectionServiceError::ProjectOpenError(format!("{e:?}"))
}

async fn get_valid_project_dir() -> DynamicResult<PathBuf> {
    let projects_dir = get_base_app_dir().join("projects");
    tokio::fs::create_dir_all(&projects_dir)
        .await
        .map_err(to_pl_err)?;
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

    for i in 0..=u32::MAX {
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
        p.metadata.file_name = make_save_file_name(&p.metadata.name, &kind)
            .await
            .map_err(|e| ProjectSelectionServiceError::ProjectCreateError(e.to_string()))?;

        let identifier = p.metadata.file_name.clone();
        p.save().await.map_err(to_pc_err)?;

        Ok(identifier)
    }

    async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError> {
        let mut projects = vec![];

        let projects_dir = get_valid_project_dir()
            .await
            .map_err(|e| ProjectSelectionServiceError::ProjectListError(e.to_string()))?;

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
                                        meta.file_name = file_name
                                            .split('.')
                                            .next()
                                            .expect("Must be")
                                            .to_string();
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
                error!("Couldn't get base project dir: {e}");
            }
        }

        Ok(projects)
    }

    async fn open(&self, ident: ProjectIdent) -> Result<bool, ProjectSelectionServiceError> {
        let projects_dir = get_valid_project_dir()
            .await
            .map_err(|e| ProjectSelectionServiceError::ProjectOpenError(e.to_string()))?;

        for loader in Plm::loaders() {
            let path = projects_dir.join(format!("{}.{}", &ident, loader.kind().extension()));
            if path.exists() && path.is_file() {
                let content = tokio::fs::read(path).await.map_err(to_po_err)?;
                let mut p = loader.load_project(content).map_err(|e| {
                    ProjectSelectionServiceError::ProjectOpenError(format!("{e:?}"))
                })?;

                p.metadata.project_type = loader.kind();
                p.metadata.file_name = ident.clone();

                {
                    *self.project.write().await = p;
                    *self.valid_project.write().await = true;
                }
                self.adapt_notifier.notify(AdaptScopes::all());
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
            },
            universes: vec![FixtureUniverse {
                addresses: [UniverseSlot::Unused; UNIVERSE_SIZE],
                fixtures: HashMap::new(),
            }],
            endpoint_mapping: EndpointMapping {
                endpoints: HashMap::new(),
            },
        }
    }
}

pub fn create_default_project() -> Project {
    Project::new()
}

pub fn get_base_app_dir() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("de", "timfritzen", "marvin_light_control")
        .expect("Could not get the project directory");
    project_dirs.data_dir().to_path_buf()
}

impl Project {
    pub async fn save(&mut self) -> Result<(), String> {
        let identifier = self.metadata.file_name.clone();
        let kind = self.metadata.project_type;

        self.metadata.last_saved = Local::now();
        self.metadata.file_name = "".to_owned();
        self.metadata.project_type = ProjectType::Invalid;

        let projects_dir = get_valid_project_dir().await.map_err(|e| e.to_string())?;

        let path = projects_dir.join(format!("{}.{}", &identifier, kind.extension()));

        let loader = Plm::for_kind(&kind).ok_or(format!("No saver found for {kind:?}"))?;
        let data = loader.store_project(self).map_err(|e| format!("{e:}"))?;
        tokio::fs::write(path, data)
            .await
            .map_err(|e| format!("{e:?}"))?;

        self.metadata.file_name = identifier;
        self.metadata.project_type = kind;

        Ok(())
    }
}
