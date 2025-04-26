use std::net::Ipv4Addr;

use crate::misc::{ShutdownHandler, ShutdownPhase};
use crate::{AServiceImpl, MlcServiceResources, MlcServiceSimple, DEFAULT_SERVER_PORT};
use mlc_communication::services::general::GeneralServiceIdent;
use mlc_communication::services::project::ProjectServiceIdent;
use mlc_communication::services::project_selection::ProjectSelectionServiceIdent;
use mlc_communication::{ServiceIdentifiable, ServiceIdentifiableServer};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::select;
use tracing::{error, info};

pub struct ServerService;

impl MlcServiceSimple for ServerService {
    fn start(self, res: &MlcServiceResources) -> impl Future<Output = ()> + Send + 'static {
        setup_server(
            DEFAULT_SERVER_PORT,
            res.service_obj.clone(),
            res.shutdown.clone(),
        )
    }
}

async fn setup_server(port: u16, service_obj: AServiceImpl, shutdown: ShutdownHandler) {
    info!("Starting Server...");

    info!("Listening on port {}", port);
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
                info!("Shutting down Server! Not accepting new connections anymore.");
                break;
            }
            conn = listener.accept() => {
                info!("New connection");
                let (socket, addr) = conn.unwrap();
                handle_connection(&service_obj, socket, addr);
            }
        }

        tokio::task::yield_now().await;
    }

    shutdown.wait(ShutdownPhase::Phase2).await;
    info!("Shutting down Server! Not listening anymore.");
}

fn handle_connection(
    service_obj: &AServiceImpl,
    socket: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
) {
    let service_obj = service_obj.clone();
    tokio::spawn(async move {
        let (mut socket_rx, socket_tx) = socket.into_split();
        info!("Accepted connection from {}", addr);

        let mut buffer = [0; 5];
        let amount = socket_rx.read(&mut buffer).await.unwrap();
        if amount != 5 {
            error!("The first message wasn't 5 long rejecting!");
        }

        let ident = String::from_utf8_lossy(&buffer).to_string();

        info!("Got ident msg: {}", ident);

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
                error!("Identifier was not valid!");
                return;
            }
        };

        if let Err(e) = r {
            error!("{}", e);
        }
    });
}
