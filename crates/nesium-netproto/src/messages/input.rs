use serde::{Deserialize, Serialize};

/// Client sends a batch of inputs to the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct InputBatch {
    /// Starting frame number for this batch.
    pub start_frame: u32,
    /// Sequential inputs for consecutive frames (one u16 per frame).
    pub buttons: Vec<u16>,
}

/// Server broadcasts confirmation of inputs for a specific player.
#[derive(Serialize, Deserialize, Debug)]
pub struct RelayInputs {
    /// The player index who generated these inputs.
    pub player_index: u8,
    /// Starting frame number.
    pub base_frame: u32,
    /// Inputs (one u16 per frame).
    pub buttons: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputAck {
    pub last_server_frame: u32,
}
