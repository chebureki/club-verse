pub mod state;
mod system;

use std::net::{SocketAddr, ToSocketAddrs};

use tokio::{
    sync::{broadcast, mpsc},
    task::AbortHandle,
};

use crate::{
    pkt::meta,
    server::system::{EventReceiver, EventSender, System},
};
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum ServerCmd {
    Foo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PlayerConnected(meta::PlayerId),
    PlayerDisconnected(meta::PlayerId),
    PacketSent(meta::PlayerId, meta::server::Packet),
    PacketReceived(meta::PlayerId, meta::client::Packet),
    // TODO: this is a COMMAND not an EVENT
    PlayerTransferRoomRequest(meta::PlayerId, meta::RoomId),
    PlayerJoinedRoom(meta::PlayerId, meta::RoomId),
    Error,
    DisconnectPlayer,
    Heartbeat,
}

pub enum Error {
    PlayerError(meta::PlayerId, anyhow::Error),
}

pub async fn from_systems(systems: Vec<Box<dyn System>>) -> Result<mpsc::Sender<ServerCmd>> {
    /* NOTE:
     * bus_tx is the sole fully owned sender!
     * When dropped all underlying systems are dropped aswell
     */
    let (bus_tx, _) = broadcast::channel::<Event>(16);
    let event_tx = EventSender(bus_tx.downgrade());

    let server_state = state::ServerState::new();

    for sys in &systems {
        sys.instantiate(
            server_state.clone(),
            event_tx.clone(),
            EventReceiver(bus_tx.subscribe()),
        )
        .await?;
    }

    let (cmd_tx, mut cmd_rx) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                ServerCmd::Foo => {
                    log::warn!("bar");
                }
            }
        }
        drop(bus_tx)
    });
    Ok(cmd_tx)
}

pub async fn bind<A>(address: A) -> Result<mpsc::Sender<ServerCmd>>
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
        Box::new(system::server::Server),
    ];

    let tx = from_systems(systems).await?;
    Ok(tx)
}
