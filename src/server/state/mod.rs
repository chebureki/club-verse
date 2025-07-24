use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::Result;
use tokio::sync::RwLock;

use crate::pkt::meta;

#[derive(Debug)]
pub struct Server {
    penguins: HashMap<meta::PlayerId, Player>,
}

#[derive(Debug)]
pub struct Player {
    pub id: meta::PlayerId,
    pub nickname: String,
}

impl Server {
    pub fn push_player(&mut self, player: Player) -> Result<()> {
        if self.penguins.contains_key(&player.id) {
            anyhow::bail!("player already in server");
        }
        self.penguins.insert(player.id, player);

        Ok(())
    }

    pub fn pop_player(&mut self, player_id: meta::PlayerId) -> Result<()> {
        match self.penguins.remove(&player_id) {
            None => anyhow::bail!("player {} was not in server", player_id),
            Some(_) => Ok(()),
        }
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
