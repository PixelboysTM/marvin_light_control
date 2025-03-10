use std::sync::Arc;

use fern::colors::{Color, ColoredLevelConfig};
use mlc_communication as com;
use mlc_communication::services::general::{Alive, View};
use mlc_communication::remoc::prelude::*;
use project::Project;
use server::setup_server;
use tokio::sync::RwLock;
use mlc_communication::remoc::rch::watch;
use mlc_communication::remoc::rch::watch::{Receiver, Sender};
use mlc_communication::remoc::rtc::CallError;
use mlc_communication::services::general::Info;

mod project;
mod server;
mod tui;
pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: Arc<RwLock<bool>>,
    info_subscribers: Arc<RwLock<Vec<Sender<Info>>>>,
    status_subscribers: Arc<RwLock<Vec<Sender<String>>>>,
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

    let server_handle = tokio::spawn(setup_server(8181, service_obj));
    // let tui_handle = tokio::spawn(create_tui());

    server_handle.await.unwrap();
    // tui_handle.await.unwrap();
}

fn setup_logging() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new().info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                chrono::Local::now().format("%H:%M.%S"),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
