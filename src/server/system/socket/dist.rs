use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, RwLock},
};

use crate::{
    conn::line,
    pkt::{
        meta::{self, PlayerId},
        xt::XTPacket,
    },
    server::system::socket::authgate::{self, AuthResult},
};

pub enum Event {
    Packet(XTPacket),
    Connected,
    Disconnected,
}

/* Unless we have a good reason to change,
 * we only store connections that are authenticated!
 * TODO: We likely should also implement some timeout / protection system
 * otherwise a malicious actor could overload the scheduler!
 * ... and we would have no idea who it was!!
 */
pub struct Distributed {
    connections: Arc<RwLock<HashMap<meta::PlayerId, line::LineConnWriter>>>,
    rx: mpsc::Receiver<(meta::PlayerId, Event)>,
}

impl Distributed {
    // todo: split into sub functions
    pub async fn new(socket: TcpListener) -> Self {
        let connections: Arc<RwLock<HashMap<meta::PlayerId, line::LineConnWriter>>> =
            Arc::new(RwLock::new(HashMap::with_capacity(64)));

        let (tx, rx) = mpsc::channel::<(meta::PlayerId, Event)>(32);

        tokio::spawn({
            let connections = connections.clone();
            async move {
                loop {
                    let (addr, (writer, reader)) = match socket.accept().await {
                        Err(e) => todo!("handle failure to accept tcp connections {e}"),
                        Ok((stream, addr)) => (addr, line::line_con(stream).await),
                    };
                    let (player_id, writer, mut reader) = match authgate::gate(writer, reader).await
                    {
                        Ok((AuthResult::Unauthenticated, _, _)) => {
                            log::warn!("Bad auth result for {addr}, discarding");
                            continue;
                        }
                        Ok((authgate::AuthResult::Authenticated(player_id), writer, reader)) => {
                            (player_id, writer, reader)
                        }
                        Err(e) => {
                            log::error!("FUCK: {:#}", e);
                            todo!("")

                        }//todo!("handle auth failure: {e}"),
                    };
                    let mut connections = connections.write().await;
                    if let Some(_) = connections.insert(player_id, writer) {
                        todo!("player already connected to server! HANDLE!");
                    }
                    log::info!("player {player_id} connected with address {addr}");

                    tokio::spawn({
                        let tx = tx.clone();
                        async move {
                            if let Err(_) = tx.send((player_id, Event::Connected)).await {
                                return;
                            };
                            loop {
                                match reader.read::<XTPacket>().await {
                                    Ok(None) => {
                                        let _ = tx.send((player_id, Event::Disconnected)).await;
                                        break;
                                    }
                                    Ok(Some(xt)) => {
                                        if let Err(_) =
                                            tx.send((player_id, Event::Packet(xt))).await
                                        {
                                            break;
                                        }
                                    }
                                    Err(_) => todo!("failure in read xt"),
                                };
                            }
                        }
                    });
                }
            }
        });

        Self {
            rx,
            connections: connections.clone(),
        }
    }
    // TODO: when the struct is dropped, is it guranteed that the socket is closed?
    pub async fn poll(&mut self) -> (meta::PlayerId, Event) {
        match self.rx.recv().await{
            Some(t) => t,
            None => panic!("socket was closed for some reason"),
        }
    }

    pub async fn push(&mut self, player_id: meta::PlayerId, xt: XTPacket) -> Result<()> {
        let mut connections = self.connections.write().await;
        match connections.get_mut(&player_id) {
            Some(writer) => match writer.write(xt).await {
                Ok(()) => Ok(()),
                Err(e) => Err(e).context("failed to send to player {player_id}"),
            },
            None => anyhow::bail!("illegal player id"),
        }
    }
}
