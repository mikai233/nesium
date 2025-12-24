use crate::{
    constants::{HEADER_LEN, MAX_UDP_PAYLOAD},
    error::ProtoError,
    header::Header,
    msg_id::MsgId,
    packet::PacketView,
};

pub fn encode_udp<T: serde::Serialize>(
    mut header: Header,
    msg_id: MsgId,
    payload: &T,
    max_payload: usize,
) -> Result<Vec<u8>, ProtoError> {
    let payload_bytes = postcard::to_stdvec(payload)?;
    if payload_bytes.len() > max_payload || payload_bytes.len() > MAX_UDP_PAYLOAD {
        return Err(ProtoError::PayloadTooLarge(payload_bytes.len()));
    }

    header.msg_id = msg_id as u8;
    header.payload_len = payload_bytes.len() as u16;

    let mut out = vec![0u8; HEADER_LEN + payload_bytes.len()];
    let mut hbuf = [0u8; HEADER_LEN];
    header.encode_into(&mut hbuf);
    out[..HEADER_LEN].copy_from_slice(&hbuf);
    out[HEADER_LEN..].copy_from_slice(&payload_bytes);
    Ok(out)
}

pub fn decode_udp<'a>(datagram: &'a [u8]) -> Result<PacketView<'a>, ProtoError> {
    if datagram.len() < HEADER_LEN {
        return Err(ProtoError::TooShort);
    }
    if datagram.len() > HEADER_LEN + MAX_UDP_PAYLOAD {
        return Err(ProtoError::FrameTooLarge(datagram.len()));
    }

    let (h, payload) = Header::decode(datagram)?;
    let msg = MsgId::from_repr(h.msg_id).ok_or(ProtoError::UnknownMsgId(h.msg_id))?;
    Ok(PacketView::new(h, msg, payload))
}
