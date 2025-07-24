use std::time::Duration;

use crate::server::{
    self, state, system::{EventReceiver, EventSender}, Event
};
use anyhow::Result;
use async_trait::async_trait;

pub struct Heartbeat;

#[async_trait]
impl server::system::System for Heartbeat {
    async fn instantiate(
        &self,
        _server: state::ServerState,
        mut event_tx: EventSender,
        mut event_rx: EventReceiver,
    ) -> Result<()> {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                event_tx.push(Event::Heartbeat).await;
            }
        });

        tokio::spawn(async move {
            while let Some(event) = event_rx.poll().await {
                match event {
                    // Event::Heartbeat => log::debug!("heartbeat received"),
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
