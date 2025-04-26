use std::sync::{Arc, RwLock};

use bitflags::bitflags;
use mlc_data::misc::ErrIgnore;
use tokio::sync::watch::{self};
use tracing::info;

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
            let scs;
            loop {
                if let Ok(()) = rx.changed().await {
                    let sc = *rx.borrow_and_update();
                    if !(sc & scopes).is_empty() {
                        scs = sc;
                        break;
                    } else {
                        log::info!("Adapt received but didn't matched listened scopes")
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
    waiters: Arc<RwLock<[u16; 4]>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShutdownPhase {
    /// Currently not shutting down
    None = 0,
    /// Everything non-critical shutting down
    Phase1 = 1,
    /// Everything critical shutting down, mainly the server
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
            waiters: Arc::new(RwLock::new([0; 4])),
        }
    }
    
    pub fn shutdown(&self) {
        if self.current() == ShutdownPhase::None { 
            tokio::task::spawn(self.advance());
        }
        info!("Already shutting down");
    }

    fn advance(&self) -> impl Future<Output = ()> + 'static {
        let next = match self.current() {
            ShutdownPhase::None => ShutdownPhase::Phase1,
            ShutdownPhase::Phase1 => ShutdownPhase::Phase2,
            ShutdownPhase::Phase2 => ShutdownPhase::Done,
            ShutdownPhase::Done => ShutdownPhase::Done,
        };
        self.notifier.send(next).debug_ignore();

        tokio::task::yield_now()
    }

    pub fn try_advance(&self) -> impl Future<Output = bool> + 'static {
        let can = {
            let w = self.waiters.read().expect("Waiter lock");
            let c = self.current();
            let v = w[c as usize];
            info!("Waiters: {:?} in Phase {:?}", v, c);
            v == 0
        };

        let fut = self.advance();

        async move {
            if can {
                fut.await
            }
            can
        }
    }

    pub fn current(&self) -> ShutdownPhase {
        *self.notifier.borrow()
    }

    pub fn wait(&self, phase: ShutdownPhase) -> impl Future<Output = ()> {
        let mut rx = self.notifier.subscribe();

        async move {
            let _d = ShutdownWaiterTracker::new(phase, self.waiters.clone());
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

struct ShutdownWaiterTracker {
    phase: ShutdownPhase,
    waiters: Arc<RwLock<[u16; 4]>>,
}

impl ShutdownWaiterTracker {
    fn new(phase: ShutdownPhase, waiters: Arc<RwLock<[u16; 4]>>) -> Self {
        {
            let mut w = waiters.write().expect("Waiter lock");
            w[phase as usize] += 1;
        }
        Self { phase, waiters }
    }
}

impl Drop for ShutdownWaiterTracker {
    fn drop(&mut self) {
        let mut w = self.waiters.write().expect("Waiter lock");
        w[self.phase as usize] -= 1;
    }
}
