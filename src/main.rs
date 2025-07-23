pub mod conn;
pub mod persistence;
pub mod pkt;
pub mod server;

use std::time::Duration;

use anyhow::{Context, Error};
use env_logger::Env;

use crate::conn::server_list;

#[tokio::main()]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    server::bind("0.0.0.0:1337").await?;

    tokio::time::sleep(Duration::from_secs(600)).await;
    Ok(())
}
