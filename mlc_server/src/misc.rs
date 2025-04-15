use std::sync::Arc;

use bitflags::bitflags;
use mlc_data::misc::ErrIgnore;
use tokio::sync::watch::{self};

#[derive(Debug, Clone)]
pub struct AdaptNotifier {
    notifier: Arc<watch::Sender<AdaptScopes>>,
    _waiter: Arc<watch::Receiver<AdaptScopes>>,
}

impl AdaptNotifier {
    pub fn create() -> Self {
        let (tx, rx) = watch::channel(AdaptScopes::NONE);

        Self {
            notifier: Arc::new(tx),
            _waiter: Arc::new(rx),
        }
    }

    pub fn notify(&self, scopes: AdaptScopes) {
        self.notifier.send(scopes).debug_ignore();
    }

    pub fn wait(&self, scopes: AdaptScopes) -> impl Future<Output = AdaptScopes> {
        let mut rx = self.notifier.subscribe();

        async move {
            let mut scs = AdaptScopes::empty();
            loop {
                if let Ok(()) = rx.changed().await {
                    let sc = *rx.borrow_and_update();
                    if !(sc & scopes).is_empty() {
                        scs = sc;
                        break;
                    } else {
                        log::info!("Adapt recvieved but didn't matched listened scopes")
                    }
                }
            }

            scs
        }
    }
}

bitflags! {

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AdaptScopes: u16 {
        const NONE =      0b00000000;
        const UNIVERSES = 0b00000001;
        const ENDPOINTS = 0b00000010;
        const SETTINGS =  0b00000100;
    }
}

#[derive(Debug, Clone)]
pub struct ShutdownHandler {
    notifier: Arc<watch::Sender<ShutdownPhase>>,
    _waiter: Arc<watch::Receiver<ShutdownPhase>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShutdownPhase {
    /// Currently not shutting down
    None = 0,
    /// Everything non-critical shutting down
    Phase1 = 1,
    /// Everything critical shutting down mainly the server
    Phase2 = 2,
    /// Shutdown should be complete
    Done = 3,
}

impl ShutdownHandler {
    pub fn create() -> Self {
        let (tx, rx) = watch::channel(ShutdownPhase::None);

        Self {
            notifier: Arc::new(tx),
            _waiter: Arc::new(rx),
        }
    }

    pub fn advance(&self) -> impl Future<Output = ()> + 'static {
        let next = match &*self.notifier.borrow() {
            ShutdownPhase::None => ShutdownPhase::Phase1,
            ShutdownPhase::Phase1 => ShutdownPhase::Phase2,
            ShutdownPhase::Phase2 => ShutdownPhase::Done,
            ShutdownPhase::Done => ShutdownPhase::Done,
        };
        self.notifier.send(next).debug_ignore();
        // async {
        tokio::task::yield_now()
        // }
    }

    pub fn wait(&self, phase: ShutdownPhase) -> impl Future<Output = ()> {
        let mut rx = self.notifier.subscribe();

        async move {
            loop {
                if let Ok(()) = rx.changed().await {
                    let sp = *rx.borrow_and_update();
                    if phase <= sp {
                        break;
                    }
                }
            }
        }
    }
}
