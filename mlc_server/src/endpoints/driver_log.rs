use crate::endpoints::{await_subs, EndpointDriver};
use crate::universe::{UniverseUpdate, UniverseUpdateSubscriber};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::info;

pub struct LogDriver {
    handle: Option<JoinHandle<()>>,
    tx: Sender<UniverseUpdateSubscriber>,
    shutdown_notify: Arc<Notify>,
}

impl LogDriver {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::mpsc::channel::<UniverseUpdateSubscriber>(1);
        Self {
            handle: None,
            tx,
            shutdown_notify: Arc::new(Notify::new()),
        }
    }
}

impl EndpointDriver<()> for LogDriver {
    async fn stop_all(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.shutdown_notify.notify_one();
            let _ = handle.await;
        }
    }

    async fn apply_config(&mut self, sub: UniverseUpdateSubscriber, _config: ()) {
        self.handle.get_or_insert_with(|| {
            let (tx, rx) = tokio::sync::mpsc::channel::<UniverseUpdateSubscriber>(10);
            self.tx = tx;
            tokio::spawn(log_runner(rx, self.shutdown_notify.clone()))
        });
        self.tx.send(sub).await.expect("Why not!");
    }
}

async fn log_runner(mut rx: Receiver<UniverseUpdateSubscriber>, shutdown: Arc<Notify>) {
    let mut subs = vec![];

    'o: loop {
        select! {
            sub = rx.recv() => {
                subs.push(sub.unwrap());
            }
            _ = shutdown.notified() => {
                break 'o;
            }
            Some(m) = await_subs(&mut subs) => {
                match m {
                    Ok(u) => {
                        info!("Universe update: {:?}", u);
                    }
                    Err(e) => {
                        tracing::error!("Error getting universe update: {}", e);
                    }
                }
            }
        }
    }
}
