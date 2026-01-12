use strum::FromRepr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum MsgId {
    Hello = 1,
    Welcome = 2,
    JoinRoom = 3,
    JoinAck = 4,
    Leave = 5,
    Error = 6,
    AttachChannel = 7,

    SwitchRole = 10,
    RoleChanged = 11,
    PlayerLeft = 12,
    PlayerJoined = 13,

    InputBatch = 20,
    RelayInputs = 21,
    InputAck = 22,

    Ping = 30,
    Pong = 31,
    SyncHint = 32,

    ResyncReq = 40,
    SnapshotFrag = 41,

    LoadRom = 50,
    RomLoaded = 51,
    StartGame = 52,

    PauseGame = 60,
    PauseSync = 61,
    ResetGame = 62,
    ResetSync = 63,
    RequestState = 64,
    SyncState = 65,
    ProvideState = 66,
    /// Server tells a late joiner to begin catch-up from a snapshot.
    BeginCatchUp = 67,
}
