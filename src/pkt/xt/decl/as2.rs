pub mod client {
    use std::num::ParseIntError;

    use thiserror::Error;

    use crate::pkt::xt::XTPacket;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        PlayerSetPosition(PlayerSetPosition),
    }
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
    pub struct PlayerSetPosition {
        pub x: usize,
        pub y: usize,
    }

    impl TryFrom<&XTPacket> for Packet {
        type Error = PacketError;
        fn try_from(value: &XTPacket) -> Result<Self, Self::Error> {
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
                ("s", "u#sp", [x, y]) => Ok(Packet::PlayerSetPosition(PlayerSetPosition {
                    x: x.parse()?,
                    y: y.parse()?,
                })),
                ("s", "u#sp", _) => Err(PacketError::BadArgCount),

                _ => Err(PacketError::Unrecognized {
                    handler_id: handler_id.to_owned(),
                    packet_id: packet_id.to_owned(),
                }),
            }
        }
    }
}

pub mod server {
    use crate::pkt::xt::{XTPacket, XT_DEFAULT_INT_ID};

    // TODO: move somewhere else
    pub type PlayerId = usize;
    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        PlayerSetPosition(PlayerSetPosition),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerSetPosition {
        pub player_id: usize,
        pub x: usize,
        pub y: usize,
    }

    impl Into<XTPacket> for &Packet {
        fn into(self) -> XTPacket {
            match self {
                Packet::PlayerSetPosition(PlayerSetPosition { player_id, x, y }) => XTPacket {
                    handler_id: None,
                    packet_id: "123".to_owned(),
                    internal_id: XT_DEFAULT_INT_ID,
                    data: vec![player_id.to_string(), x.to_string(), y.to_string()],
                },
            }
        }
    }
}

#[cfg(test)]
mod as2_packet_tests {
    use crate::pkt::xt::XTPacket;
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn basic_client_parse() {
        let xt = XTPacket {
            handler_id: Some("s".to_owned()),
            packet_id: "u#sp".to_owned(),
            internal_id: 1,
            data: vec!["395".to_owned(), "384".to_owned()],
        };

        let res: Result<client::Packet, client::PacketError> = (&xt).try_into();
        assert_matches!(
            res,
            Ok(client::Packet::PlayerSetPosition(
                client::PlayerSetPosition { x: 395, y: 384 }
            ))
        )
    }

    #[test]
    fn basic_client_parse_error_handling_count() {
        let xt = XTPacket {
            handler_id: Some("s".to_owned()),
            packet_id: "u#sp".to_owned(),
            internal_id: 1,
            data: vec!["395".to_owned()],
        };

        let res: Result<client::Packet, client::PacketError> = (&xt).try_into();
        assert_matches!(res, Err(client::PacketError::BadArgCount))
    }

    #[test]
    fn basic_client_parse_error_handling_datatype() {
        let xt = XTPacket {
            handler_id: Some("s".to_owned()),
            packet_id: "u#sp".to_owned(),
            internal_id: 1,
            data: vec!["foobar".to_owned(), "384".to_owned()],
        };

        let res: Result<client::Packet, client::PacketError> = (&xt).try_into();
        assert_matches!(res, Err(client::PacketError::BadDatatypeInt(_)))
    }
}
