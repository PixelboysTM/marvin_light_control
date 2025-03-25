use std::net::Ipv4Addr;
use std::sync::Arc;

use log::error;
use mlc_communication::remoc::rtc::ServerBase;
use mlc_communication::remoc::{self, prelude::*};
use mlc_communication::{ServiceIdentifiable, services::*};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::ServiceImpl;

pub async fn setup_server(
    port: u16,
    service_obj: Arc<RwLock<ServiceImpl>>,
    shutdown: CancellationToken,
) {
    log::info!("Starting Server...");

    log::info!("Listening on port {}", port);
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await {
        Ok(l) => l,
        Err(e) => {
            error!("Could not start server. Got error:\n {e:#?}");
            return;
        }
    };

    loop {
        select! {
            _ = shutdown.cancelled() => {
                log::info!("Shutting down Server! Not listening anymore.");
                break;
            }
            conn = listener.accept() => {
                log::info!("New connection");
                let (socket, addr) = conn.unwrap();
                handle_connection(&service_obj, socket, addr);
            }
        }

        tokio::task::yield_now().await;
    }
}

fn handle_connection(
    service_obj: &Arc<RwLock<ServiceImpl>>,
    socket: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
) {
    let service_obj = service_obj.clone();
    tokio::spawn(async move {
        let (mut socket_rx, socket_tx) = socket.into_split();
        log::info!("Accepted connection from {}", addr);

        let mut buffer = [0; 5];
        let amount = socket_rx.read(&mut buffer).await.unwrap();
        if amount != 5 {
            log::error!("First message wasn't 5 long rejecting!");
        }

        let ident = String::from_utf8_lossy(&buffer).to_string();

        log::info!("Got ident msg: {}", ident);

        match buffer {
            project_selection::ProjectSelectionServiceIdent::IDENT => {
                create::<_, project_selection::ProjectSelectionServiceServerSharedMut<_>>(
                    service_obj,
                    socket_rx,
                    socket_tx,
                    ident,
                )
                .await
                .unwrap();
            }
            general::GeneralServiceIdent::IDENT => {
                create::<_, general::GeneralServiceServerSharedMut<_>>(
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
