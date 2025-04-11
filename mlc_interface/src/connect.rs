use dioxus::hooks::use_signal;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use dioxus::signals::{Readable, Signal};
use mlc_communication::remoc::ConnectExt;
use mlc_communication::{remoc, ServiceIdentifiable};
use std::net::Ipv4Addr;
use std::ops::Deref;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::utils::UniqueEq;
use crate::CONNECT_URL;

pub async fn connect_url<I: ServiceIdentifiable>(
    addr: (Ipv4Addr, u16),
) -> Result<I::Client, Box<dyn std::error::Error>> {
    let socket = TcpStream::connect(addr).await?;
    let (socket_rx, mut socket_tx) = socket.into_split();
    socket_tx.write_all(&I::IDENT).await?;
    let client: I::Client = remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
        .consume()
        .await?;

    Ok(client)
}

pub async fn connect<I: ServiceIdentifiable>() -> Result<I::Client, Box<dyn std::error::Error>> {
    connect_url::<I>(*CONNECT_URL.read()).await
}

pub enum UseServiceState<T> {
    Pending,
    Ready(T),
    Errored(Box<dyn std::error::Error>),
}

pub type SClient<S> = Memo<UniqueEq<<S as ServiceIdentifiable>::Client>>;

pub fn use_service_url<I: ServiceIdentifiable>(
    addr: (Ipv4Addr, u16),
) -> Result<Memo<UniqueEq<I::Client>>, RenderError> {
    let mut state: Signal<UseServiceState<UniqueEq<I::Client>>> =
        use_signal(|| UseServiceState::Pending);
    let fut = use_future(move || async move {
        let r = connect_url::<I>(addr).await;

        match r {
            Ok(c) => *state.write() = UseServiceState::Ready(c.into()),
            Err(e) => *state.write() = UseServiceState::Errored(e),
        }
    });

    match state.read().deref() {
        UseServiceState::Ready(_) => {}
        UseServiceState::Pending => {
            return Err(RenderError::Suspended(SuspendedFuture::new(fut.task())));
        }
        UseServiceState::Errored(_error) => return Err(RenderError::default()), // TODO: Pack da halt den error rein
    }

    Ok(use_memo(move || {
        let s = state.read();

        match s.deref() {
            UseServiceState::Pending => unreachable!("Muste be Ready"),
            UseServiceState::Ready(c) => c.clone(),
            UseServiceState::Errored(_) => unreachable!("Muste be Ready"),
        }
    }))
}

pub fn use_service<I: ServiceIdentifiable>() -> Result<Memo<UniqueEq<I::Client>>, RenderError> {
    use_service_url::<I>(*CONNECT_URL.read())
}

pub trait RtcSuspend<T> {
    fn rtc_suspend(&self) -> Result<MappedSignal<T>, RenderError>;
}

impl<T: Clone, E: std::fmt::Debug + Clone> RtcSuspend<T> for Resource<Result<T, E>> {
    fn rtc_suspend(&self) -> Result<MappedSignal<T>, RenderError> {
        match self.suspend() {
            Ok(v) => match &*v.read() {
                Ok(_) => Ok(v.clone().map(|r| r.as_ref().expect("Must be"))),
                Err(e) => {
                    error!("Making fake msg: {e:?}");
                    Err(RenderError::default())
                } // TODO: Make real error msg
            },
            Err(s) => Err(s),
        }
    }
}
