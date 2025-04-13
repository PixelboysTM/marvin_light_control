use std::sync::Arc;

use bitflags::bitflags;
use mlc_data::misc::ErrIgnore;
use tokio::sync::watch::{self, Sender};

#[derive(Debug, Clone)]
pub struct AdaptNotifer {
    notifier: Arc<tokio::sync::watch::Sender<AdaptScopes>>,
    waiter: Arc<tokio::sync::watch::Receiver<AdaptScopes>>,
}

impl AdaptNotifer {
    pub fn create() -> Self {
        let (tx, rx) = watch::channel(AdaptScopes::NONE);

        Self {
            notifier: Arc::new(tx),
            waiter: Arc::new(rx),
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
    }
}
