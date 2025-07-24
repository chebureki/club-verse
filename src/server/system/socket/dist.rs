use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, RwLock},
};
use tokio_util::sync::CancellationToken;

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

    // notify actual sockets to close their connection
    cancel: CancellationToken,
}

impl Distributed {
    // todo: split into sub functions
    pub async fn new(socket: TcpListener) -> Self {
        let connections: Arc<RwLock<HashMap<meta::PlayerId, line::LineConnWriter>>> =
            Arc::new(RwLock::new(HashMap::with_capacity(64)));

        let (tx, rx) = mpsc::channel::<(meta::PlayerId, Event)>(32);
        let cancel = CancellationToken::new();
        tokio::spawn({
            let connections = connections.clone();
            let cancel = cancel.clone();
            async move {
                loop {
                    let (addr, (writer, reader)) = tokio::select! {
                        _ = cancel.cancelled() => break,
                        conn_res =  socket.accept() => match conn_res{
                            Err(e) => todo!("handle failure to accept tcp connections {e}"),
                            Ok((stream, addr)) => (addr, line::line_con(stream).await),
                        }
                    };
                    log::debug!("accepted connection from {addr}");

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
                        } //todo!("handle auth failure: {e}"),
                    };
                    let mut conn_map = connections.write().await;
                    if let Some(_) = conn_map.insert(player_id, writer) {
                        todo!("player already connected to server! HANDLE!");
                    }
                    log::info!("player {player_id} connected with address {addr}");

                    tokio::spawn({
                        let connections = connections.clone();
                        let tx = tx.clone();
                        let cancel = cancel.clone();
                        async move {
                            if let Err(_) = tx.send((player_id, Event::Connected)).await {
                                return;
                            };
                            loop {
                                let xt_res = tokio::select! {
                                    _ = cancel.cancelled() => {
                                        break;
                                    }
                                    res = reader.read::<XTPacket>() => res
                                };

                                match xt_res {
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
                            let _ = connections.write().await.remove(&player_id);


                            log::info!("connection for player {player_id} {addr} dropped");
                        }
                    });
                }
            }
        });

        Self {
            rx,
            connections: connections.clone(),
            cancel,
        }
    }
    // TODO: when the struct is dropped, is it guranteed that the socket is closed?
    pub async fn poll(&mut self) -> (meta::PlayerId, Event) {
        match self.rx.recv().await {
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

impl Drop for Distributed {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
