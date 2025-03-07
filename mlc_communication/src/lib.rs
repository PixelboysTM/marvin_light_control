pub use remoc;
use remoc::prelude::*;

use macro_rules_attribute::derive_alias;
derive_alias! {
    #[derive(Serde!)] = #[derive(serde::Serialize, serde::Deserialize)];
    #[derive(Com!)] = #[derive(Debug, Clone, Serde!)];
}

pub type ServiceIdentifier = [u8; 5];

pub trait ServiceIdentifiable {
    const IDENT: ServiceIdentifier;
    type Client: RemoteSend;
}

pub struct EchoServiceIdent;
impl ServiceIdentifiable for EchoServiceIdent {
    const IDENT: ServiceIdentifier = *b"echoi";
    type Client = EchoServiceClient;
}

#[rtc::remote]
pub trait EchoService {
    async fn echo(&self, ping: String) -> Result<String, rtc::CallError>;
}

pub struct AnotherServiceIdent;
impl ServiceIdentifiable for AnotherServiceIdent {
    const IDENT: ServiceIdentifier = *b"anoth";
    type Client = AnotherServiceClient;
}
#[rtc::remote]
pub trait AnotherService {
    async fn hello(&self) -> Result<(), rtc::CallError>;
}

pub mod general_service {
    use crate::{Com, Serde, ServiceIdentifiable, ServiceIdentifier};
    use macro_rules_attribute::derive;
    use remoc::rtc;

    pub struct GeneralServiceIdent;
    impl ServiceIdentifiable for GeneralServiceIdent {
        const IDENT: ServiceIdentifier = *b"genrl";
        type Client = GeneralServiceClient<remoc::codec::Bincode>;
    }

    #[derive(Com!)]
    pub struct Alive;

    #[derive(Com!)]
    pub enum View {
        Project,
        Edit,
    }

    #[rtc::remote]
    pub trait GeneralService {
        async fn alive(&self) -> Result<Alive, rtc::CallError>;
        async fn is_valid_view(&self, view: View) -> Result<bool, rtc::CallError>;
    }
}
