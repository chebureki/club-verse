mod authgate;
mod dist;

pub mod as2 {
    use std::net::SocketAddr;

    use crate::{
        pkt,
        server::system::{socket::dist, EventReceiver, EventSender},
    };

    use anyhow::{Context, Result};
    use async_trait::async_trait;
    use tokio::{net::TcpListener, sync::broadcast};

    use crate::server::{system::System, Event};

    pub struct Socket {
        pub address: SocketAddr,
    }

    // TODO: this should be generic, such it also works for as3
    #[async_trait]
    impl System for Socket {
        async fn instantiate(
            &self,
            mut event_tx: EventSender,
            mut event_rx: EventReceiver,
        ) -> Result<()> {
            let socket = TcpListener::bind(&self.address)
                .await
                .context("failed to bind for socket")?;

            log::info!("server listening on {}", &self.address);
            let mut dist = dist::Distributed::new(socket).await;

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        event = event_rx.poll() => match event{
                            None => break,
                            Some(Event::PlayerPacketOut(player_id, meta)) => {
                                let xt: pkt::xt::XTPacket = (pkt::xt::as2::server::Packet(meta)).into();
                                dist.push(player_id, xt).await.unwrap();

                            }
                            _ => {}
                        },
                        (player_id, event) = dist.poll() => match event{
                          dist::Event::Connected => {
                              event_tx.push(Event::PlayerConnected(player_id)).await;
                          },
                          dist::Event::Disconnected => todo!("player disconnected"),
                          dist::Event::Packet(xt) => {
                              let as2: pkt::xt::as2::client::Packet = match xt.try_into(){
                                  Err(e) => todo!("bad as2 from client: {e}"),
                                  Ok(as2) => as2,
                              };
                              event_tx.push(Event::PlayerPacketIn(player_id, as2.0)).await;
                          },
                        }
                    }
                }
            });

            Ok(())
        }
    }
}
