pub mod conn;
pub mod persistence;
pub mod pkt;
pub mod server;

use std::time::Duration;

use anyhow::{Context, Error, Result};
use env_logger::Env;

use crate::conn::server_list;

#[tokio::main()]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let server_tx = server::bind("0.0.0.0:1337").await?;

    tokio::signal::ctrl_c().await?;
    log::info!("terminating ...");
    drop(server_tx);
    tokio::time::sleep(Duration::from_secs(3)).await;
    Ok(())
    // tokio::time::sleep(Duration::from_secs(600)).await;
    // drop(tx);
    // Ok(())
}
