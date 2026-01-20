use crate::{
    constants::{HEADER_LEN, TCP_LEN_PREFIX},
    error::ProtoError,
    header::Header,
    limits::{MAX_TCP_FRAME, max_payload_for},
    messages::Message,
    packet::PacketView,
};

/// Encode a message that implements the [`Message`] trait into a TCP frame.
///
/// This is the primary encoding API. The header is constructed automatically
/// from the message type's `msg_id()` method, and the payload limit is
/// selected based on the message type.
///
/// # Example
/// ```ignore
/// use nesium_netproto::codec::encode_message;
/// use nesium_netproto::messages::session::Hello;
///
/// let msg = Hello { client_nonce: 123, ... };
/// let frame = encode_message(&msg)?;
/// ```
pub fn encode_message<T: Message>(payload: &T) -> Result<Vec<u8>, ProtoError> {
    let msg_id = T::msg_id();
    let max_payload = max_payload_for(msg_id);

    let payload_bytes = postcard::to_stdvec(payload)?;
    if payload_bytes.len() > max_payload {
        return Err(ProtoError::PayloadTooLarge(payload_bytes.len()));
    }

    let mut header = Header::new(msg_id);
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

pub fn try_decode_tcp_frames<'a>(
    in_buf: &'a [u8],
) -> Result<(Vec<PacketView<'a>>, usize), ProtoError> {
    let mut frames = Vec::new();
    let mut offset = 0usize;

    loop {
        if in_buf.len().saturating_sub(offset) < TCP_LEN_PREFIX {
            break;
        }
        let len_bytes = &in_buf[offset..offset + TCP_LEN_PREFIX];
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

        let frame = &in_buf[offset + TCP_LEN_PREFIX..offset + total_needed];
        let (h, payload) = Header::decode(frame)?;

        frames.push(PacketView::new(h, payload));
        offset += total_needed;
    }

    Ok((frames, offset))
}
