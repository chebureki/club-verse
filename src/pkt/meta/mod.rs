pub type PlayerId = usize;
pub type RoomId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum ModeratorStatus {
    Mascot,
    StealthModerator,
    Moderator,
    None, //lol
}
pub mod client {
    use crate::{datamodel, pkt::meta::PlayerId};

    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        Heartbeat,
        JoinServer {
            penguin_id: PlayerId,
            login_key: String,
            language: String,
        },
        GetInventory,
        GetBuddies,
        GetIgnoreList,
        StartMailEngine,
        GetMail,
        GetMyPuffles,
        GetLastRevision,
        GetEPFPoints,
        GetFieldOPStatus,
        GetEPFAgentStatus,
        QueryPlayerAwards {
            player_id: PlayerId,
        },
        // TODO:
        GetWaddlePopulation {},
        GetPlayer {
            player: datamodel::PlayerId,
        },
        SetPosition {
            x: isize,
            y: isize,
        },
        SendMessage{
            message: String,
        }
    }
}

pub mod server {
    use std::usize;

    use crate::{datamodel, pkt::meta::ModeratorStatus};

    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        Heartbeat,
        Error(Error),
        Loaded,
        LoginResponse {
            // hash: String,
            // // TODO: why does it have a pipe?
            // buddies_online: String,
        },
        ActiveFeatures {
            // TODO
        },
        JoinedServer {
            // TODO: fields should be more semantic and less codey
            agent_status: bool,
            moderator_status: ModeratorStatus,
            // TODO: wtf is this?
            book_modified: bool,
        },
        LoadPlayer {
            gist: datamodel::PlayerGist,
            coins: usize,
            safe_chat: bool,
            egg_timer_minutes: usize,
            // unix timestamp in millis, but rounded to the second
            penguin_standard_time: usize,
            // age in days since registration
            age: usize,
            minutes_played: usize,
            membership_days_remain: usize,
            // UTC-N , positive N. For America/Vancouver PDT, it's N=7
            server_time_offset: usize,
            opened_playercard: bool,
            map_category: datamodel::MapCategory,
            new_player_status: datamodel::NewPlayerStatus,
        },
        GetInventory {
            items: Vec<datamodel::ItemId>,
        },
        // TODO:
        GetBuddies {
            // buddies: Vec<datamodel::Buddy>,
        },
        GetIgnoreList {},
        //TODO
        GetPlayerStamps {
            player_id: datamodel::PlayerId,
        },
        //TODO
        QueryPlayerAwards {
            player_id: datamodel::PlayerId,
        },
        // TODO
        GetMail {},
        GetLastRevision(String),
        StartMailEngine {
            unread_mail_count: usize,
            mail_count: usize,
        },
        GetEPFPoints {
            career_medals: usize,
            agent_medals: usize,
        },
        // TODO
        GetFieldOPStatus {},
        // TODO
        GetEPFAgentStatus {},
        JoinRoom {
            room_id: datamodel::RoomId,
            players: Vec<datamodel::PlayerGist>,
        },
        AddedPlayer {
            player: datamodel::PlayerGist,
        },
        // TODO:
        GetWaddlePopulation {},
        GetPlayer {
            player: datamodel::PlayerGist,
        },
        SetPosition {
            player_id: datamodel::PlayerId,
            x: isize,
            y: isize,
        },
        SendMessage{
            player_id: datamodel::PlayerId,
            message: String,
        }
    }

    #[repr(u32)]
    #[derive(Clone, Debug, PartialEq)]
    pub enum Error {
        ConnectionLost = 1,
        TimeOut = 2,
        MultiConnections = 3,
        Disconnect = 4,
        Kick = 5,
        ConnectionNotAllowed = 6,

        NameNotFound = 100,
        PasswordWrong = 101,
        ServerFull = 103,
        OldSaltError = 104,
        PasswordRequired = 130,
        PasswordShort = 131,
        PasswordLong = 132,
        NameRequired = 140,
        NameShort = 141,
        NameLong = 142,
        LoginFlooding = 150,

        PlayerInRoom = 200,
        RoomFull = 210,
        GameFull = 211,
        RoomCapacityRule = 212,
        RoomDoesNotExist = 213,

        AlreadyOwnInventoryItem = 400,
        NotEnoughCoins = 401,
        ItemNotExist = 402,
        MaxFurnitureItems = 403,
        NotEnoughMedals = 405,
        MaxPufflecareItems = 406,
        MaxPufflhatItems = 407,
        AlreadyOwnSuperplayItem = 408,
        MaxCjMats = 409,
        ItemNotAvailable = 410,
        PuffleLimit = 440,
        NameNotAllowed = 441,
        PuffleLimitNm = 442,

        AlreadyOwnIgloo = 500,
        AlreadyOwnFloor = 501,
        AlreadyOwnLocation = 502,

        BanDuration = 601,
        BanAnHour = 602,
        BanForever = 603,
        AutoBan = 610,
        HackingAutoBan = 611,

        GameCheat = 800,
        InvalidRoomIdSpecifiedInJJr = 851,

        AccountNotActivate = 900,
        BuddyLimit = 901,
        PlayTimeUp = 910,
        OutPlayTime = 911,
        Grounded = 913,
        PlayTimeEnding = 914,
        PlayHoursEnding = 915,
        PlayHoursUp = 916,
        PlayHoursHasntStart = 917,
        PlayHoursUpdate = 918,
        SystemReboot = 990,
        NotMember = 999,

        NoDbConnection = 1000,
        NoSocketConnection = 10001,
        Timeout = 10002,
        PasswordSavePrompt = 10003,
        SocketLostConnection = 10004,
        LoadError = 10005,
        MaxIglooFurnitureError = 10006,
        MultipleConnections = 10007,
        ConnectionTimeout = 10008,
        MaxStampbookCoverItems = 10009,
        WebServiceLoadError = 10010,
        WebServiceSendError = 10011,
        ChromeMacLoginError = 10104,

        RedemptionConnectionLost = 20001,
        RedemptionAlreadyHaveItem = 20002,
        RedemptionServerFull = 20103,
        NameRequiredRedemption = 20140,
        NameShortRedemption = 20141,
        PasswordRequiredRedemption = 20130,
        PasswordShortRedemption = 20131,
        RedemptionBookIdNotExist = 20710,
        RedemptionBookAlreadyRedeemed = 20711,
        RedemptionWrongBookAnswer = 20712,
        RedemptionBookTooManyAttempts = 20713,
        RedemptionCodeNotFound = 20720,
        RedemptionCodeAlreadyRedeemed = 20721,
        RedemptionTooManyAttempts = 20722,
        RedemptionCatalogNotAvailable = 20723,
        RedemptionNoExclusiveRedeems = 20724,
        RedemptionCodeGroupRedeemed = 20725,
        RedemptionCodeExpired = 20726,
        RedemptionPufflesMax = 20730,
        RedemptionPuffleInvalid = 21700,
        RedemptionPuffleCodeMax = 21701,
        RedemptionCodeTooShort = 21702,
        RedemptionCodeTooLong = 21703,
        GoldenCodeNotReady = 21704,
        RedemptionPuffleNameEmpty = 21705,
    }
}
