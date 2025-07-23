pub mod heartbeat;
pub mod socket;


use async_trait::async_trait;
use anyhow::Result;
use tokio::sync::broadcast;

use crate::server::Event;

#[async_trait]  
pub trait System {
    async fn instantiate(&self, publisher: broadcast::Sender<Event>) -> Result<()>;
}
