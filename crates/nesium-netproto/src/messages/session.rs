use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum TransportKind {
    Tcp,
    Udp,
    Kcp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub client_nonce: u32,
    pub transport: TransportKind,
    pub proto_min: u8,
    pub proto_max: u8,
    pub rom_hash: [u8; 16],
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Welcome {
    pub server_nonce: u32,
    pub assigned_client_id: u32,
    pub room_id: u32,
    pub tick_hz: u16,
    pub input_delay_frames: u8,
    pub max_payload: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinRoom {
    pub room_code: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinAck {
    pub ok: bool,
    pub player_index: u8,
    pub start_frame: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Leave {
    pub reason_code: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMsg {
    pub code: u16,
    pub message: u16,
}
