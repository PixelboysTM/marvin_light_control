use std::sync::Arc;

use mlc_data::{
    misc::ErrIgnore,
    project::universe::{FixtureAddress, UNIVERSE_SIZE, UniverseId},
};
use tokio::{
    select,
    sync::{
        RwLock,
        broadcast::{Receiver, Sender, error::RecvError},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{
    misc::{AdaptNotifer, AdaptScopes},
    project::Project,
};

#[derive(Debug)]
pub struct UniverseRuntime {
    runtime_universes: Vec<[u8; UNIVERSE_SIZE]>,
    update_notifier: Sender<UniverseUpdate>,
    cmd_recv: tokio::sync::mpsc::UnboundedReceiver<RuntimeCommand>,
    project: Arc<RwLock<Project>>,
}

pub struct UniverseRntimeController {
    update_subscriber: Sender<UniverseUpdate>,
    cmd_sender: tokio::sync::mpsc::UnboundedSender<RuntimeCommand>,
}

impl UniverseRntimeController {
    pub fn subscribe(&self) -> Receiver<UniverseUpdate> {
        let rx = self.update_subscriber.subscribe();

        self.cmd_sender
            .send(RuntimeCommand::ResendUniverses)
            .debug_ignore();

        rx
    }
    pub fn subscribe_universe(&self, universe: UniverseId) -> UniverseUpdateSubscriber {
        let sub = self.subscribe();
        UniverseUpdateSubscriber {
            rx: sub,
            universe_id: universe,
        }
    }

    pub fn cmd(&self, cmd: RuntimeCommand) {
        self.cmd_sender.send(cmd).debug_ignore();
    }
}

#[derive(Debug)]
pub struct UniverseUpdateSubscriber {
    rx: Receiver<UniverseUpdate>,
    universe_id: UniverseId,
}

impl UniverseUpdateSubscriber {
    pub async fn recv(&mut self) -> Result<UniverseUpdate, RecvError> {
        loop {
            let msg = self.rx.recv().await?;
            match msg {
                UniverseUpdate::Single { update } => {
                    if update.0.universe() == self.universe_id {
                        return Ok(UniverseUpdate::Single { update });
                    }
                }
                UniverseUpdate::Many { mut updates } => {
                    updates.retain(|u| u.0.universe() == self.universe_id);
                    if !updates.is_empty() {
                        return Ok(UniverseUpdate::Many { updates });
                    }
                }
                UniverseUpdate::Entire { universe, values } => {
                    if universe == self.universe_id {
                        return Ok(UniverseUpdate::Entire { universe, values });
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeCommand {
    ResendUniverses,
    ResendUniverse(UniverseId),
}

pub type UpdateChunk = (FixtureAddress, u8);

#[derive(Debug, Clone)]
pub enum UniverseUpdate {
    Single {
        update: UpdateChunk,
    },
    Many {
        updates: Vec<UpdateChunk>,
    },
    Entire {
        universe: UniverseId,
        values: [u8; UNIVERSE_SIZE],
    },
}

impl UniverseRuntime {
    pub fn start(
        shutdown: CancellationToken,
        adapt_notifier: AdaptNotifer,
        project: Arc<RwLock<Project>>,
    ) -> (JoinHandle<()>, UniverseRntimeController) {
        let (update_tx, _update_rx) = tokio::sync::broadcast::channel(32);
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();

        let runtime = Self {
            runtime_universes: vec![],
            update_notifier: update_tx.clone(),
            cmd_recv: cmd_rx,
            project,
        };

        let j = runtime.spawn(shutdown, adapt_notifier);

        (
            j,
            UniverseRntimeController {
                update_subscriber: update_tx,
                cmd_sender: cmd_tx,
            },
        )
    }

    fn spawn(
        mut self,
        shutdown: CancellationToken,
        adapt_notifier: AdaptNotifer,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            log::info!("Starting Universe Runtime");
            loop {
                select! {
                    _ = shutdown.cancelled() => {
                        log::info!("Shutting down Universe Runtime!");
                        break;
                    }
                    _ = adapt_notifier.wait(AdaptScopes::UNIVERSES) => {
                        self.adapt().await;
                    }
                    Some(cmd) = self.cmd_recv.recv() => {
                        self.handle_cmd(cmd).await;
                    }
                }
            }
            log::info!("Exiting Universe Runtime");
        })
    }

    #[tracing::instrument]
    async fn handle_cmd(&mut self, cmd: RuntimeCommand) {
        log::trace!("Starting RuntimeCommand Handling");
        match cmd {
            RuntimeCommand::ResendUniverses => {
                for u in 1..=self.runtime_universes.len() {
                    self.send_universe(u as u16).await
                }
            }
            RuntimeCommand::ResendUniverse(i) => self.send_universe(i).await,
        }
        log::trace!("Finished RuntimeCommand Handling");
    }

    async fn send_universe(&mut self, universe: UniverseId) {
        if let Some(data) = self.runtime_universes.get(universe as usize - 1) {
            self.update_notifier
                .send(UniverseUpdate::Entire {
                    universe,
                    values: data.clone(),
                })
                .debug_ignore();
        } else {
            tracing::error!("Send Universe requested for invalid universe");
        };
    }

    #[tracing::instrument]
    async fn adapt(&mut self) {
        log::info!("Adapting");
        let p = self.project.read().await;
        self.runtime_universes
            .resize(p.universes.len(), [0; UNIVERSE_SIZE]);
        for (i, u) in self.runtime_universes.iter_mut().enumerate() {
            *u = [0; 512];
            self.update_notifier
                .send(UniverseUpdate::Entire {
                    universe: (i + 1) as u16,
                    values: u.clone(),
                })
                .debug_ignore();
        }
    }
}
