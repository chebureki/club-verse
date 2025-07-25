use anyhow::{Context, Result};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use crate::{
    conn::line,
    pkt::{self, meta, xt::as2},
};

pub struct ServerList {}

impl ServerList {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn bind(self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:6969")
            .await
            .context("failed to bind to addr")?;

        log::info!("server list login binded to: {}", listener.local_addr()?);
        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(t) => t,
                Err(e) => {
                    log::error!(
                        "failed to accept socket connection, connection will be discarded!: {e:?}"
                    );
                    continue;
                }
            };
            tokio::spawn(login_loop(stream, addr));
        }
    }
}

async fn login_loop(stream: TcpStream, addr: SocketAddr) {
    log::info!("connection from: {addr}");
    let (mut tx, mut rx) = line::line_con(stream).await;

    let (username, password) = loop {
        match rx.read::<pkt::xml::client::Packet>().await {
            // TODO: BAD: user error and server error are not differentiated
            Err(line::ReadError::EnvError(e)) => {
                log::error!("failed to read xml: {}", e);
                continue;
            }

            Err(line::ReadError::ParseError(e)) => {
                log::error!("failed to parse xml: {}", e);
                continue;
            }
            Ok(None) => {
                log::info!("{addr} left login loop early");
                return;
            }
            Ok(Some(pkt::xml::client::Packet::VersionCheck { expected })) => {
                log::info!("client expects version {expected}");
                tx.write(pkt::xml::server::Packet::ApiOK).await.unwrap();
            }

            Ok(Some(pkt::xml::client::Packet::RandomKey)) => {
                tx.write(pkt::xml::server::Packet::RandomKey("houdini".to_owned()))
                    .await
                    .unwrap();
            }

            Ok(Some(pkt::xml::client::Packet::Login { username, password })) => {
                log::info!("attempt to login into: {username} from {addr}");
                break (username, password);
            }
        }
    };

    if &username != "kirill"  {
        tx.write(as2::server::Packet(meta::server::Packet::Error(meta::server::Error::NameNotFound))).await.unwrap();
        return;
    }

    let list = pkt::xt::XTPacket {
        handler_id: None,
        packet_id: "l".to_owned(),
        internal_id: -1,
        data: vec![
            "102".to_owned(),
            "fc15ebff4bae96e53d1a55ba559eca3f".to_owned(),
            "".to_owned(),
            "3100,0|3101,0|3102,0|3103,0".to_owned(),
        ],
    };

    tx.write(list).await.unwrap();

    // %xt%l%-1%102%fc15ebff4bae96e53d1a55ba559eca3f%%3100,0|3101,0|3102,0|3103,0% 
}
