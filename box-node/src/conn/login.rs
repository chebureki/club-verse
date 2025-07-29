use anyhow::Result;

use crate::pkt;


pub struct LoginHandler {}

pub enum LoginResp {
    /// User finished
    HandShook,

    /// Forward Data
    Packet(pkt::xml::server::Packet),


    // TODO: we need some termination logic!!!
}

impl LoginHandler {
    pub async fn new() -> Result<Self> {
        Ok(Self{})
    }

    pub async fn handle(&mut self, packet: &pkt::xml::client::Packet) -> Result<LoginResp> {
        match packet {
            pkt::xml::client::Packet::VersionCheck { expected } => {
                log::info!("client expects version: {}", expected);
                Ok(LoginResp::Packet(pkt::xml::server::Packet::ApiOK))
            }
            pkt::xml::client::Packet::RandomKey => Ok(LoginResp::Packet(
                pkt::xml::server::Packet::RandomKey("houdini".to_owned())
            )),
            pkt::xml::client::Packet::Login { username, password: _ } => {
                log::info!("user: {username} is logging in!");
                Ok(LoginResp::HandShook)
            }
        }
    }
}
