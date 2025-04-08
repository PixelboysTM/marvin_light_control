use std::marker::PhantomData;
use std::sync::Arc;
pub use remoc;
use remoc::prelude::*;

use macro_rules_attribute::derive_alias;
use remoc::rtc::ServerBase;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use mlc_data::DynamicResult;

derive_alias! {
    #[derive(Serde!)] = #[derive(serde::Serialize, serde::Deserialize)];
    #[derive(Com!)] = #[derive(Debug, Clone, Serde!)];
}

pub mod services;

pub type ServiceIdentifier = [u8; 5];

pub trait ServiceIdentifiable {
    const IDENT: ServiceIdentifier;
    type Client: Client + RemoteSend + Clone;
}

pub trait ServiceIdentifiableServer<T: Send + Sync + 'static>: ServiceIdentifiable  {
    type S;

    async fn spinup(service: Arc<T>, socket_rx: OwnedReadHalf, socket_tx: OwnedWriteHalf) -> DynamicResult<()>
        where
            <Self as ServiceIdentifiableServer<T>>::S: ServerBase,
            <<Self as ServiceIdentifiableServer<T>>::S as ServerBase>::Client: Clone,
            <Self as ServiceIdentifiableServer<T>>::S: ServerShared<T, remoc::codec::Default>,
            <<Self as ServiceIdentifiableServer<T>>::S as ServerBase>::Client: RemoteSend,
    {
        let (server, client) = Self::S::new(service, 1);

        remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
            .provide(client)
            .await?;

        server.serve(true).await?;
        log::info!("Closing connection for ident: {}", String::from_utf8_lossy(&Self::IDENT).to_string());
        Ok(())
    }
}
