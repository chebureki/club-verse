use anyhow::{anyhow, Context, Result};

use crate::{
    conn::line::{self, LineConnReader, LineConnWriter},
    pkt::{self, meta},
};

pub enum AuthResult {
    Unauthenticated,
    Authenticated(meta::PlayerId),
}

pub async fn gate(
    writer: LineConnWriter,
    reader: LineConnReader,
) -> Result<(AuthResult, LineConnWriter, LineConnReader)> {
    match login_loop(writer, reader)
        .await
        // TODO: log connection?
        .context("failure in login loop")?
    {
        (None, writer, reader) => Ok((AuthResult::Unauthenticated, writer, reader)),
        (Some(player_id), writer, reader) => {
            Ok((AuthResult::Authenticated(player_id), writer, reader))
        }
    }
}

async fn login_loop(
    writer: LineConnWriter,
    reader: LineConnReader,
) -> Result<(Option<meta::PlayerId>, LineConnWriter, LineConnReader)> {
    let mut writer = writer;
    let mut reader = reader;

    let (username, _password) = loop {
        log::info!("waiting for input????");
        let line = reader.read().await;
        match line {
            // TODO: BAD: user error and server error are not differentiated
            Err(line::ReadError::EnvError(e)) => return Err(e),
            Err(line::ReadError::ParseError(e)) => {
                log::warn!("line is not parseable xml: {}", e);
                continue;
            }
            Ok(None) => {
                return Err(anyhow!("login loop was quit early!"));
            }
            Ok(Some(pkt::xml::client::Packet::VersionCheck { expected })) => {
                log::info!("client expects version {expected}");
                writer.write(pkt::xml::server::Packet::ApiOK).await.unwrap();
            }

            Ok(Some(pkt::xml::client::Packet::RandomKey)) => {
                writer
                    .write(pkt::xml::server::Packet::RandomKey("houdini".to_owned()))
                    .await
                    .unwrap();
            }

            Ok(Some(pkt::xml::client::Packet::Login { username, password })) => {
                break (username, password);
            }
        }
    };

    if &username != "kirill" {
        writer
            .write(pkt::xt::as2::server::Packet(meta::server::Packet::Error(
                meta::server::Error::NameNotFound,
            )))
            .await
            .unwrap();
        return Ok((None, writer, reader));
    }
    Ok((Some(102), writer, reader))
}
