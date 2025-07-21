#[cfg(test)]
mod xt_parse_tests {
    use crate::pkt::meta;
    use crate::server;

    use assert_matches::assert_matches;

    fn server() -> server::Server {
        server::Server::new()
    }

    #[tokio::test]
    async fn heartbeat() {
        let mut server = server();
        let mut out = Vec::new();
        let packet_in: server::PacketIn =
            server::PacketIn::Player(69, meta::client::Packet::Heartbeat);
        server.transition(packet_in, &mut out).await.unwrap();

        assert_matches!(
            out.as_slice(),
            [server::PacketOut {
                player_id: 69,
                packet: meta::server::Packet::Heartbeat,
            }]
        );
    }
}
