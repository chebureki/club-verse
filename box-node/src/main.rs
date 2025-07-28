pub mod auth;
pub mod conn;
pub mod datamodel;
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


            // "http://localhost:8080/realms/box-verse/protocol/openid-connect/certs".to_string(),
            // "http://localhost:8080/realms/box-verse".to_string(),
            // vec!["account".to_string()],
            // Duration::from_secs(10000),
    // Initialize JWT authentication
    let jwks_url =
        "http://keycloak:8080/realms/box-verse/protocol/openid-connect/certs".to_string();
    let issuer = "http://keycloak:8080/realms/box-verse".to_string();
    let audience = vec!["account".to_string()];

    let auth = auth::Auth::new(jwks_url, issuer, audience, Duration::from_secs(60 * 30))
        .await
        .context("Failed to initialize JWT authentication")?;

    let server_tx = server::bind("0.0.0.0:1337", auth).await?;

    tokio::signal::ctrl_c().await?;
    drop(server_tx);
    Ok(())
}
