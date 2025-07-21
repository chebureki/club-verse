//mod conn;
mod logic;
pub mod data;

use std::collections::HashMap;

use anyhow::{bail, Error, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

use crate::pkt::meta::{self, PlayerId};
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum PacketIn {
    Player(PlayerId, meta::client::Packet),
    // Self/ Other/ Meta?
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemPacket{
    PlayerJoined(player),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketOut {
    player_id: meta::PlayerId,
    packet: meta::server::Packet,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PacketCast {
    Player(meta::PlayerId),
    Room(meta::RoomId),
    //Server,
}

pub struct Server {
    players: HashMap<meta::PlayerId, state::Player>,
}

pub enum Cmd {
    NewConnection {
        tx: TcpStream,
        rx: TcpListener,
    },
    Healthcheck {
        resp_to: oneshot::Sender<Result<()>>,
    },
}

impl Server {
    pub fn new() -> Self {
        Self {
            players: HashMap::with_capacity(128),
        }
    }

    fn cast(
        &self,
        packet: meta::server::Packet,
        cast: PacketCast,
        packet_out: &mut Vec<PacketOut>,
    ) -> Result<(), Error> {
        // TODO: we need a room membership lookup, otherwise we send shit in linear time
        match cast {
            PacketCast::Player(player_id) => packet_out.push(PacketOut { player_id, packet }),
            _ => todo!(),
        }
        Ok(())
    }

}

pub async fn cmd_loop(server: Server) -> mpsc::Sender<Cmd> {
    let (tx, mut rx) = mpsc::channel(256);

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Cmd::NewConnection { tx, rx } => todo!(),
                Cmd::Healthcheck { resp_to } => todo!(),
            }

        }
    });

    tx
}
