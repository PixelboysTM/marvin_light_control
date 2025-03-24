use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use mlc_communication::remoc::prelude::*;
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;
use mlc_communication::services::general::{Alive, View};
use mlc_communication::{self as com, remoc};
use project::Project;
use server::setup_server;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tui::create_tui;

mod project;
mod server;
mod tui;

const DEFAULT_SERVER_PORT: u16 = 8181;

pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: Arc<RwLock<bool>>,
    info_subscribers: Sender<Info>,
    status_subscribers: Sender<String>,
}
pub type AServiceImpl = Arc<RwLock<ServiceImpl>>;

impl ServiceImpl {
    pub fn send_info(&self, info: Info) {
        if let Err(err) = self.info_subscribers.send(info) {
            eprintln!("SendInfo error: {err:#?}");
        }
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
        let rx = self.info_subscribers.subscribe();
        Ok(rx)
    }

    async fn status(&self) -> Result<Receiver<String>, CallError> {
        let rx = self.status_subscribers.subscribe();
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
        info_subscribers: remoc::rch::watch::channel(Info::Idle).0,
        status_subscribers: remoc::rch::watch::channel(String::new()).0,
    }));

    let task_cancel_token = CancellationToken::new();
    let mut task_handles = vec![];

    task_handles.spawn(setup_server(
        DEFAULT_SERVER_PORT,
        service_obj.clone(),
        task_cancel_token.clone(),
    ));
    task_handles.spawn(create_shutdown_notifier(
        service_obj.clone(),
        task_cancel_token.clone(),
    ));

    let should_tui_exit = Arc::new(RwLock::new(false));
    let tui_handle = tokio::spawn(create_tui(
        task_cancel_token.clone(),
        should_tui_exit.clone(),
        service_obj.clone(),
    ));

    trace!("This is a trace");
    debug!("This is a debug");
    info!("This is a info");
    warn!("This is a warning");
    error!("This is a error");

    for handle in task_handles {
        handle.await.unwrap();
    }

    info!("Aquire tui exit");
    *should_tui_exit.write().await = true;

    tui_handle.await.unwrap();
}

async fn create_shutdown_notifier(
    obj: Arc<RwLock<ServiceImpl>>,
    task_cancel_token: CancellationToken,
) {
    task_cancel_token.cancelled().await;
    let _ = obj.read().await.send_info(Info::Shutdown);
}

fn setup_logging() -> Result<(), fern::InitError> {
    tui_logger::init_logger(log::LevelFilter::Trace).expect("Hello");
    tui_logger::set_default_level(log::LevelFilter::Trace);
    Ok(())
}

pub trait SpawnExt<S> {
    fn spawn(&mut self, s: S);
}

impl<F> SpawnExt<F> for Vec<JoinHandle<F::Output>>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn spawn(&mut self, s: F) {
        self.push(tokio::spawn(s));
    }
}
