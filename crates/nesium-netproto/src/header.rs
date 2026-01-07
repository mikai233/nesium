use crate::{
    constants::{HEADER_LEN, MAGIC, VERSION},
    error::ProtoError,
};

/// Packet header (wire format).
///
/// Encoding rules:
/// - Fixed size: exactly `HEADER_LEN` bytes.
/// - Integer fields are little-endian.
/// - Layout is defined by `encode_into()` / `decode()` offsets below.
///
/// Decode rules (current implementation):
/// - Requires `buf.len() >= HEADER_LEN`.
/// - Requires `buf[0..2] == MAGIC`.
/// - Requires `version == VERSION`.
/// - Requires `buf.len() == HEADER_LEN + payload_len`.
#[derive(Debug, Clone, Copy)]
pub struct Header {
    /// Wire-format version. `decode()` rejects versions != `VERSION`.
    pub version: u8,

    /// Message identifier. `decode()` does not validate it; upper layers should.
    pub msg_id: u8,

    /// Per-packet flags (bitfield). This header implementation only stores it.
    /// Interpretation is up to upper layers (transport/session logic).
    pub flags: u8,

    /// Logical room identifier. This header implementation only stores it.
    pub room_id: u32,

    /// Server-assigned client identifier. 0 can mean "not assigned yet".
    pub client_id: u32,

    /// Sender sequence number. This header implementation only stores it.
    pub seq: u32,

    /// Acknowledgement number for peer sequence numbers. Stored only.
    pub ack: u32,

    /// Acknowledgement bitmask for peer sequence numbers. Stored only.
    pub ack_bits: u32,

    /// Payload length in bytes. `decode()` requires `buf.len() == HEADER_LEN + payload_len`.
    pub payload_len: u32,
}

impl Header {
    /// Header size in bytes for the current wire layout.
    pub const LEN: usize = HEADER_LEN;

    /// Create a header with default values and a specific `msg_id`.
    pub fn new(msg_id: u8) -> Self {
        Self {
            version: VERSION,
            msg_id,
            flags: 0,
            room_id: 0,
            client_id: 0,
            seq: 0,
            ack: 0,
            ack_bits: 0,
            payload_len: 0,
        }
    }

    /// Encode this header into `out` using the current fixed wire layout.
    ///
    /// Offsets (bytes):
    /// - 0..2   magic
    /// - 2      version
    /// - 3      msg_id
    /// - 4      flags
    /// - 5      reserved (always 0)
    /// - 6..10  room_id (u32 LE)
    /// - 10..14 client_id (u32 LE)
    /// - 14..18 seq (u32 LE)
    /// - 18..22 ack (u32 LE)
    /// - 22..26 ack_bits (u32 LE)
    /// - 26..30 payload_len (u32 LE)
    pub fn encode_into(&self, out: &mut [u8; HEADER_LEN]) {
        out[0..2].copy_from_slice(&MAGIC);
        out[2] = self.version;
        out[3] = self.msg_id;
        out[4] = self.flags;
        out[5] = 0; // reserved

        out[6..10].copy_from_slice(&self.room_id.to_le_bytes());
        out[10..14].copy_from_slice(&self.client_id.to_le_bytes());
        out[14..18].copy_from_slice(&self.seq.to_le_bytes());
        out[18..22].copy_from_slice(&self.ack.to_le_bytes());
        out[22..26].copy_from_slice(&self.ack_bits.to_le_bytes());
        out[26..30].copy_from_slice(&self.payload_len.to_le_bytes());
    }

