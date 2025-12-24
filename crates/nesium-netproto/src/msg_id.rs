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

    InputBatch = 20,
    RelayInputs = 21,
    InputAck = 22,

    Ping = 30,
    Pong = 31,
    SyncHint = 32,

    ResyncReq = 40,
    SnapshotFrag = 41,
}
