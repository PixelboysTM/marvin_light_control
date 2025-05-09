pub mod general {
    use crate::{Com, Serde, ServiceIdentifiable, ServiceIdentifiableServer, ServiceIdentifier};
    use macro_rules_attribute::derive;

    use remoc::rtc::{Deserialize, Serialize};
    use remoc::{rch::watch, rtc};

    pub struct GeneralServiceIdent;
    impl ServiceIdentifiable for GeneralServiceIdent {
        const IDENT: ServiceIdentifier = *b"genrl";
        type Client = GeneralServiceClient;
    }

    impl<T: GeneralService + Send + Sync + 'static> ServiceIdentifiableServer<T>
        for GeneralServiceIdent
    {
        type S = GeneralServiceServerShared<T>;
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
    pub struct Alive;

    #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
    pub enum View {
        Project,
        Edit,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum Info {
        Idle,
        Shutdown,
        Saved,
        Autosaved,
        Warning { title: String, msg: String },
        ProjectInfo { info: ProjectInfo },
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum ProjectInfo {
        BlueprintsChanged,
        UniverseListChanged,
        SettingsChanged,
    }

    impl From<ProjectInfo> for Info {
        fn from(value: ProjectInfo) -> Self {
            Self::ProjectInfo { info: value }
        }
    }

    #[rtc::remote]
    pub trait GeneralService {
        async fn alive(&self) -> Result<Alive, rtc::CallError>;
        async fn is_valid_view(&self, view: View) -> Result<bool, rtc::CallError>;
        async fn info(&self) -> Result<watch::Receiver<Info>, rtc::CallError>;
        async fn status(&self) -> Result<watch::Receiver<String>, rtc::CallError>;
        async fn save(&self) -> Result<bool, rtc::CallError>;
    }
}

pub mod project_selection {
    use crate::{ServiceIdentifiable, ServiceIdentifiableServer, ServiceIdentifier};
    use mlc_data::project::{ProjectMetadata, ProjectType};
    use remoc::rtc;
    use serde::{Deserialize, Serialize};

    pub struct ProjectSelectionServiceIdent;
    impl ServiceIdentifiable for ProjectSelectionServiceIdent {
        const IDENT: ServiceIdentifier = *b"prjsl";
        type Client = ProjectSelectionServiceClient;
    }
    impl<T: ProjectSelectionService + Send + Sync + 'static> ServiceIdentifiableServer<T>
        for ProjectSelectionServiceIdent
    {
        type S = ProjectSelectionServiceServerShared<T>;
    }

    pub type ProjectIdent = String;

    #[rtc::remote]
    pub trait ProjectSelectionService {
        async fn create(
            &self,
            name: String,
            kind: ProjectType,
        ) -> Result<ProjectIdent, ProjectSelectionServiceError>;
        async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError>;
        async fn open(&self, ident: ProjectIdent) -> Result<bool, ProjectSelectionServiceError>;
        async fn delete(&self, ident: ProjectIdent) -> Result<(), ProjectSelectionServiceError>;
    }

    #[derive(Debug, thiserror::Error, Serialize, Deserialize)]
    pub enum ProjectSelectionServiceError {
        #[error("Failed to list all projects: {0: }")]
        ProjectListError(String),

        #[error("Failed to create the project: {0: }")]
        ProjectCreateError(String),

        #[error("Failed to open project: {0: }")]
        ProjectOpenError(String),

        #[error("Failed to delete the project: {0: }")]
        ProjectDeleteError(String),

        #[error("Network communication error: {0:?}")]
        RemocError(#[from] rtc::CallError),
    }
}

pub mod project {
    use crate::{ServiceIdentifiable, ServiceIdentifiableServer, ServiceIdentifier};
    use mlc_data::fixture::blueprint::{FixtureBlueprint, Metadata};
    use mlc_data::project::universe::{UniverseAddress, UniverseId};
    use mlc_data::project::{ProjectMetadata, ProjectSettings};
    use remoc::rtc;
    use serde::{Deserialize, Serialize};

    pub struct ProjectServiceIdent;
    impl ServiceIdentifiable for ProjectServiceIdent {
        const IDENT: ServiceIdentifier = *b"prjts";
        type Client = ProjectServiceClient;
    }

    impl<T: ProjectService + Send + Sync + 'static> ServiceIdentifiableServer<T>
        for ProjectServiceIdent
    {
        type S = ProjectServiceServerShared<T>;
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct FixtureBlueprintHead {
        pub meta: Metadata,
        pub modes: Vec<String>,
        pub num_channels: u32,
    }

    #[rtc::remote]
    pub trait ProjectService {
        async fn list_available_fixture_blueprints(
            &self,
        ) -> Result<Vec<FixtureBlueprintHead>, ProjectServiceError>;
        async fn import_fixture_blueprints(
            &self,
            identifiers: Vec<String>,
        ) -> Result<(), ProjectServiceError>;
        async fn list_blueprints(&self) -> Result<Vec<FixtureBlueprint>, ProjectServiceError>;

        async fn universe_list(&self) -> Result<Vec<UniverseId>, ProjectServiceError>;
        async fn universe_sub(
            &self,
            universe: UniverseId,
        ) -> Result<
            (
                remoc::rch::mpsc::Receiver<(UniverseAddress, u8)>,
                remoc::rch::mpsc::Sender<(UniverseAddress, u8)>,
            ),
            ProjectServiceError,
        >;
        async fn get_settings(&self) -> Result<ProjectSettings, ProjectServiceError>;
        async fn update_settings(
            &self,
            settings: ProjectSettings,
        ) -> Result<(), ProjectServiceError>;

        async fn get_meta(&self) -> Result<ProjectMetadata, ProjectServiceError>;
    }

    #[derive(Debug, thiserror::Error, Serialize, Deserialize, Clone)]
    pub enum ProjectServiceError {
        #[error("It is no valid project loaded!")]
        InvalidProject,

        #[error("Saving Project Failed: {0: }")]
        SavingFailed(String),

        #[error("Listing available fixture blueprints failed: {0:?}")]
        BlueprintListFailed(String),

        #[error("Network communication error: {0:?}")]
        RemocError(#[from] rtc::CallError),
    }
}
