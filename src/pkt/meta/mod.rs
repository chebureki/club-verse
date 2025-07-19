pub type PlayerId = usize;
pub mod client {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        PlayerSetPosition(PlayerSetPosition),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerSetPosition {
        pub x: usize,
        pub y: usize,
    }
}

pub mod server {
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
}
