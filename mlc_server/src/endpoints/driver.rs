use crate::endpoints::EndpointDriver;
use crate::universe::{UniverseUpdate, UniverseUpdateSubscriber};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::info;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub struct LogDriver {
    handle: Option<JoinHandle<()>>,
    tx: Sender<UniverseUpdateSubscriber>,
}

impl LogDriver {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::mpsc::channel::<UniverseUpdateSubscriber>(1);
        Self { handle: None, tx }
    }
}

impl EndpointDriver<()> for LogDriver {
    async fn stop_all(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    async fn apply_config(&mut self, sub: UniverseUpdateSubscriber, _config: ()) {
        self.handle.get_or_insert_with(|| {
            let (tx, rx) = tokio::sync::mpsc::channel::<UniverseUpdateSubscriber>(10);
            self.tx = tx;
            tokio::spawn(log_runner(rx))
        });
        self.tx.send(sub).await.expect("Why not!");
    }
}

async fn log_runner(mut rx: Receiver<UniverseUpdateSubscriber>) {
    let mut subs = vec![];

    async fn await_subs(
        subs: &mut [UniverseUpdateSubscriber],
    ) -> Option<Result<UniverseUpdate, RecvError>> {
        info!("Waiting for universe updates");
        
        let mut f = FuturesUnordered::new();
        for sub in subs {
            f.push(sub.recv());
        }
        f.next().await
    }

    loop {
        select! {
            sub = rx.recv() => {
                subs.push(sub.unwrap());
            }
            Some(m) = await_subs(&mut subs) => {
                match m {
                    Ok(u) => {
                        tracing::info!("Universe update: {:?}", u);
                    }
                    Err(e) => {
                        tracing::error!("Error getting universe update: {}", e);
                    }
                }
            }
        }
    }
}
