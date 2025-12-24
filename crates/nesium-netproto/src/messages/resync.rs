use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResyncReq {
    pub from_frame: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SnapshotFrag {
    pub snapshot_id: u32,
    pub frag_index: u16,
    pub frag_count: u16,
    pub uncompressed_len: u32,
    pub data: Vec<u8>,
}
