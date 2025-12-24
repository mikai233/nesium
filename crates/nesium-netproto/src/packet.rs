use crate::{header::Header, msg_id::MsgId};

#[derive(Debug, Clone, Copy)]
pub struct PacketView<'a> {
    pub header: Header,
    pub msg_id: MsgId,
    pub payload: &'a [u8],
}

impl<'a> PacketView<'a> {
    pub fn new(header: Header, msg_id: MsgId, payload: &'a [u8]) -> Self {
        Self {
            header,
            msg_id,
            payload,
        }
    }
}
