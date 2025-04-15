use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::misc::{ShutdownHandler, ShutdownPhase};
use crate::{AServiceImpl, ServiceImpl};
use log::error;
use mlc_communication::remoc::rtc::ServerBase;
use mlc_communication::remoc::{self, prelude::*};
use mlc_communication::services::general::GeneralServiceIdent;
use mlc_communication::services::project::ProjectServiceIdent;
use mlc_communication::services::project_selection::ProjectSelectionServiceIdent;
use mlc_communication::{services::*, ServiceIdentifiable, ServiceIdentifiableServer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

pub async fn setup_server(port: u16, service_obj: AServiceImpl, shutdown: ShutdownHandler) {
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
            _ = shutdown.wait(ShutdownPhase::Phase1) => {
                log::info!("Shutting down Server! Not accepting new connections anymore.");
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


    shutdown.wait(ShutdownPhase::Phase2).await;
    log::info!("Shutting down Server! Not listening anymore.");
}

fn handle_connection(
    service_obj: &AServiceImpl,
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

        // let service_idents: Vec<Box<dyn ServiceIdentifiableServer<ServiceImpl>>> = vec![
        //     Box::new(project_selection::ProjectSelectionServiceIdent),
        //     Box::new(general::GeneralServiceIdent),
        //     Box::new(project::ProjectServiceIdent),
        // ];

        let r = match buffer {
            ProjectSelectionServiceIdent::IDENT => {
                ProjectSelectionServiceIdent::spinup(service_obj, socket_rx, socket_tx).await
            }
            GeneralServiceIdent::IDENT => {
                GeneralServiceIdent::spinup(service_obj, socket_rx, socket_tx).await
            }
            ProjectServiceIdent::IDENT => {
                ProjectServiceIdent::spinup(service_obj, socket_rx, socket_tx).await
            }
            _ => {
                log::error!("Identifier was not valid!");
                return;
            }
        };

        if let Err(e) = r {
            error!("{}", e);
        }
    });
}
