#[cfg(test)]
pub mod tests {

    use crate::{
        datamodel,
        pkt::meta,
        server::{
            self, state,
            system::{self, EventReceiver, EventSender},
            Event,
        },
    };
    use anyhow::Result;
    use assert_matches::assert_matches;
    use async_trait::async_trait;
    use std::time::Duration;
    use tokio::sync::{mpsc, oneshot};

    /// Accumulate events and send relevant events
    pub struct TestSystem {
        resp_to: oneshot::Sender<Vec<Event>>,
        inputs: Vec<Event>,
    }

    #[async_trait]
    impl server::system::System for TestSystem {
        async fn instantiate(
            self: Box<Self>,
            _server: state::ServerState,
            mut event_tx: EventSender,
            mut event_rx: EventReceiver,
        ) -> Result<()> {
            let TestSystem { resp_to, inputs } = *self;
            tokio::spawn(async move {
                let mut acc = Vec::new();
                for e in inputs {
                    event_tx.push(e).await;
                }
                while let Some(event) = event_rx.poll().await {
                    acc.push(event);
                }
                let _ = resp_to.send(acc);
            });
            Ok(())
        }
    }

    async fn test_server(inputs: Vec<Event>) -> Vec<Event> {
        let (tx, rx) = oneshot::channel::<Vec<Event>>();
        let test_system = TestSystem {
            resp_to: tx,
            inputs,
        };

        let server_tx = server::from_systems(vec![
            Box::new(test_system),
            Box::new(system::server::Server),
        ])
        .await;

        // TODO: WHACK this shit sucks
        tokio::time::sleep(Duration::from_secs(2)).await;
        drop(server_tx);

        match rx.await {
            Ok(events) => events,
            Err(_) => panic!("error in test server"),
        }
    }

    #[tokio::test]
    async fn simple_join() {
        let input: Vec<Event> = vec![
            Event::PlayerConnected(102),
            Event::PacketReceived(
                102,
                meta::client::Packet::JoinServer {
                    penguin_id: 102,
                    login_key: "asdasdfas".to_owned(),
                    language: "en".to_owned(),
                },
            ),
        ];
        let actual_output: Vec<Event> = test_server(input).await;

        assert_matches!(
            actual_output.as_slice(),
            [
                Event::PlayerConnected(102),
                Event::PacketReceived(
                    102,
                    meta::client::Packet::JoinServer {
                        penguin_id: 102,
                        ..
                    }
                ),
                Event::PacketSent(102, meta::server::Packet::Loaded),
                Event::PacketSent(102, meta::server::Packet::ActiveFeatures {}),
                Event::PacketSent(102, meta::server::Packet::LoadPlayer { .. }),
                Event::PacketSent(102, meta::server::Packet::JoinedServer { .. }),
                Event::PacketSent(
                    102,
                    meta::server::Packet::GetPlayerStamps { player_id: 102 }
                ),
                Event::PlayerTransferRoomRequest(102, 230, _),
                Event::PlayerJoinedRoom(102, 230),
                Event::PacketSent(102, meta::server::Packet::JoinRoom { room_id: 230, .. }),
                Event::PacketSent(
                    102,
                    meta::server::Packet::AddedPlayer {
                        player: datamodel::PlayerGist { id: 102, .. }
                    }
                )
            ]
        )
    }
}
