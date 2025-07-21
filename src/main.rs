pub mod conn;
pub mod pkt;
pub mod server;

use anyhow::{Context, Error};
use env_logger::Env;

use crate::conn::server_list;

#[tokio::main()]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    // log::info!("This is an informational message.");

    server_list::ServerList::new()
        .await
        .context("failed to init server list login")?
        .bind()
        .await
        .context("failed to bind to address for server list login ")?;

    Ok(())
}
