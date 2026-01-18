use crate::{header::Header, msg_id::MsgId};

#[derive(Debug, Clone, Copy)]
pub struct PacketView<'a> {
    pub header: Header,
    pub payload: &'a [u8],
}

impl<'a> PacketView<'a> {
    pub fn new(header: Header, payload: &'a [u8]) -> Self {
        Self { header, payload }
    }

    pub fn msg_id(&self) -> MsgId {
        self.header.msg_id
    }
}
