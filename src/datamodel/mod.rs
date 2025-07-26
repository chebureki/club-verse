/* TODO:
 * some item "holders" are more restrictive.
 * I can't equip a shirt as a hat
 * this will do fine for now.
 * but we need some validation at some point ...
 *
 * perhaps we could do an enum?
 * but ehhhhh
 */
pub type ItemId = usize;
pub type PlayerId = usize;
pub type RoomId = usize;


// TODO: there seem to be four... no idea what they do
#[derive(Debug, Clone, PartialEq)]
pub enum MapCategory{
    Normal
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerGist {
    pub id: PlayerId,
    pub nickname: String,
    pub approval: bool,
    pub color: ItemId,
    pub head: ItemId,
    pub face: ItemId,
    pub neck: ItemId,
    pub body: ItemId,
    pub hand: ItemId,
    pub feet: ItemId,
    pub flag: ItemId,
    pub photo: ItemId,
    // TODO: the odd weird fields ...
    pub x: i32,
    pub y: i32,
    pub frame: u8,
    pub member: bool,
    pub membership_days: u32,
    pub avatar: ItemId,
    // TODO: NEEDED FOR AS3 BUT NOT AS2
    // pub penguin_state: String,
    // pub party_state: String,
    pub puffle_state: PlayerPuffleGist,
}

// TODO: i believe houdini is incorrect here!
// #[derive(Debug, Clone, PartialEq)]
// pub struct RoomGist{
//     players: Vec<PlayerGist>
// }

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerStampsGist {

}





#[derive(Debug, Clone, PartialEq)]
pub struct NewPlayerStatus{}
//     OpenedIglooViewer = 1
//     ActiveIglooLayoutOpenFlag = 2
//     PuffleTreasureInfographic = 512
//     PlayerOptInAbTestDayZero = 1024
//     PlayerSwapPuffle = 2048
//     MoreThanTenPufflesBackyardMessage = 4096
//     VisitBackyardFirstTime = 8192
//     HasWalkedPuffleFirstTime = 65536
//     HasWalkedPuffleSecondTime = 131072

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPuffleGist {}

/// Serialized information used in communication
pub trait IntoPlayerGistString {
    fn into_gist_string(self) -> String;
}

impl IntoPlayerGistString for PlayerGist {
    fn into_gist_string(self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.id,
            self.nickname,
            self.approval as u8,
            self.color,
            self.head,
            self.face,
            self.neck,
            self.body,
            self.hand,
            self.feet,
            self.flag,
            self.photo,
            self.x,
            self.y,
            self.frame,
            self.member as u8,
            self.membership_days,
            self.avatar,
            //TODO
            0, //self.penguin_state,
            0, //self.party_state,
            // TODO: implement for as3
            match self.puffle_state {
                PlayerPuffleGist {} => "||||",
            } // self.puffle_state,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_gist_real() {
        let player_gist = PlayerGist {
            id: 102,
            nickname: "Kirill".to_owned(),
            approval: false,
            color: 1,
            head: 429,
            face: 0,
            neck: 0,
            body: 0,
            hand: 0,
            feet: 0,
            flag: 0,
            photo: 0,
            x: 0,
            y: 0,
            frame: 1,
            member: true,
            membership_days: 9,
            avatar: 0,
            // TODO: IM
            // penguin_state: "".to_owned(),
            // party_state: "".to_owned(),
            puffle_state: PlayerPuffleGist {},
        };
        assert_eq!(
            player_gist.into_gist_string(),
            "102|Kirill|0|1|429|0|0|0|0|0|0|0|0|0|1|1|9|0|0|0|||||"
        )
    }
}
