use std::{
    fs::read,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::{
    datamodel::{self},
    pkt::meta,
    server::{
        resp::Request,
        state,
        system::{self, EventReceiver, EventSender},
        Event,
    },
};

pub type Result<T> = std::result::Result<T, ServerError>;
pub struct Server;

#[derive(Debug)]
pub enum ServerError {
    // player sends something stupid, TODO: perhaps log? give context?
    PlayerFault(datamodel::PlayerId, meta::server::Error),
    ServerFault(anyhow::Error),
}

#[inline]
async fn handle_player_connected(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    log::info!("player {player_id} connected!");

    event_tx
        .push(Event::PacketSent(player_id, meta::server::Packet::Loaded))
        .await;
    // let player
    Ok(())
}

#[inline]
async fn handle_player_disconnected(
    player_id: meta::PlayerId,
    server: state::ServerState,
    _event_tx: EventSender,
) -> Result<()> {
    log::info!("player {player_id} disconnected");
    // TODO: UPDATE CONNECTED PEOPLE
    match server.write().await.pop_player(player_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerError::ServerFault(e)),
    }
}

#[inline]
async fn handle_get_ignore_list(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetIgnoreList {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_set_position(
    player_id: meta::PlayerId,
    x: isize,
    y: isize,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let mut server = server.write().await;
    let player = server.get_mut_player(player_id);
    let room_id = player.room.unwrap();
    player.x = x;
    player.y = y;

    for e in server
        .room_players(room_id)
        .map(|state::Player { id, .. }| {
            Event::PacketSent(*id, meta::server::Packet::SetPosition { player_id, x, y })
        })
    {
        event_tx.push(e).await;
    }

    // TODO: update frame and toy!!
    Ok(())
}

#[inline]
async fn handle_get_inventory(
    player_id: meta::PlayerId,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let server = server.read().await;
    let _player = server.get_player(player_id);
    let items = vec![1, 429, 9057, 339, 609, 8009];
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetInventory { items },
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_buddies(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetBuddies {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_start_mail_engine(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::StartMailEngine {
                unread_mail_count: 0,
                mail_count: 2,
            },
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_epf_points(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetEPFPoints {
                career_medals: 0,
                agent_medals: 0,
            },
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_field_op_status(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetFieldOPStatus {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_epf_agent_status(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetEPFAgentStatus {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_last_revision(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetLastRevision("houdini".to_owned()),
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_mail(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetMail {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_query_player_awards(
    player_id_requester: meta::PlayerId,
    player_id_to_query: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id_requester,
            meta::server::Packet::QueryPlayerAwards {
                player_id: player_id_to_query,
            },
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_player_transfer_room_request(
    player_id: meta::PlayerId,
    room_id: datamodel::RoomId,
    server: state::ServerState,
    mut event_tx: EventSender,
    req: Request<()>,
) -> Result<()> {
    let mut server = server.write().await;
    let player = server.get_mut_player(player_id);
    let prev_room = player.room.clone();
    player.room = Some(room_id);

    if let Some(room_id) = prev_room {
        for member in server.room_players(room_id) {
            event_tx
                .push(Event::PacketSent(
                    member.id,
                    meta::server::Packet::RemovePenguin { player_id },
                ))
                .await;
        }
    }
    event_tx
        .push(Event::PlayerJoinedRoom(player_id, room_id))
        .await;
    req.fulfill(()).await;
    Ok(())
}

#[inline]
async fn handle_get_player(
    player_id: meta::PlayerId,
    player: meta::PlayerId,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let server = server.read().await;
    // TODO: player can crash server!
    let player = server.get_player(player).clone().into();
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetPlayer { player },
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_get_waddle_population(
    player_id: meta::PlayerId,
    _server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetWaddlePopulation {},
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_player_joined_room(
    player_id: meta::PlayerId,
    room_id: datamodel::RoomId,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let server = server.read().await;

    let joiner_gist: datamodel::PlayerGist = server.get_player(player_id).clone().into();

    let gists: Vec<datamodel::PlayerGist> = server
        .room_players(room_id)
        .map(|p| p.clone().into())
        .collect();
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::JoinRoom {
                room_id,
                players: gists,
            },
        ))
        .await;

    for event in server
        .room_players(room_id)
        .map(|state::Player { id, .. }| {
            Event::PacketSent(
                *id,
                meta::server::Packet::AddedPlayer {
                    player: joiner_gist.clone(),
                },
            )
        })
    {
        event_tx.push(event).await;
    }
    Ok(())
}

#[inline]
async fn handle_send_message(
    player_id: meta::PlayerId,
    message: String,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let server = server.read().await;
    let room = server.get_player(player_id).room.unwrap();
    for e in server.room_players(room).map(|p| {
        Event::PacketSent(
            p.id,
            meta::server::Packet::SendMessage {
                player_id,
                message: message.clone(),
            },
        )
    }) {
        event_tx.push(e).await;
    }
    Ok(())
}

#[inline]
async fn handle_join_room(
    player_id: meta::PlayerId,
    room_id: datamodel::RoomId,
    x: isize,
    y: isize,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    let (rx, tx) = Request::new();
    event_tx
        .push(Event::PlayerTransferRoomRequest(player_id, room_id, tx))
        .await;
    match rx.recv().await {
        Ok(()) => Ok(()),
        // TODO: completely incorrect!!!
        Err(e) => Err(ServerError::PlayerFault(
            player_id,
            meta::server::Error::RoomDoesNotExist,
        )),
    }
}

#[inline]
async fn handle_heartbeat(player_id: meta::PlayerId, mut event_tx: EventSender) -> Result<()> {
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::Heartbeat,
        ))
        .await;
    Ok(())
}

#[inline]
async fn handle_join_server(
    player_id: meta::PlayerId,
    penguin_id: meta::PlayerId,
    server: state::ServerState,
    mut event_tx: EventSender,
) -> Result<()> {
    // TODO: load from DB
    let player = state::Player {
        id: player_id,
        room: None,
        nickname: "kirill_{player_id}".to_owned(),
        x: 0,
        y: 0,
    };

    // TODO: what if player is already connected
    // TODO: handle login ket
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::ActiveFeatures {},
        ))
        .await;

    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::LoadPlayer {
                gist: player.clone().into(),
                coins: 100,
                safe_chat: false,
                egg_timer_minutes: 100,
                penguin_standard_time: (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time not available?")
                    .as_secs()
                    * 1000) as usize,
                age: 0,
                minutes_played: 10,
                membership_days_remain: 1000,
                server_time_offset: 7,
                opened_playercard: true,
                map_category: datamodel::MapCategory::Normal,
                new_player_status: datamodel::NewPlayerStatus {},
            },
        ))
        .await;

    server.write().await.push_player(player).unwrap();
    event_tx
        .push(Event::PacketSent(
            penguin_id,
            meta::server::Packet::JoinedServer {
                agent_status: false,
                moderator_status: meta::ModeratorStatus::None,
                book_modified: false,
            },
        ))
        .await;
    event_tx
        .push(Event::PacketSent(
            player_id,
            meta::server::Packet::GetPlayerStamps { player_id },
        ))
        .await;

    drop(server);
    let (resp, req) = Request::new();
    event_tx
        .push(Event::PlayerTransferRoomRequest(player_id, 230, req))
        .await;
    match resp.recv().await {
        Ok(v) => log::info!("YAAY ALL GOOD"),
        Err(e) => {
            log::error!("YAAAY TIMEOUT")
        }
    }
    Ok(())
}

#[inline]
async fn handle_server_event(
    event: Event,
    server: state::ServerState,
    event_tx: EventSender,
) -> Result<()> {
    match event {
        Event::PlayerConnected(player_id) => {
            handle_player_connected(player_id, server, event_tx).await?;
        }
        Event::PlayerDisconnected(player_id) => {
            handle_player_disconnected(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetIgnoreList) => {
            handle_get_ignore_list(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::SetPosition { x, y }) => {
            handle_set_position(player_id, x, y, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetInventory) => {
            handle_get_inventory(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetBuddies) => {
            handle_get_buddies(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::StartMailEngine) => {
            handle_start_mail_engine(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetEPFPoints) => {
            handle_get_epf_points(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetFieldOPStatus) => {
            handle_get_field_op_status(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetEPFAgentStatus) => {
            handle_get_epf_agent_status(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetLastRevision) => {
            handle_get_last_revision(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetMail) => {
            handle_get_mail(player_id, server, event_tx).await?;
        }
        Event::PacketReceived(
            player_id_requester,
            meta::client::Packet::QueryPlayerAwards {
                player_id: player_id_to_query,
            },
        ) => {
            handle_query_player_awards(player_id_requester, player_id_to_query, server, event_tx)
                .await?;
        }
        Event::PlayerTransferRoomRequest(player_id, room_id, req) => {
            handle_player_transfer_room_request(player_id, room_id, server, event_tx, req).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetPlayer { player }) => {
            handle_get_player(player_id, player, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::GetWaddlePopulation {}) => {
            handle_get_waddle_population(player_id, server, event_tx).await?;
        }
        Event::PlayerJoinedRoom(player_id, room_id) => {
            handle_player_joined_room(player_id, room_id, server, event_tx).await?;
        }
        Event::PacketReceived(player_id, meta::client::Packet::SendMessage { message }) => {
            handle_send_message(player_id, message, server, event_tx).await?;
        }

        Event::PacketReceived(player_id, meta::client::Packet::Heartbeat) => {
            handle_heartbeat(player_id, event_tx).await?;
        }
        Event::PacketReceived(
            player_id,
            meta::client::Packet::JoinRoom {
                room: room_id,
                x,
                y,
            },
        ) => {
            handle_join_room(player_id, room_id, x, y, server, event_tx).await?;
        }
        Event::PacketReceived(
            player_id,
            meta::client::Packet::JoinServer {
                penguin_id,
                login_key: _,
                language: _,
            },
        ) => {
            handle_join_server(player_id, penguin_id, server, event_tx).await?;
        }
        _ => {}
    }
    Ok(())
}

#[async_trait]
impl system::System for Server {
    async fn instantiate(
        &self,
        server: state::ServerState,
        event_tx: EventSender,
        mut event_rx: EventReceiver,
    ) -> std::result::Result<(), anyhow::Error> {
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(16));

            while let Some(event) = event_rx.poll().await {
                let event_tx = event_tx.clone();
                let server = server.clone();
                let semaphore = Arc::clone(&semaphore);

                tokio::spawn(async move {
                    /*
                     * TODO:
                     * non user requests should bypass the semaphore!!!
                     * perhaps we also should have some mechanism to
                     * have a per player semaphore!
                     * Otherwise a hacker could takeup all workers
                     * and lag the server
                     */
                    let permit: Option<SemaphorePermit<'_>> = match semaphore.acquire().await {
                        Ok(permit) => Some(permit),
                        Err(_e) => {
                            log::warn!("failed to acquire semaphore! Dropping event");
                            return;
                        }
                    };
                    match handle_server_event(event, server, event_tx).await {
                        Ok(_) => {}
                        Err(ServerError::ServerFault(e)) => {
                            log::error!("SERVER FAULT: {e}");
                        }
                        Err(ServerError::PlayerFault(_player_id, _error)) => {
                            todo!("handle player fault")
                        }
                    }
                    drop(permit);
                });
            }
        });
        Ok(())
    }
}
