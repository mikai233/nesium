use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use nesium_netproto::codec::try_decode_tcp_frames;
use nesium_netproto::error::ProtoError;
use nesium_netproto::header::Header;
use nesium_netproto::msg_id::MsgId;

/// A single decoded packet that owns its payload bytes.
/// This is safe to move across tasks/channels.
#[derive(Debug, Clone)]
pub struct PacketOwned {
    pub header: Header,
    pub payload: bytes::Bytes,
}

impl PacketOwned {
    pub fn msg_id(&self) -> MsgId {
        self.header.msg_id
    }
}

/// A small TCP framing helper:
/// - keeps an internal receive buffer (`BytesMut`)
/// - decodes as many frames as possible
/// - returns owned packets + keeps the remaining bytes for the next read
pub struct TcpFramer {
    buf: BytesMut,
}

impl TcpFramer {
    /// Create a framer with an initial buffer capacity.
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(initial_capacity),
        }
    }

    /// Get mutable access to the internal buffer for socket reads.
    ///
    /// Typical usage:
    /// - `framer.buf_mut().reserve(n)`
    /// - `socket.read_buf(framer.buf_mut()).await?`
    pub fn buf_mut(&mut self) -> &mut BytesMut {
        &mut self.buf
    }

    /// Try to decode as many frames as possible from the current buffer.
    ///
    /// On success:
    /// - returns a vector of `PacketOwned`
    /// - consumes the decoded bytes from the internal buffer
    pub fn drain_packets(&mut self) -> Result<Vec<PacketOwned>, ProtoError> {
        // Decode borrowed views from the current buffer.
        // We must immediately copy payload into owned bytes before we advance the buffer.
        let (views, consumed) = try_decode_tcp_frames(&self.buf)?;

        // Convert borrowed payload slices into owned Bytes.
        // Note: We copy here to detach from the buffer lifecycle.
        let mut out = Vec::with_capacity(views.len());

        for v in views {
            // If your netproto returns (Header, MsgId, &[u8]) instead of PacketView:
            // just adjust field access here.
            let payload = Bytes::copy_from_slice(v.payload);

            out.push(PacketOwned {
                header: v.header,
                payload,
            });
        }
        // Drop consumed bytes from the front.
        self.buf.advance(consumed);

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use nesium_netproto::{codec::encode_message, messages::sync::Ping, msg_id::MsgId};

    use super::TcpFramer;

    #[test]
    fn framer_can_decode_one_frame() {
        let mut framer = TcpFramer::new(1024);

        // Build a valid TCP frame using netproto encoder.
        let payload = Ping { t_ms: 123 };

        let bytes = encode_message(&payload).unwrap();

        framer.buf_mut().extend_from_slice(&bytes);

        let packets = framer.drain_packets().unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].msg_id(), MsgId::Ping);
    }
}
