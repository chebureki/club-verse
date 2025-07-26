use std::{
    fs::read,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    datamodel::{self, IntoPlayerGistString},
    pkt::meta,
    server::{
        state,
        system::{self, EventReceiver, EventSender},
        Event,
    },
};

pub struct Server;

#[async_trait]
impl system::System for Server {
    async fn instantiate(
        &self,
        server: state::ServerState,
        mut event_tx: EventSender,
        mut event_rx: EventReceiver,
    ) -> Result<()> {
        tokio::spawn(async move {
            loop {
                while let Some(event) = event_rx.poll().await {
                    match event {
                        Event::PlayerConnected(player_id) => {
                            log::info!("player {player_id} connected!");

                            event_tx
                                .push(Event::PacketSent(player_id, meta::server::Packet::Loaded))
                                .await;
                            // let player
                        }
                        Event::PlayerDisconnected(player_id) => {
                            log::info!("player {player_id} disconnected");
                            server.write().await.pop_player(player_id).unwrap();
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetIgnoreList) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetIgnoreList {},
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetInventory) => {
                            let server = server.read().await;
                            let _player = server.get_player(player_id);
                            let items = vec![1, 429, 9057, 339, 609, 8009];
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetInventory { items },
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetBuddies) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetBuddies {},
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::StartMailEngine) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::StartMailEngine {
                                        unread_mail_count: 0,
                                        mail_count: 2,
                                    },
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetEPFPoints) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetEPFPoints {
                                        career_medals: 0,
                                        agent_medals: 0,
                                    },
                                ))
                                .await;
                        }

                        Event::PacketReceived(
                            player_id,
                            meta::client::Packet::GetFieldOPStatus,
                        ) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetFieldOPStatus {},
                                ))
                                .await;
                        }
                        Event::PacketReceived(
                            player_id,
                            meta::client::Packet::GetEPFAgentStatus,
                        ) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetEPFAgentStatus {},
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetLastRevision) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetLastRevision("houdini".to_owned()),
                                ))
                                .await;
                        }
                        Event::PacketReceived(player_id, meta::client::Packet::GetMail) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetMail {},
                                ))
                                .await;
                        }
                        Event::PacketReceived(
                            player_id_requester,
                            meta::client::Packet::QueryPlayerAwards {
                                player_id: player_id_to_query,
                            },
                        ) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id_requester,
                                    meta::server::Packet::QueryPlayerAwards {
                                        player_id: player_id_to_query,
                                    },
                                ))
                                .await;
                        }
                        Event::PlayerTransferRoomRequest(player_id, room_id) => {
                            let mut server = server.write().await;
                            let player = server.get_mut_player(player_id);
                            player.room = Some(room_id);
                            event_tx
                                .push(Event::PlayerJoinedRoom(player_id, room_id))
                                .await;
                        }
                        Event::PacketReceived(
                            player_id,
                            meta::client::Packet::GetPlayer { player },
                        ) => {
                            let server = server.read().await;
                            // TODO: player can crash server!
                            let player = server.get_player(player).clone().into();
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetPlayer { player },
                                ))
                                .await;
                        }
                        Event::PacketReceived(
                            player_id,
                            meta::client::Packet::GetWaddlePopulation {},
                        ) => {
                            event_tx
                                .push(Event::PacketSent(
                                    player_id,
                                    meta::server::Packet::GetWaddlePopulation {},
                                ))
                                .await;
                        }
                        Event::PlayerJoinedRoom(player_id, room_id) => {
                            let server = server.read().await;

                            let joiner_gist: datamodel::PlayerGist =
                                server.get_player(player_id).clone().into();

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

                            for event in
                                server
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
                        }

                        Event::PacketReceived(
                            player_id,
                            meta::client::Packet::JoinServer {
                                penguin_id,
                                login_key: _,
                                language: _,
                            },
                        ) => {
                            // TODO: load from DB
                            let player = state::Player {
                                id: player_id,
                                room: None,
                                nickname: "kirill".to_owned(),
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
                                            * 1000)
                                            as usize,
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

                            event_tx
                                .push(Event::PlayerTransferRoomRequest(player_id, 230))
                                .await;
                        }
                        // event_t
                        _ => {}
                    }
                }
            }
        });
        Ok(())
    }
}

// current_time = int(time.time())
// penguin_standard_time = current_time * 1000
//
// pst = pytz.timezone(p.server.config.timezone)
// dt = datetime.fromtimestamp(current_time, pst)
// server_time_offset = abs(int(dt.strftime('%z')) // 100)
//
// if p.timer_active:
//     minutes_until_timer_end = datetime.combine(datetime.today(), p.timer_end) - datetime.now()
//     minutes_until_timer_end = minutes_until_timer_end.total_seconds() // 60
//
//     minutes_played_today = await get_minutes_played_today(p)
//     minutes_left_today = (p.timer_total.total_seconds() // 60) - minutes_played_today
//     p.egg_timer_minutes = int(min(minutes_until_timer_end, minutes_left_today))
// else:
//     p.egg_timer_minutes = 24 * 60
//
//
// spawn = random.choice(p.server.rooms.spawn_rooms)
// await p.join_room(spawn)
//
// p.server.penguins_by_id[p.id] = p
// p.server.penguins_by_username[p.username] = p
//
// if p.character is not None:
//     p.server.penguins_by_character_id[p.character] = p
//
// p.login_timestamp = datetime.now()
// p.joined_world = True
//
// server_key = f'houdini.players.{p.server.config.id}'
// await p.server.redis.sadd(server_key, p.id)
// await p.server.redis.hset('houdini.population', p.server.config.id, len(p.server.penguins_by_id))
