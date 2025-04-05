use dioxus::signals::Readable;
use mlc_communication::remoc::ConnectExt;
use mlc_communication::{remoc, ServiceIdentifiable};
use std::net::Ipv4Addr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

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
