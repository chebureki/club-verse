pub mod client {
    use crate::pkt::{self, meta::client::*};
    use std::num::ParseIntError;

    use thiserror::Error;

    use crate::pkt::xt::XTPacket;

    #[derive(Debug, Clone, PartialEq, Error)]
    pub enum PacketError {
        /// Fewer or more arguments than expected
        #[error("Bad argument count, received fewer or more arguments than expected")]
        BadArgCount,

        /// Failed to parse an integer argument
        #[error("Bad data type: failed to parse integer - {0}")]
        BadDatatypeInt(#[from] ParseIntError),

        /// Entirely not recognized
        #[error("Unrecognized packet: handler_id='{handler_id}', packet_id='{packet_id}'")]
        Unrecognized {
            handler_id: String,
            packet_id: String,
        },
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Packet(pub pkt::meta::client::Packet);


    impl TryFrom<XTPacket> for Packet {
        type Error = PacketError;
        fn try_from(value: XTPacket) -> Result<Self, Self::Error> {
            let XTPacket {
                handler_id,
                packet_id,
                internal_id: _,
                data,
            } = value;

            // TODO: is it guranteed that this is a logic error at this point?
            let handler_id = match handler_id {
                Some(hi) => hi,
                None => panic!("attempt to parse a server packet as a client packet"),
            };

            match (handler_id.as_str(), packet_id.as_str(), data.as_slice()) {
                // ("s", "u#sp", [x, y]) => Ok(Packet::PlayerSetPosition {
                //     x: x.parse()?,
                //     y: y.parse()?,
                // }),
                // ("s", "u#sp", _) => Err(PacketError::BadArgCount),
                //
                // ("s", "u#h", []) => Ok(Packet::Heartbeat),
                // ("s", "u#h", _) => Err(PacketError::BadArgCount),

                _ => Err(PacketError::Unrecognized {
                    handler_id: handler_id.to_owned(),
                    packet_id: packet_id.to_owned(),
                }),
            }
        }
    }


    // pub struct Packet(pub meta::client::Packet);


    // pub fn deserialize(packet: XTPacket) -> Result<Packet>;


    // impl TryFrom<&XTPacket> for Packet {
    //     type Error = PacketError;
    //     fn try_from(value: &XTPacket) -> Result<Self, Self::Error> {
    //         let XTPacket {
    //             handler_id,
    //             packet_id,
    //             internal_id: _,
    //             data,
    //         } = value;
    //
    //         // TODO: is it guranteed that this is a logic error at this point?
    //         let handler_id = match handler_id {
    //             Some(hi) => hi,
    //             None => panic!("attempt to parse a server packet as a client packet"),
    //         };
    //
    //         match (handler_id.as_str(), packet_id.as_str(), data.as_slice()) {
    //             ("s", "u#sp", [x, y]) => Ok(Packet::PlayerSetPosition {
    //                 x: x.parse()?,
    //                 y: y.parse()?,
    //             }),
    //             ("s", "u#sp", _) => Err(PacketError::BadArgCount),
    //
    //             ("s", "u#h", []) => Ok(Packet::Heartbeat),
    //             ("s", "u#h", _) => Err(PacketError::BadArgCount),
    //
    //             _ => Err(PacketError::Unrecognized {
    //                 handler_id: handler_id.to_owned(),
    //                 packet_id: packet_id.to_owned(),
    //             }),
    //         }
    //     }
    // }
}

pub mod server {
    use crate::pkt::{
        self,
        xt::{XTPacket, XT_DEFAULT_INT_ID},
    };

    // // TODO: move somewhere else
    #[derive(Clone, Debug, PartialEq)]
    pub struct Packet(pub pkt::meta::server::Packet);

    impl Into<String> for Packet {
        fn into(self) -> String {
            let xt: XTPacket = self.into();
            xt.into()
        }
    }

    impl Into<XTPacket> for Packet {
        fn into(self) -> XTPacket {
            match self.0 {
                pkt::meta::server::Packet::PlayerSetPosition { player_id, x, y } => todo!(),
                pkt::meta::server::Packet::Heartbeat => todo!(),
                pkt::meta::server::Packet::Error(error) => {
                    let error: u32 = error.clone() as u32;
                    XTPacket {
                        handler_id: None,
                        packet_id: "e".to_owned(),
                        internal_id: XT_DEFAULT_INT_ID,
                        data: vec![error.to_string()],
                    }
                }
                pkt::meta::server::Packet::LoginResponse {} => XTPacket {
                    handler_id: None,
                    packet_id: "l".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    // TODO: why does houdini return empty?
                    data: vec!["".to_owned()],
                },
            }
        }
    }
}

// #[cfg(test)]
// mod as2_packet_tests {
//     use crate::pkt::xt::XTPacket;
//     use assert_matches::assert_matches;
//
//     use super::*;
//     use crate::pkt::meta;
//
//     #[test]
//     fn basic_client_parse() {
//         let xt = XTPacket {
//             handler_id: Some("s".to_owned()),
//             packet_id: "u#sp".to_owned(),
//             internal_id: 1,
//             data: vec!["395".to_owned(), "384".to_owned()],
//         };
//
//         let res: Result<meta::client::Packet, client::PacketError> = xt.try_into();
//         assert_matches!(
//             res,
//             Ok(meta::client::Packet::PlayerSetPosition { x: 395, y: 384 })
//         )
//     }
//
//     #[test]
//     fn basic_client_parse_error_handling_count() {
//         let xt = XTPacket {
//             handler_id: Some("s".to_owned()),
//             packet_id: "u#sp".to_owned(),
//             internal_id: 1,
//             data: vec!["395".to_owned()],
//         };
//
//         let res: Result<meta::client::Packet, client::PacketError> = (&xt).try_into();
//         assert_matches!(res, Err(client::PacketError::BadArgCount))
//     }
//
//     #[test]
//     fn basic_client_parse_error_handling_datatype() {
//         let xt = XTPacket {
//             handler_id: Some("s".to_owned()),
//             packet_id: "u#sp".to_owned(),
//             internal_id: 1,
//             data: vec!["foobar".to_owned(), "384".to_owned()],
//         };
//
//         let res: Result<meta::client::Packet, client::PacketError> = (&xt).try_into();
//         assert_matches!(res, Err(client::PacketError::BadDatatypeInt(_)))
//     }
// }
