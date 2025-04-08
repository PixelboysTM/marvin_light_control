pub use remoc;
use remoc::prelude::*;

use macro_rules_attribute::derive_alias;
derive_alias! {
    #[derive(Serde!)] = #[derive(serde::Serialize, serde::Deserialize)];
    #[derive(Com!)] = #[derive(Debug, Clone, Serde!)];
}

pub mod services;

pub type ServiceIdentifier = [u8; 5];

pub trait ServiceIdentifiable {
    const IDENT: ServiceIdentifier;
    type Client: remoc::rtc::Client + RemoteSend + Clone;
}
