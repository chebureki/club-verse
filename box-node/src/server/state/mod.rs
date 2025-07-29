use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::Result;
use tokio::sync::RwLock;

use crate::{
    datamodel::{self, RoomId},
    pkt::meta,
};

#[derive(Debug)]
pub struct Server {
    penguins: HashMap<meta::PlayerId, Player>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: meta::PlayerId,
    pub nickname: String,
    pub room: Option<RoomId>,
    pub x: isize,
    pub y: isize,
}

impl Into<datamodel::PlayerGist> for Player {
    fn into(self) -> datamodel::PlayerGist {
        datamodel::PlayerGist {
            id: self.id,
            nickname: self.nickname,
            approval: false,
            color: 1,
            head: 429,
            face: 0,
            neck: 0,
            body: 0,
            hand: 0,
            feet: 0,
            flag: 0,
            photo: 0,
            x: self.x,
            y: self.y,
            frame: 1,
            member: true,
            membership_days: 9,
            avatar: 0,
            // TODO: IM
            // penguin_state: "".to_owned(),
            // party_state: "".to_owned(),
            puffle_state: datamodel::PlayerPuffleGist {},
        }
    }
}

impl Server {
    pub fn push_player(&mut self, player: Player) -> Result<()> {
        if self.penguins.contains_key(&player.id) {
            anyhow::bail!("player already in server");
        }
        self.penguins.insert(player.id, player);

        Ok(())
    }

    // TODO: make it a Result<>, if this can be triggered by the player
    pub fn get_player(&self, player_id: meta::PlayerId) -> &Player {
        self.penguins
            .get(&player_id)
            .expect("no such player ... bad state management!")
    }

    pub fn get_mut_player(&mut self, player_id: meta::PlayerId) -> &mut Player {
        self.penguins
            .get_mut(&player_id)
            .expect("no such player ... bad state management!")
    }

    pub fn pop_player(&mut self, player_id: meta::PlayerId) -> Result<()> {
        match self.penguins.remove(&player_id) {
            None => anyhow::bail!("player {} was not in server", player_id),
            Some(_) => Ok(()),
        }
    }

    pub fn room_players(&self, room_id: RoomId) -> impl Iterator<Item = &Player> + '_ {
        self.penguins
            .iter()
            .filter(move |(_, p)| p.room == Some(room_id))
            .map(|(_, p)| p)
    }
}

#[derive(Clone, Debug)]
pub struct ServerState(Arc<RwLock<Server>>);

impl ServerState {
    pub fn new() -> Self {
        let server: Server = Server {
            penguins: HashMap::with_capacity(256),
        };
        Self(Arc::new(RwLock::new(server)))
    }
}
impl Deref for ServerState {
    type Target = Arc<RwLock<Server>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl DerefMut for ServerState {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl
