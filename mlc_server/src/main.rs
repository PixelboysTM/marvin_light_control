use crate::project::create_default_project;
use bson::datetime;
use log::{debug, error, info, trace, warn};
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;
use mlc_communication::services::general::{Alive, View};
use mlc_communication::services::project::ProjectServiceError;
use mlc_communication::{self as com, remoc::prelude::*};
use mlc_data::misc::ErrIgnore;
use mlc_data::DynamicResult;
use mlc_ofl::OflLibrary;
use project::{get_base_app_dir, Project};
use server::setup_server;
use std::fmt::write;
use std::fs::OpenOptions;
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::{FmtSpan, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{layer, time, MakeWriter};
use tracing_subscriber::Layer;
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
    async fn save(&self) -> Result<bool, CallError> {
        let mut p = match self.validate_project_mut().await {
            Ok(p) => p,
            Err(_e) => return Ok(false),
        };
        if let Err(e) = p.save().await.map_err(ProjectServiceError::SavingFailed) {
            self.info
                .send(Info::Warning {
                    title: "Failed to save".to_string(),
                    msg: e.to_string(),
                })
                .ignore();
        }

        self.info.send(Info::Saved).ignore();
        self.status
            .send(format!("Saved Project '{}' to disk!", p.metadata.name))
            .ignore();

        Ok(true)
    }
}

#[tokio::main]
async fn main() {
    let log_rx = setup_logging().unwrap();

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
        log_rx,
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

fn setup_logging() -> DynamicResult<std::sync::mpsc::Receiver<Vec<u8>>> {
    use tracing_subscriber::prelude::*;

    let (w, rx) = FuturesWriter::new();

    let debug = {
        let mut d = false;
        #[cfg(debug_assertions)]
        {
            d = true;
        }
        d
    };

    let sub = tracing_subscriber::Registry::default()
        .with(
            layer()
                .compact()
                .with_ansi(false)
                .with_writer(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("server.log")?,
                )
                .with_filter(LevelFilter::from_level(Level::WARN)),
        )
        .with(
            layer()
                .with_ansi(false)
                .with_writer(
                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open("server-verbose.log")?,
                )
                .with_filter(LevelFilter::TRACE),
        )
        .with(
            layer()
                .compact()
                .with_writer(w)
                .with_timer(NiceTime)
                .with_ansi(true)
                // .with_target(true)
                .with_file(debug)
                .with_line_number(debug)
                .with_span_events(FmtSpan::FULL)
                .with_target(false)
                .with_filter(LevelFilter::INFO),
        );

    tracing::subscriber::set_global_default(sub)?;

    LogTracer::init()?;
    Ok(rx)
}

pub struct NiceTime;

impl FormatTime for NiceTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("[%H:%M.%S]"))
    }
}

struct FuturesWriter {
    sink: std::sync::mpsc::Sender<Vec<u8>>,
}

impl FuturesWriter {
    fn new() -> (Self, std::sync::mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = std::sync::mpsc::channel();

        (Self { sink: tx }, rx)
    }
}

impl Write for FuturesWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink
            .send(buf.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for FuturesWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        Self {
            sink: self.sink.clone(),
        }
    }
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
