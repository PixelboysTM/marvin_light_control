use std::sync::Arc;

use fern::colors::{Color, ColoredLevelConfig};
use mlc_communication as com;
use mlc_communication::general_service::{Alive, View};
use mlc_communication::remoc::prelude::*;
use mlc_communication::{AnotherService, EchoService};
use project::Project;
use server::setup_server;
use tokio::sync::RwLock;

mod project;
mod server;
mod tui;
pub struct ServiceImpl {
    project: Arc<RwLock<Project>>,
    valid_project: Arc<RwLock<bool>>,
}

#[rtc::async_trait]
impl EchoService for ServiceImpl {
    async fn echo(&self, ping: String) -> Result<String, rtc::CallError> {
        log::info!("Got ping: {}", ping);
        Ok(ping)
    }
}

#[rtc::async_trait]
impl AnotherService for ServiceImpl {
    async fn hello(&self) -> Result<(), rtc::CallError> {
        log::debug!("Frontend says hello");
        Ok(())
    }
}

#[rtc::async_trait]
impl com::general_service::GeneralService for ServiceImpl {
    async fn alive(&self) -> Result<Alive, rtc::CallError> {
        Ok(Alive)
    }
    async fn is_valid_view(&self, view: View) -> Result<bool, rtc::CallError> {
        Ok(match view {
            View::Project => !*self.valid_project.read().await,
            View::Edit => *self.valid_project.read().await,
        })
    }
}

#[tokio::main]
async fn main() {
    setup_logging().unwrap();

    let project = Arc::new(RwLock::new(Project::new()));

    let service_obj = Arc::new(RwLock::new(ServiceImpl {
        project,
        valid_project: Arc::new(RwLock::new(false)),
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
