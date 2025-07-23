pub mod state;
mod system;

use std::net::{SocketAddr, ToSocketAddrs};

use tokio::{
    sync::{broadcast, mpsc},
    task::AbortHandle,
};

use crate::{pkt::meta, server::system::System};
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PlayerConnected(meta::PlayerId),
    PlayerDisconnected(meta::PlayerId),
    PlayerPacketIn(meta::PlayerId, meta::client::Packet),
    PlayerPacketOut(meta::PlayerId, meta::server::Packet),
    Error,

    DisconnectPlayer,

    Heartbeat,
}

pub enum Error {
    PlayerError(meta::PlayerId, anyhow::Error),
}

pub async fn from_systems(systems: Vec<Box<dyn System>>) -> Result<()> {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    for sys in &systems {
        sys.instantiate(event_tx.clone())
            .await
            .context("failed to instantiate server system")?;
    }
    Ok(())
}

pub async fn bind<A>(address: A) -> Result<()>
where
    A: ToSocketAddrs,
{
    let address = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No address found"))?;
    let systems: Vec<Box<dyn system::System>> = vec![
        Box::new(system::heartbeat::Heartbeat),
        Box::new(system::socket::as2::Socket { address }),
    ];

    from_systems(systems).await?;
    Ok(())
}
