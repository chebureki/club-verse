pub mod client {
    use crate::pkt::{self, meta};
    use std::num::ParseIntError;

    use anyhow::Context;
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
            } = &value;

            // TODO: is it guranteed that this is a logic error at this point?
            let handler_id = match handler_id {
                Some(hi) => hi,
                None => panic!("attempt to parse a server packet as a client packet"),
            };

            let data = data.as_slice();
            let meta: meta::client::Packet = (match (handler_id.as_str(), packet_id.as_str()) {
                ("s", "j#js") => match data {
                    [penguin_id, login_key, language] => {
                        Ok(meta::client::Packet::JoinServer {
                            penguin_id: penguin_id.parse()?,
                            login_key: login_key.to_owned(),
                            language: language.to_owned(),
                        })
                    }
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "i#gi") => match data {
                    [] => Ok(meta::client::Packet::GetInventory),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "b#gb") => match data {
                    [] => Ok(meta::client::Packet::GetBuddies),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "n#gn") => match data {
                    [] => Ok(meta::client::Packet::GetIgnoreList),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "l#mst") => match data {
                    [] => Ok(meta::client::Packet::StartMailEngine),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "l#mg") => match data {
                    [] => Ok(meta::client::Packet::GetMail),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "p#pgu") => match data {
                    [] => Ok(meta::client::Packet::GetMyPuffles),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "u#glr") => match data {
                    [] => Ok(meta::client::Packet::GetLastRevision),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "f#epfgr") => match data {
                    [] => Ok(meta::client::Packet::GetEPFPoints),
                    _ => Err(PacketError::BadArgCount),
                },

                ("s", "f#epfgf") => match data {
                    [] => Ok(meta::client::Packet::GetEPFPoints),
                    _ => Err(PacketError::BadArgCount),
                },
                _ => Err(PacketError::Unrecognized {
                    handler_id: handler_id.to_owned(),
                    packet_id: packet_id.to_owned(),
                }),
            })?;
            Ok(Packet(meta))
        }
    }
}

pub mod server {
    use crate::pkt::{
        self,
        meta::ModeratorStatus,
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
                pkt::meta::server::Packet::Loaded => XTPacket {
                    handler_id: None,
                    packet_id: "l".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![],
                },
                pkt::meta::server::Packet::LoginResponse {} => XTPacket {
                    handler_id: None,
                    packet_id: "l".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    // TODO: why does houdini return empty?
                    data: vec!["".to_owned()],
                },
                pkt::meta::server::Packet::ActiveFeatures {} => XTPacket {
                    handler_id: None,
                    packet_id: "activefeatures".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![],
                },
                pkt::meta::server::Packet::JoinedServer {
                    agent_status,
                    moderator_status,
                    book_modified,
                } => XTPacket {
                    handler_id: None,
                    packet_id: "js".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![
                        if agent_status {
                            "1".to_owned()
                        } else {
                            "0".to_owned()
                        },
                        "0".to_owned(), // TODO: wtf is this? beta?
                        match moderator_status {
                            ModeratorStatus::Mascot => 3,
                            ModeratorStatus::StealthModerator => 2,
                            ModeratorStatus::Moderator => 1,
                            ModeratorStatus::None => 0,
                        }
                        .to_string(),
                        if book_modified {
                            "1".to_owned()
                        } else {
                            "0".to_owned()
                        }, // await p.send_xt('js', int(p.agent_status), int(0),moderator_status, int(p.book_modified))
                    ],
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
