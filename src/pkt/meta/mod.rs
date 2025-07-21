pub type PlayerId = usize;
pub type RoomId = usize;
pub mod client {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        PlayerSetPosition { x: usize, y: usize },
        Heartbeat,
    }
}

pub mod server {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Packet {
        PlayerSetPosition {
            player_id: usize,
            x: usize,
            y: usize,
        },
        Heartbeat,
        Error(Error),
        LoginResponse{
            // hash: String,
            // // TODO: why does it have a pipe?
            // buddies_online: String, 
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

    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerSetPosition {
        pub player_id: usize,
        pub x: usize,
        pub y: usize,
    }
}
