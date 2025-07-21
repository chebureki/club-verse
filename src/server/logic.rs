use anyhow::Error;

use crate::server::{PacketIn, PacketOut, Server};

pub enum ServerError {
    /// Bad Inputs
    ClientFault(Error),

    /// DB down, FS exhausted ...
    InfraFault(Error),

    /// Server is in a bad state or stupid!
    ServerFault(Error),
}

pub trait ServerAutomata {
    fn handle_pkt(
        &mut self,
        packet_in: PacketIn,
        packet_out: &mut Vec<PacketOut>,
    ) -> Result<(), ServerError>;
}

impl ServerAutomata for Server{
    fn handle_pkt(
            &mut self,
            packet_in: PacketIn,
            packet_out: &mut Vec<PacketOut>,
        ) -> Result<(), ServerError> {
        todo!()
    }
}





// impl ServerAutomata for Server{
//     fn handle(
//             &mut self,
//             packet_in: PacketIn,
//             packet_out: &mut Vec<PacketOut>,
//         ) -> Result<(), ServerError> {
//         todo!()
//     }
//
// }

// impl ServerAutomata for Server {
//     // fn transition(&mut self, packet_in: PacketIn, packet_out: &mut Vec<PacketOut>) {
//     // }
// }

// pub(crate) async fn transition(
//     &mut self,
//     packet_in: PacketIn,
//     packet_out: &mut Vec<PacketOut>,
// ) -> Result<(), Error> {
//     match packet_in {
//         PacketIn::Player(player_id, packet) => match packet {
//             meta::client::Packet::Heartbeat => self.cast(
//                 meta::server::Packet::Heartbeat,
//                 PacketCast::Player(player_id),
//                 packet_out,
//             ),
//             meta::client::Packet::PlayerSetPosition { x, y } => todo!(),
//         },
//     }
// }
// pub(crate) async fn transition(
//     &mut self,
//     packet_in: PacketIn,
//     packet_out: &mut Vec<PacketOut>,
// ) -> Result<(), Error> {
//     match packet_in {
//         PacketIn::Player(player_id, packet) => match packet {
//             meta::client::Packet::Heartbeat => self.cast(
//                 meta::server::Packet::Heartbeat,
//                 PacketCast::Player(player_id),
//                 packet_out,
//             ),
//             meta::client::Packet::PlayerSetPosition { x, y } => todo!(),
//         },
//     }
// }
