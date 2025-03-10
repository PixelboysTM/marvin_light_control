pub mod general {
    use crate::{Com, Serde, ServiceIdentifiable, ServiceIdentifier};
    use macro_rules_attribute::derive;
    use remoc::{rtc, rch::watch};

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
        Idle
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
    use mlc_data::project::{ProjectMetadata, ProjectType};
    use mlc_data::uuid::Uuid;
    use crate::{ServiceIdentifiable, ServiceIdentifier};

    pub struct ProjectSelectionServiceIdent;
    impl ServiceIdentifiable for ProjectSelectionServiceIdent {
        const IDENT: ServiceIdentifier = *b"prjsl";
        type Client = ProjectSelectionServiceClient;
    }

    #[rtc::remote]
    pub trait ProjectSelectionService {
        async fn create(&self, name: String, kind: ProjectType) -> Result<(), rtc::CallError>;
        async fn list(&self) -> Result<Vec<ProjectMetadata>, rtc::CallError>;
        async fn open(&self, id: Uuid) -> Result<bool, rtc::CallError>;
        async fn delete(&self, id: Uuid) -> Result<(), rtc::CallError>;
    }
}