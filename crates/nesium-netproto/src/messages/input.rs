use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputBatch {
    pub base_frame: u32,
    pub buttons: [u16; 8],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelayInputs {
    pub player_id: u8,
    pub base_frame: u32,
    pub buttons: [u16; 16],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputAck {
    pub last_server_frame: u32,
}
