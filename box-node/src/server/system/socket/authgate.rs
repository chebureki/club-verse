use std::fs::read;

use anyhow::{anyhow, Context, Result};

use crate::{
    auth::AuthHandle, conn::line::{self, LineConnReader, LineConnWriter}, pkt::{self, meta}
};

pub enum AuthResult {
    Unauthenticated,
    Authenticated(meta::PlayerId),
}

pub async fn gate(
    auth: AuthHandle,
    writer: LineConnWriter,
    reader: LineConnReader,
) -> Result<(AuthResult, LineConnWriter, LineConnReader)> {
    match login_loop(auth, writer, reader)
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


/* NOTE:
 * we discard the password entirely
 * in the customized SWF, we imply a JWT is passed in the username field
 * ...yea ugly, but a more pretty solution would require a more thourough hack 
 * of the boot sequence
 * the original code SUCKS ASS and I hate decompiling and recompiling actionscript!!!
 */
async fn login_loop(
    auth: AuthHandle,
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
    let token = username.as_str();
    let user= match {auth.read().await.verify_user(token)}{
        Ok(user) => user,
        Err(e) => return Err(e).context("failure in token validation"),
    };
    log::info!("woooo {}", user);
    Ok((Some(102), writer, reader))
    // let auth.read(path)


    // let user_id = match username.as_str() {
    //     "kirill" => 102,
    //     "peter" => 103,
    //     _ => {
    //         writer
    //             .write(pkt::xt::as2::server::Packet(meta::server::Packet::Error(
    //                 meta::server::Error::NameNotFound,
    //             )))
    //             .await
    //             .unwrap();
    //         return Ok((None, writer, reader));
    //     }
    // };
    // Ok((Some(user_id), writer, reader))
}
