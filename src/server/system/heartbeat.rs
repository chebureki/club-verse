use std::time::Duration;

use crate::server::{self, Event};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

pub struct Heartbeat;

#[async_trait]
impl server::system::System for Heartbeat {
    async fn instantiate(&self, publisher: broadcast::Sender<Event>) -> Result<()> {
        let mut rx = publisher.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                publisher.send(Event::Heartbeat).unwrap();
            }
        });

        tokio::spawn(async move {
            loop {
                let event = match rx.recv().await {
                    Ok(event) => event,
                    Err(_e) => break,
                };

                match event {
                    Event::Heartbeat => log::debug!("heartbeat received"),
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
