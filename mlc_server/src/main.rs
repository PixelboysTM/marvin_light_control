use std::pin::Pin;
use std::sync::Arc;

use crate::project::create_default_project;
use log::{debug, error, info, trace, warn};
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;
use mlc_communication::services::general::{Alive, View};
use mlc_communication::{self as com, remoc::prelude::*};
use mlc_data::misc::ErrIgnore;
use mlc_ofl::OflLibrary;
use project::{Project, get_base_app_dir};
use server::setup_server;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tui::create_tui;

mod project;
mod server;
mod tui;

const DEFAULT_SERVER_PORT: u16 = 8181;

pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: RwLock<bool>,
    info: Sender<Info>,
    status: Sender<String>,
    adapt_notifier: Arc<Notify>,
    ofl_library: mlc_ofl::OflLibrary,
}
pub type AServiceImpl = Arc<ServiceImpl>;

impl ServiceImpl {
    pub fn send_info(&self, info: Info) {
        if let Err(err) = self.info.send(info) {
            error!("SendInfo error: {err:#?}");
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
        let rx = self.info.subscribe();
        Ok(rx)
    }

    async fn status(&self) -> Result<Receiver<String>, CallError> {
        let rx = self.status.subscribe();
        Ok(rx)
    }
}

#[tokio::main]
async fn main() {
    setup_logging().unwrap();

    let project = Arc::new(RwLock::new(create_default_project()));
    let adapt_notifier = Arc::new(Notify::new());

    let lib_path = get_base_app_dir().join("library");
    tokio::fs::create_dir_all(&lib_path).await.ignore();

    let service_obj = Arc::new(ServiceImpl {
        project,
        valid_project: RwLock::new(false),
        info: rch::watch::channel(Info::Idle).0,
        status: rch::watch::channel(String::new()).0,
        adapt_notifier: adapt_notifier.clone(),
        ofl_library: OflLibrary::create(lib_path.join("ofl.json")),
    });

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
    task_handles.spawn(autosave_service(
        service_obj.clone(),
        adapt_notifier.clone(),
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

async fn create_shutdown_notifier(obj: AServiceImpl, task_cancel_token: CancellationToken) {
    task_cancel_token.cancelled().await;

    let mut p = obj.project.write().await;
    if obj.project_valid().await && p.settings.save_on_quit {
        p.save().await.unwrap();
    }
    obj.send_info(Info::Shutdown);
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

async fn autosave_service(
    service_obj: AServiceImpl,
    adapt_notifier: Arc<Notify>,
    shutdown: CancellationToken,
) {
    fn save_fut(p: &Project, valid: bool) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        if valid {
            if let Some(d) = &p.settings.autosave {
                Box::pin(tokio::time::sleep(*d))
            } else {
                Box::pin(futures::future::pending())
            }
        } else {
            Box::pin(futures::future::pending())
        }
    }

    loop {
        let duration = save_fut(
            &*service_obj.project.read().await,
            *service_obj.valid_project.read().await,
        );

        select! {
            _ = adapt_notifier.notified() => {
                continue;
            }
            _ = shutdown.cancelled() => {
                break;
            }
            _ = duration => {
                info!("Autosave triggered!");
                let _ = service_obj.project.write().await.save().await.map_err(|e| error!("{e:?}"));
                service_obj.send_info(Info::Autosaved);
            }
        }

        tokio::task::yield_now().await;
    }
}