    /// Decode a packet buffer that contains exactly `[Header][Payload]`.
    ///
    /// This function expects `buf` to contain the entire packet:
    /// - If `buf.len() < HEADER_LEN`, returns `TooShort`.
    /// - If magic/version mismatch, returns an error.
    /// - Reads `payload_len` from the header and requires:
    ///   `buf.len() == HEADER_LEN + payload_len`.
    /// - On success, returns `(Header, payload_slice)`.
    pub fn decode(buf: &[u8]) -> Result<(Header, &[u8]), ProtoError> {
        if buf.len() < HEADER_LEN {
            return Err(ProtoError::TooShort);
        }
        if buf[0..2] != MAGIC {
            return Err(ProtoError::BadMagic);
        }

        let version = buf[2];
        if version != VERSION {
            return Err(ProtoError::UnsupportedVersion(version));
        }

        let msg_id = buf[3];
        let flags = buf[4];

        let room_id = read_u32_le(buf, 6)?;
        let client_id = read_u32_le(buf, 10)?;
        let seq = read_u32_le(buf, 14)?;
        let ack = read_u32_le(buf, 18)?;
        let ack_bits = read_u32_le(buf, 22)?;
        let payload_len = read_u32_le(buf, 26)?;

        let payload_len_usize = payload_len as usize;
        if buf.len() != HEADER_LEN + payload_len_usize {
            return Err(ProtoError::LengthMismatch);
        }

        let h = Header {
            version,
            msg_id,
            flags,
            room_id,
            client_id,
            seq,
            ack,
            ack_bits,
            payload_len,
        };

        Ok((h, &buf[HEADER_LEN..]))
    }
}

fn read_u32_le(buf: &[u8], start: usize) -> Result<u32, ProtoError> {
    let bytes: [u8; 4] = buf
        .get(start..start + 4)
        .ok_or(ProtoError::TooShort)?
        .try_into()
        .map_err(|_| ProtoError::TooShort)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u16_le(buf: &[u8], start: usize) -> Result<u16, ProtoError> {
    let bytes: [u8; 2] = buf
        .get(start..start + 2)
        .ok_or(ProtoError::TooShort)?
        .try_into()
        .map_err(|_| ProtoError::TooShort)?;
    Ok(u16::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::constants::{HEADER_LEN, MAGIC};

    #[test]
    fn header_len_is_locked() {
        assert_eq!(Header::LEN, HEADER_LEN);
        assert_eq!(Header::LEN, 28);
    }

    #[test]
    fn header_encode_offsets_are_locked() {
        let mut h = Header::new(0x12);
        h.flags = 0xA5;

        h.room_id = 0x11223344;
        h.client_id = 0x55667788;

        h.seq = 0x01020304;
        h.ack = 0x0A0B0C0D;
        h.ack_bits = 0xF0E0D0C0;

        h.payload_len = 0x3344;

        let mut buf = [0u8; HEADER_LEN];
        h.encode_into(&mut buf);

        assert_eq!(&buf[0..2], &MAGIC);
        assert_eq!(buf[2], h.version);
        assert_eq!(buf[3], h.msg_id);
        assert_eq!(buf[4], h.flags);
        assert_eq!(buf[5], 0);

        assert_eq!(
            u32::from_le_bytes(buf[6..10].try_into().unwrap()),
            h.room_id
        );
        assert_eq!(
            u32::from_le_bytes(buf[10..14].try_into().unwrap()),
            h.client_id
        );
        assert_eq!(u32::from_le_bytes(buf[14..18].try_into().unwrap()), h.seq);
        assert_eq!(u32::from_le_bytes(buf[18..22].try_into().unwrap()), h.ack);
        assert_eq!(
            u32::from_le_bytes(buf[22..26].try_into().unwrap()),
            h.ack_bits
        );
        assert_eq!(
            u32::from_le_bytes(buf[26..30].try_into().unwrap()),
            h.payload_len
        );

        assert_eq!(buf.len(), Header::LEN);
    }

    #[test]
    fn header_decode_requires_exact_total_length() {
        let mut h = Header::new(1);
        h.payload_len = 3;

        let mut packet = vec![0u8; HEADER_LEN + 3];
        let mut hbuf = [0u8; HEADER_LEN];
        h.encode_into(&mut hbuf);

        packet[..HEADER_LEN].copy_from_slice(&hbuf);
        packet[HEADER_LEN..].copy_from_slice(&[1, 2, 3]);

        let (decoded, payload) = Header::decode(&packet).unwrap();
        assert_eq!(decoded.payload_len, 3);
        assert_eq!(payload, &[1, 2, 3]);

        let mut too_long = packet.clone();
        too_long.push(9);
        assert!(Header::decode(&too_long).is_err());

        let too_short = &packet[..packet.len() - 1];
        assert!(Header::decode(too_short).is_err());
    }
}
