use crate::endpoints::{await_subs, EndpointDriver};
use crate::universe::{UniverseUpdate, UniverseUpdateSubscriber};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use mlc_data::endpoints::EndpointSpeed;
use mlc_data::project::universe::{UniverseId, UNIVERSE_SIZE};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::Interval;
use tracing::info;

pub struct SacnDriver {
    handle: Option<JoinHandle<()>>,
    tx: Sender<(UniverseUpdateSubscriber, (EndpointSpeed, u16))>,
    shutdown_notify: Arc<Notify>,
}

impl SacnDriver {
    pub fn new() -> Self {
        let (tx, _) =
            tokio::sync::mpsc::channel::<(UniverseUpdateSubscriber, (EndpointSpeed, u16))>(1);
        Self {
            handle: None,
            tx,
            shutdown_notify: Arc::new(Notify::new()),
        }
    }
}

impl EndpointDriver<(EndpointSpeed, u16)> for SacnDriver {
    async fn stop_all(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    async fn apply_config(&mut self, sub: UniverseUpdateSubscriber, config: (EndpointSpeed, u16)) {
        self.handle.get_or_insert_with(|| {
            let (tx, rx) =
                tokio::sync::mpsc::channel::<(UniverseUpdateSubscriber, (EndpointSpeed, u16))>(10);
            self.tx = tx;
            tokio::spawn(sacn_runner(rx, self.shutdown_notify.clone()))
        });
        self.tx.send((sub, config)).await.expect("Why not!");
    }
}

async fn sacn_runner(
    mut rx: Receiver<(UniverseUpdateSubscriber, (EndpointSpeed, u16))>,
    shutdown: Arc<Notify>,
) {
    let mut subs = vec![];
    let mut mapping: HashMap<u16, UniverseId> = HashMap::new();
    let mut update_intervals: HashMap<EndpointSpeed, Vec<u16>> = HashMap::new();
    let mut update_timers: HashMap<EndpointSpeed, Interval> = HashMap::new();
    let mut cache: HashMap<UniverseId, [u8; UNIVERSE_SIZE + 1]> = HashMap::new();

    let mut dmx_source = sacn::source::SacnSource::new_v4("MLC Controller").unwrap();
    dmx_source.set_is_sending_discovery(true);

    'o: loop {
        select! {
            sub = rx.recv() => {
                let (sub, (speed, id)) = sub.unwrap();
                update_intervals.entry(speed).or_default().push(id);
                *mapping.entry(id).or_insert(0) = sub.universe();
                cache.entry(sub.universe()).or_insert_with(|| [0; UNIVERSE_SIZE + 1]);
                update_timers.entry(speed).or_insert_with(|| tokio::time::interval(speed.duration()));
                dmx_source.register_universe(id).unwrap();

                subs.push(sub);
            }
            _ = shutdown.notified() => {
                break 'o;
            }
            Some(speed) = await_times(&mut update_timers) => {
                let to_update = &update_intervals[&speed];
                for id in to_update {
                    let universe = mapping[id];
                    let data = &cache[&universe];
                    dmx_source.send(&[*id], data, None, None, None).unwrap();
                }
            }
            Some(m) = await_subs(&mut subs) => {
                match m {
                    Ok(u) => {
                        match u {
                            UniverseUpdate::Single{ update } => {
                                if let Some(universe) = cache.get_mut(&update.0.universe()) {
                                    universe[update.0.address().take()] = update.1;
                                }
                            }
                            UniverseUpdate::Many{ updates } => {
                                for update in updates {
                                    if let Some(universe) = cache.get_mut(&update.0.universe()) {
                                        universe[update.0.address().take()] = update.1;
                                    }
                                }
                            }
                            UniverseUpdate::Entire{ universe, values } => {
                                if let Some(universe) = cache.get_mut(&universe) {
                                    universe[0..].copy_from_slice(&*values);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error getting universe update: {}", e);
                    }
                }
            }
        }
    }
}

async fn await_times(subs: &mut HashMap<EndpointSpeed, Interval>) -> Option<EndpointSpeed> {
    // info!("Waiting for universe updates");

    let mut f = FuturesUnordered::new();
    for (speed, interval) in subs {
        f.push(async move {
            let _ = interval.tick().await;
            speed.clone()
        });
    }
    f.next().await
}
