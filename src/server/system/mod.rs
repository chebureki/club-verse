pub mod heartbeat;
pub mod server;
pub mod socket;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::server::{state, Event};

#[async_trait]
pub trait System {
    async fn instantiate(
        &self,
        server: state::ServerState,
        tx: EventSender,
        rx: EventReceiver,
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct EventSender(pub(crate) broadcast::WeakSender<Event>);

pub struct EventReceiver(pub(crate) broadcast::Receiver<Event>);

impl EventSender {
    /* NOTE:
     * Arrival is not guranteed!
     * The only scenario where the payload is not delivered
     * is when upstream is closed, and won't receive anyway
     */
    pub async fn push(&mut self, event: Event) -> () {
        if let Some(tx) = self.0.upgrade() {
            match tx.send(event) {
                Ok(_) => {}
                Err(event) => {
                    log::warn!("dropping event {event}");
                }
            }
        }
    }
}

impl EventReceiver {
    pub async fn poll(&mut self) -> Option<Event> {
        loop {
            match self.0.recv().await {
                Ok(event) => return Some(event),
                Err(RecvError::Closed) => return None,
                Err(RecvError::Lagged(n)) => {
                    log::warn!("EventBus lag!!! Dropped {n} events");
                    continue;
                }
            }
        }
    }
}
