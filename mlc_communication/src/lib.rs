pub use remoc;
use remoc::prelude::*;
use std::io::{Read, Write};
use std::sync::Arc;

use macro_rules_attribute::derive_alias;
use mlc_data::DynamicResult;
use remoc::codec::{Codec, DeserializationError, SerializationError};
use remoc::rtc::ServerBase;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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

pub type CommCodec = remoc::codec::Default;

pub trait ServiceIdentifiableServer<T: Send + Sync + 'static>: ServiceIdentifiable {
    type S;

    fn spinup(
        service: Arc<T>,
        socket_rx: OwnedReadHalf,
        socket_tx: OwnedWriteHalf,
    ) -> impl Future<Output = DynamicResult<()>>
    where
        <Self as ServiceIdentifiableServer<T>>::S: ServerBase,
        <Self as ServiceIdentifiableServer<T>>::S: ServerShared<T, CommCodec>,
        <<Self as ServiceIdentifiableServer<T>>::S as ServerBase>::Client: Clone + RemoteSend,
    {
        async {
            let (server, client) = Self::S::new(service, 1);

            remoc::Connect::io::<_, _, _, _, CommCodec>(
                remoc::Cfg::default(),
                socket_rx,
                socket_tx,
            )
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

#[derive(Clone, Serialize, Deserialize)]
pub struct BsonCodec;

impl Codec for BsonCodec {
    fn serialize<Writer, Item>(mut writer: Writer, item: &Item) -> Result<(), SerializationError>
    where
        Writer: Write,
        Item: Serialize,
    {
        let d = bson::to_vec(item).map_err(SerializationError::new)?;
        writer.write_all(&d).map_err(SerializationError::new)
    }

    fn deserialize<Reader, Item>(reader: Reader) -> Result<Item, DeserializationError>
    where
        Reader: Read,
        Item: DeserializeOwned,
    {
        bson::from_reader(reader).map_err(DeserializationError::new)
    }
}
