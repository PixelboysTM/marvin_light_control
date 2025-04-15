pub use remoc;
use remoc::prelude::*;
use std::sync::Arc;

use macro_rules_attribute::derive_alias;
use mlc_data::DynamicResult;
use remoc::rtc::ServerBase;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

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

pub trait ServiceIdentifiableServer<T: Send + Sync + 'static>: ServiceIdentifiable {
    type S;

    fn spinup(
        service: Arc<T>,
        socket_rx: OwnedReadHalf,
        socket_tx: OwnedWriteHalf,
    ) -> impl Future<Output = DynamicResult<()>>
    where
        <Self as ServiceIdentifiableServer<T>>::S: ServerBase,
        <Self as ServiceIdentifiableServer<T>>::S: ServerShared<T, remoc::codec::Default>,
        <<Self as ServiceIdentifiableServer<T>>::S as ServerBase>::Client: Clone + RemoteSend,
    {
        async {
            let (server, client) = Self::S::new(service, 1);

            remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
                .provide(client)
                .await?;

            server.serve(true).await?;
            log::info!(
                "Closing connection for ident: {}",
                String::from_utf8_lossy(&Self::IDENT)
            );
            Ok(())
        }
    }
}
