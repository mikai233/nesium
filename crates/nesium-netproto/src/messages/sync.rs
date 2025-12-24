use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub t_ms: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Pong {
    pub t_ms: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncHint {
    pub recommended_input_delay: u8,
    pub target_frame: u32,
    pub jitter_budget_ms: u16,
}
