use crate::global_services::{AutosaveService, ShutdownService};
use crate::logging::setup_logging;
use crate::misc::ShutdownHandler;
use crate::project::create_default_project;
use crate::server::ServerService;
use crate::tui::TuiService;
use crate::universe::UniverseRuntimeService;
use log::error;
use misc::AdaptNotifier;
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;
use mlc_communication::services::general::{Alive, View};
use mlc_communication::services::project::ProjectServiceError;
use mlc_communication::{self as com, remoc::prelude::*};
use mlc_data::misc::ErrIgnore;
use mlc_ofl::OflLibrary;
use project::{get_base_app_dir, Project};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use universe::UniverseRuntimeController;
use crate::endpoints::EndpointsManagerService;

mod global_services;
mod logging;
mod misc;
mod project;
mod server;
mod tui;
mod universe;
mod endpoints;

const DEFAULT_SERVER_PORT: u16 = 8181;

pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: RwLock<bool>,
    info: Sender<Info>,
    status: Sender<String>,
    adapt_notifier: AdaptNotifier,
    ofl_library: OflLibrary,
    universe_runtime: Arc<UniverseRuntimeController>,
    shutdown: ShutdownHandler,
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
    async fn alive(&self) -> Result<Alive, CallError> {
        Ok(Alive)
    }
    async fn is_valid_view(&self, view: View) -> Result<bool, CallError> {
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
    async fn save(&self) -> Result<bool, CallError> {
        let mut p = match self.validate_project_mut().await {
            Ok(p) => p,
            Err(_e) => return Ok(false),
        };
        if let Err(e) = p.save().await.map_err(ProjectServiceError::SavingFailed) {
            self.send_info(Info::Warning {
                title: "Failed to save".to_string(),
                msg: e.to_string(),
            });
        }

        self.send_info(Info::Saved);
        self.status
            .send(format!("Saved Project '{}' to disk!", p.metadata.name))
            .ignore();

        Ok(true)
    }
}

pub struct MlcServiceResources {
    service_obj: AServiceImpl,
    shutdown: ShutdownHandler,
    adapt_notifier: AdaptNotifier,
}

pub struct MlcServiceResourcesBuilder {
    resources: MlcServiceResources,
    handles: Vec<JoinHandle<()>>,
}

impl MlcServiceResourcesBuilder {
    pub fn new(
        service_obj: AServiceImpl,
        shutdown_handler: ShutdownHandler,
        adapt_notifier: AdaptNotifier,
    ) -> Self {
        Self {
            resources: MlcServiceResources {
                service_obj,
                adapt_notifier,
                shutdown: shutdown_handler,
            },
            handles: vec![],
        }
    }

    pub fn add_service<S: MlcService<(), ()>>(&mut self, service: S) {
        self.add_complex_service(service, ());
    }

    pub fn add_complex_service<S: MlcService<I, O>, I, O>(&mut self, service: S, input: I) -> O {
        let (handle, out) = service.start(&self.resources, input);
        self.handles.spawn(handle);
        out
    }

    pub fn add_service_handle(&mut self, handle: JoinHandle<()>) {
        self.handles.push(handle);
    }

    pub fn addd_dynamic<F: Future<Output = ()> + Send + 'static>(&mut self, fut: F) {
        self.handles.spawn(fut);
    }

    pub async fn wait(self) {
        for handle in self.handles {
            handle.await.unwrap()
        }
    }
}

pub trait MlcService<I = (), O = ()> {
    fn start(
        self,
        resources: &MlcServiceResources,
        i: I,
    ) -> (impl Future<Output = ()> + Send + 'static, O);
}

pub trait MlcServiceSimple {
    fn start(self, resources: &MlcServiceResources) -> impl Future<Output = ()> + Send + 'static;
}

impl<S: MlcServiceSimple> MlcService for S {
    fn start(
        self,
        resources: &MlcServiceResources,
        _: (),
    ) -> (impl Future<Output = ()> + Send + 'static, ()) {
        (<S as MlcServiceSimple>::start(self, resources), ())
    }
}

#[tokio::main]
async fn main() {
    let log_rx = setup_logging().unwrap();

    let project = Arc::new(RwLock::new(create_default_project()));
    let adapt_notifier = AdaptNotifier::create();
    let shutdown_handler = ShutdownHandler::create();

    let lib_path = get_base_app_dir().join("library");
    tokio::fs::create_dir_all(&lib_path).await.ignore();

    let (universe_runtime_service, universe_runtime_controller) = UniverseRuntimeService::create();

    let service_obj = Arc::new(ServiceImpl {
        project,
        valid_project: RwLock::new(false),
        info: rch::watch::channel(Info::Idle).0,
        status: rch::watch::channel(String::new()).0,
        adapt_notifier: adapt_notifier.clone(),
        ofl_library: OflLibrary::create(lib_path.join("ofl.json")),
        universe_runtime: Arc::new(universe_runtime_controller),
        shutdown: shutdown_handler.clone(),
    });

    let mut service_handler =
        MlcServiceResourcesBuilder::new(service_obj, shutdown_handler, adapt_notifier);

    service_handler.add_service(universe_runtime_service);

    service_handler.add_service(ServerService);
    service_handler.add_service(ShutdownService);
    service_handler.add_service(AutosaveService);
    service_handler.add_service(EndpointsManagerService);

    service_handler.add_complex_service(TuiService, log_rx);

    service_handler.wait().await;
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
