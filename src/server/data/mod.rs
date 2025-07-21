pub mod state;
pub mod persistent;
//
// pub mod persistence;
//
// use crate::pkt::meta::{self, PlayerId, RoomId};
//
//
//
// /* TODO:
//  * we need to differntiate emphemral and persistent state!
//  * I don't have a perfect model for it ... yet ha!
//  */
// pub struct Player {
//     id: PlayerId,
//     username: String,
//     room: Option<RoomMembership>,
// }
//
// /* TODO:
//  * membership is easy to keep correct in memory
//  * but we need a proper mechanism for signaling to clients!
//  */
// pub struct RoomMembership {
//     room: meta::RoomId,
//     position: (usize, usize),
// }
