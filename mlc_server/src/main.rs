use std::sync::Arc;
use std::time::Duration;

use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, trace, warn};
use mlc_communication as com;
use mlc_communication::remoc::prelude::*;
use mlc_communication::remoc::rch::watch;
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;
use mlc_communication::services::general::{Alive, View};
use project::Project;
use server::setup_server;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tui::create_tui;

mod project;
mod server;
mod tui;
pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: Arc<RwLock<bool>>,
    info_subscribers: Arc<RwLock<Vec<Sender<Info>>>>,
    status_subscribers: Arc<RwLock<Vec<Sender<String>>>>,
}

impl ServiceImpl {
    pub async fn send_info(&self, info: Info) {
        let mut is = self.info_subscribers.write().await;
        is.retain(|s| {
            let r = s.send(info.clone());
            if let Err(e) = r {
                match e {
                    watch::SendError::Closed => {
                        debug!("InfoSubscriber connection closed")
                    }
                    watch::SendError::RemoteSend(send_error_kind) => {
                        error!("InfoSubscriber Error RemoteSend: {send_error_kind:?}")
                    }
                    watch::SendError::RemoteConnect(connect_error) => {
                        error!("InfoSubscriber Error RemoteConnect: {connect_error:?}")
                    }
                    watch::SendError::RemoteListen(listener_error) => {
                        error!("InfoSubscriber Error RemoteListen: {listener_error:?}")
                    }
                    watch::SendError::RemoteForward => {
                        error!("InfoSubscriber Error RemmoteForward")
                    }
                }
                false
            } else {
                true
            }
        });
    }
}

#[rtc::async_trait]
impl com::services::general::GeneralService for ServiceImpl {
    async fn alive(&self) -> Result<Alive, rtc::CallError> {
        Ok(Alive)
    }
    async fn is_valid_view(&self, view: View) -> Result<bool, rtc::CallError> {
        Ok(match view {
            View::Project => !*self.valid_project.read().await,
            View::Edit => *self.valid_project.read().await,
        })
    }

    async fn info(&self) -> Result<Receiver<Info>, CallError> {
        let (tx, rx) = watch::channel(Info::Idle);

        self.info_subscribers.write().await.push(tx);

        Ok(rx)
    }

    async fn status(&self) -> Result<Receiver<String>, CallError> {
        let (tx, rx) = watch::channel(String::new());

        self.status_subscribers.write().await.push(tx);

        Ok(rx)
    }
}

#[tokio::main]
async fn main() {
    setup_logging().unwrap();

    let project = Arc::new(RwLock::new(Project::new()));

    let service_obj = Arc::new(RwLock::new(ServiceImpl {
        project,
        valid_project: Arc::new(RwLock::new(false)),
        info_subscribers: Arc::new(RwLock::new(Vec::new())),
        status_subscribers: Arc::new(RwLock::new(Vec::new())),
    }));
    // let s2 = service_obj.clone();

    let server_handle = tokio::spawn(setup_server(8181, service_obj));
    let tui_handle = tokio::spawn(create_tui());

    // let idle = tokio::spawn(async move {
    //     loop {
    //         sleep(Duration::from_secs(2)).await;
    //         s2.read().await.send_info(Info::Idle).await;
    //     }
    // });

    trace!("This is a trace");
    debug!("This is a debug");
    info!("This is a info");
    warn!("This is a warning");
    error!("This is a error");

    // idle.await.unwrap();
    server_handle.await.unwrap();
    tui_handle.await.unwrap();
}

fn setup_logging() -> Result<(), fern::InitError> {
    tui_logger::init_logger(log::LevelFilter::Trace).expect("Hello");
    tui_logger::set_default_level(log::LevelFilter::Trace);
    Ok(())
}
