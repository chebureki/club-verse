use anyhow::Result;
use async_trait::async_trait;

use crate::{
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
                            // await p.send_xt('activefeatures')
                            //
                            // moderator_status = 3 if p.character else 2 if p.stealth_moderator else 1 if p.moderator else 0
                            //
                            // await p.send_xt('js', int(p.agent_status), int(0),
                            //                 moderator_status, int(p.book_modified))
                            //
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
                            // await p.send_xt('lp', await p.string, p.coins, int(p.safe_chat), p.egg_timer_minutes,
                            //                 penguin_standard_time, p.age, 0, p.minutes_played,
                            //                 p.membership_days_remain, server_time_offset, int(p.opened_playercard),
                            //                 p.map_category, p.status_field)
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
                                .await
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(())
    }
}
