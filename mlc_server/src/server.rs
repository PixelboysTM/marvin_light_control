use std::net::Ipv4Addr;
use std::sync::Arc;

use mlc_communication::remoc::rtc::ServerBase;
use mlc_communication::remoc::{self, prelude::*};
use mlc_communication::{self as com, general_service};
use mlc_communication::{
    AnotherServiceIdent, AnotherServiceServerSharedMut, EchoServiceIdent,
    EchoServiceServerSharedMut, ServiceIdentifiable,
};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

use crate::ServiceImpl;

pub async fn setup_server(port: u16, service_obj: Arc<RwLock<ServiceImpl>>) {
    log::info!("Starting Server...");

    log::info!("Listening on port {}", port);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let (mut socket_rx, socket_tx) = socket.into_split();
        log::info!("Accepted connection from {}", addr);

        let mut buffer = [0; 5];
        let amount = socket_rx.read(&mut buffer).await.unwrap();
        if amount != 5 {
            log::error!("First message wasn't 5 long rejecting!");
        }

        let ident = String::from_utf8_lossy(&buffer).to_string();

        log::info!("Got ident msg: {}", ident);

        let service_obj = service_obj.clone();
        tokio::spawn(async move {
            match buffer {
                EchoServiceIdent::IDENT => {
                    create::<_, EchoServiceServerSharedMut<_>>(
                        service_obj,
                        socket_rx,
                        socket_tx,
                        ident,
                    )
                    .await
                    .unwrap();
                }
                AnotherServiceIdent::IDENT => {
                    create::<_, AnotherServiceServerSharedMut<_>>(
                        service_obj,
                        socket_rx,
                        socket_tx,
                        ident,
                    )
                    .await
                    .unwrap();
                }
                general_service::GeneralServiceIdent::IDENT => {
                    create::<_, general_service::GeneralServiceServerSharedMut<_>>(
                        service_obj,
                        socket_rx,
                        socket_tx,
                        ident,
                    )
                    .await
                    .unwrap();
                }
                _ => {
                    log::error!("Identifier was not valid!");
                }
            }
        });
    }
}

async fn create<T: Send + Sync + 'static, S: ServerSharedMut<T, remoc::codec::Default>>(
    service_obj: Arc<RwLock<T>>,
    socket_rx: OwnedReadHalf,
    socket_tx: OwnedWriteHalf,
    ident: String,
) -> Result<(), Box<dyn std::error::Error>>
where
    <S as ServerBase>::Client: RemoteSend,
{
    let (server, client) = S::new(service_obj, 1);

    remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
        .provide(client)
        .await?;

    server.serve(true).await?;
    log::info!("Closing connection for ident: {}", ident);
    Ok(())
}
