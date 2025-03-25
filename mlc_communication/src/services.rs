pub mod general {
    use crate::{Com, Serde, ServiceIdentifiable, ServiceIdentifier};
    use macro_rules_attribute::derive;
    use remoc::{rch::watch, rtc};

    pub struct GeneralServiceIdent;
    impl ServiceIdentifiable for GeneralServiceIdent {
        const IDENT: ServiceIdentifier = *b"genrl";
        type Client = GeneralServiceClient;
    }

    #[derive(Com!)]
    pub struct Alive;

    #[derive(Com!)]
    pub enum View {
        Project,
        Edit,
    }

    #[derive(Com!)]
    pub enum Info {
        Idle,
        Shutdown,
    }

    #[rtc::remote]
    pub trait GeneralService {
        async fn alive(&self) -> Result<Alive, rtc::CallError>;
        async fn is_valid_view(&self, view: View) -> Result<bool, rtc::CallError>;
        async fn info(&self) -> Result<watch::Receiver<Info>, rtc::CallError>;
        async fn status(&self) -> Result<watch::Receiver<String>, rtc::CallError>;
    }
}

pub mod project_selection {
    use remoc::rtc;
    use serde::{Deserialize, Serialize};
    use crate::{ServiceIdentifiable, ServiceIdentifier};
    use mlc_data::project::{ProjectMetadata, ProjectType};
    use mlc_data::uuid::Uuid;
    use remoc::rtc;

    pub struct ProjectSelectionServiceIdent;
    impl ServiceIdentifiable for ProjectSelectionServiceIdent {
        const IDENT: ServiceIdentifier = *b"prjsl";
        type Client = ProjectSelectionServiceClient;
    }

    #[rtc::remote]
    pub trait ProjectSelectionService {
        async fn create(&self, name: String, kind: ProjectType) -> Result<(), ProjectSelectionServiceError>;
        async fn list(&self) -> Result<Vec<ProjectMetadata>, ProjectSelectionServiceError>;
        async fn open(&self, id: Uuid) -> Result<bool, rtc::CallError>;
        async fn delete(&self, id: Uuid) -> Result<(), rtc::CallError>;
    }

    #[derive(Debug, thiserror::Error, Serialize, Deserialize)]
    pub enum ProjectSelectionServiceError {
        #[error("Failed to list all projects: {0:?}")]
        ProjectListError(String),

        #[error("Network communication error: {0:?}")]
        RemocError(#[from] rtc::CallError),

    }
}