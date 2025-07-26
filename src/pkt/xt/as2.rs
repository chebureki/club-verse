pub mod client {
    use crate::{
        datamodel::{self, IntoPlayerGistString},
        pkt::{self, meta},
    };
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
                    [penguin_id, login_key, language] => Ok(meta::client::Packet::JoinServer {
                        penguin_id: penguin_id.parse()?,
                        login_key: login_key.to_owned(),
                        language: language.to_owned(),
                    }),
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
                ("s", "f#epfga") => match data {
                    [] => Ok(meta::client::Packet::GetEPFAgentStatus),
                    _ => Err(PacketError::BadArgCount),
                },
                ("s", "i#qpa") => match data {
                    [player_id] => Ok(meta::client::Packet::QueryPlayerAwards {
                        player_id: player_id.parse()?,
                    }),
                    _ => Err(PacketError::BadArgCount),
                },
                ("z", "gw") => match data {
                    _ => Ok(meta::client::Packet::GetWaddlePopulation {}),
                },
                ("s", "u#gp") => match data {
                    [player_id] => Ok(meta::client::Packet::GetPlayer {
                        player: player_id.parse()?,
                    }),
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
    use crate::{
        datamodel::{self, IntoPlayerGistString},
        pkt::{
            self,
            meta::ModeratorStatus,
            xt::{XTPacket, XT_DEFAULT_INT_ID},
        },
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
                pkt::meta::server::Packet::LoadPlayer {
                    gist,
                    coins,
                    safe_chat,
                    egg_timer_minutes,
                    penguin_standard_time,
                    age,
                    minutes_played,
                    membership_days_remain,
                    server_time_offset,
                    opened_playercard,
                    map_category,
                    new_player_status,
                } => XTPacket {
                    handler_id: None,
                    packet_id: "lp".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![
                        gist.into_gist_string(),
                        coins.to_string(),
                        if safe_chat {
                            "1".to_owned()
                        } else {
                            "0".to_owned()
                        },
                        egg_timer_minutes.to_string(),
                        penguin_standard_time.to_string(),
                        age.to_string(),
                        "0".to_owned(),
                        minutes_played.to_string(),
                        membership_days_remain.to_string().to_owned(),
                        server_time_offset.to_string().to_owned(),
                        if opened_playercard {
                            "1".to_owned()
                        } else {
                            "0".to_owned()
                        },
                        match map_category {
                            datamodel::MapCategory::Normal => "0".to_owned(),
                        },
                        match new_player_status {
                            _ => "0".to_owned(),
                        },
                    ],
                },
                pkt::meta::server::Packet::GetInventory { items } => XTPacket {
                    handler_id: None,
                    packet_id: "gi".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: items.into_iter().map(|i| i.to_string()).collect(),
                },
                pkt::meta::server::Packet::GetBuddies {} => XTPacket {
                    handler_id: None,
                    packet_id: "gb".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![],
                },
                pkt::meta::server::Packet::GetIgnoreList {} => XTPacket {
                    handler_id: None,
                    packet_id: "gn".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![],
                },
                pkt::meta::server::Packet::GetPlayerStamps { player_id } => XTPacket {
                    handler_id: None,
                    packet_id: "gps".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![player_id.to_string(), "93|96|189".to_owned()],
                },

                pkt::meta::server::Packet::QueryPlayerAwards { player_id } => XTPacket {
                    handler_id: None,
                    packet_id: "qpa".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![player_id.to_string(), "8009".to_owned()],
                },

                pkt::meta::server::Packet::GetMail {} => XTPacket {
                    handler_id: None,
                    packet_id: "mg".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![
                        "sys|0|112||1752891915|2|1".to_owned(),
                        "sys|0|125||1752251683|1|1".to_owned(),
                    ],
                },
                pkt::meta::server::Packet::GetLastRevision(revision) => XTPacket {
                    handler_id: None,
                    packet_id: "glr".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![revision],
                },
                pkt::meta::server::Packet::StartMailEngine {
                    unread_mail_count,
                    mail_count,
                } => XTPacket {
                    handler_id: None,
                    packet_id: "mst".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![unread_mail_count.to_string(), mail_count.to_string()],
                },

                pkt::meta::server::Packet::GetEPFPoints {
                    career_medals,
                    agent_medals,
                } => XTPacket {
                    handler_id: None,
                    packet_id: "epfgr".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![career_medals.to_string(), agent_medals.to_string()],
                },

                pkt::meta::server::Packet::GetFieldOPStatus {} => XTPacket {
                    handler_id: None,
                    packet_id: "epfgf".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec!["0".to_owned()],
                },

                pkt::meta::server::Packet::GetEPFAgentStatus {} => XTPacket {
                    handler_id: None,
                    packet_id: "epfga".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec!["1".to_owned()],
                },

                pkt::meta::server::Packet::JoinRoom { room_id, players } => XTPacket {
                    handler_id: None,
                    packet_id: "jr".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: {
                        let mut vec: Vec<String> = Vec::new();
                        vec.push(room_id.to_string());
                        for p in players {
                            vec.push(p.into_gist_string());
                        }
                        vec
                    },
                },
                pkt::meta::server::Packet::AddedPlayer { player } => XTPacket {
                    handler_id: None,
                    packet_id: "ap".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![player.into_gist_string()],
                },
                pkt::meta::server::Packet::GetWaddlePopulation {} => XTPacket {
                    handler_id: None,
                    packet_id: "gw".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec!["100|,,,%101|,,%102|,%103|,".to_owned()],
                },
                pkt::meta::server::Packet::GetPlayer { player } => XTPacket {
                    handler_id: None,
                    packet_id: "gp".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![player.into_gist_string()],
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
