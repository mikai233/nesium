use crate::{
    constants::{HEADER_LEN, TCP_LEN_PREFIX},
    error::ProtoError,
    header::Header,
    limits::{MAX_TCP_FRAME, max_payload_for},
    msg_id::MsgId,
    packet::PacketView,
};

/// Encode a TCP frame with explicit max payload size.
///
/// For most use cases, prefer [`encode_tcp_frame_auto`] which automatically
/// selects the appropriate limit based on message type.
pub fn encode_tcp_frame<T: serde::Serialize>(
    mut header: Header,
    msg_id: MsgId,
    payload: &T,
    max_payload: usize,
) -> Result<Vec<u8>, ProtoError> {
    let payload_bytes = postcard::to_stdvec(payload)?;
    if payload_bytes.len() > max_payload {
        return Err(ProtoError::PayloadTooLarge(payload_bytes.len()));
    }

    header.msg_id = msg_id as u8;
    header.payload_len = payload_bytes.len() as u32;

    let frame_len = HEADER_LEN + payload_bytes.len();
    if frame_len > MAX_TCP_FRAME {
        return Err(ProtoError::FrameTooLarge(frame_len));
    }

    let mut out = Vec::with_capacity(TCP_LEN_PREFIX + frame_len);
    out.extend_from_slice(&(frame_len as u32).to_le_bytes());

    let mut hbuf = [0u8; HEADER_LEN];
    header.encode_into(&mut hbuf);
    out.extend_from_slice(&hbuf);
    out.extend_from_slice(&payload_bytes);
    Ok(out)
}

/// Encode a TCP frame with automatic payload limit selection.
///
/// The limit is chosen based on the message type:
/// - Data messages (ROM, snapshots, etc.): up to 2 MB
/// - Control messages (handshake, inputs, etc.): up to 4 KB
pub fn encode_tcp_frame_auto<T: serde::Serialize>(
    header: Header,
    msg_id: MsgId,
    payload: &T,
) -> Result<Vec<u8>, ProtoError> {
    encode_tcp_frame(header, msg_id, payload, max_payload_for(msg_id))
}

pub fn try_decode_tcp_frames<'a>(
    in_buf: &'a [u8],
) -> Result<(Vec<PacketView<'a>>, usize), ProtoError> {
    let mut frames = Vec::new();
    let mut offset = 0usize;

    loop {
        if in_buf.len().saturating_sub(offset) < TCP_LEN_PREFIX {
            break;
        }
        let len_bytes = &in_buf[offset..offset + 4];
        let frame_len =
            u32::from_le_bytes(len_bytes.try_into().expect("slice length is 4")) as usize;

        if frame_len < HEADER_LEN {
            return Err(ProtoError::LengthMismatch);
        }
        if frame_len > MAX_TCP_FRAME {
            return Err(ProtoError::FrameTooLarge(frame_len));
        }

        let total_needed = TCP_LEN_PREFIX + frame_len;
        if in_buf.len().saturating_sub(offset) < total_needed {
            break;
        }

        let frame = &in_buf[offset + 4..offset + total_needed];
        let (h, payload) = Header::decode(frame)?;
        let msg = MsgId::from_repr(h.msg_id).ok_or(ProtoError::UnknownMsgId(h.msg_id))?;

        frames.push(PacketView::new(h, msg, payload));
        offset += total_needed;
    }

    Ok((frames, offset))
}
