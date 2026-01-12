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
            payload_len: 0,
        }
    }

    /// Encode this header into `out` using the current fixed wire layout.
    ///
    /// Offsets (bytes):
    /// - 0..2   magic
    /// - 2      version
    /// - 3      msg_id
    /// - 4..8   payload_len (u32 LE)
    pub fn encode_into(&self, out: &mut [u8; HEADER_LEN]) {
        out[0..2].copy_from_slice(&MAGIC);
        out[2] = self.version;
        out[3] = self.msg_id;
        out[4..8].copy_from_slice(&self.payload_len.to_le_bytes());
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
        let payload_len = read_u32_le(buf, 4)?;

        let payload_len_usize = payload_len as usize;
        if buf.len() != HEADER_LEN + payload_len_usize {
            return Err(ProtoError::LengthMismatch);
        }

        let h = Header {
            version,
            msg_id,
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

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::constants::{HEADER_LEN, MAGIC};

    #[test]
    fn header_len_is_locked() {
        assert_eq!(Header::LEN, HEADER_LEN);
        assert_eq!(Header::LEN, 8);
    }

    #[test]
    fn header_encode_offsets_are_locked() {
        let mut h = Header::new(0x12);
        h.payload_len = 0x3344;

        let mut buf = [0u8; HEADER_LEN];
        h.encode_into(&mut buf);

        assert_eq!(&buf[0..2], &MAGIC);
        assert_eq!(buf[2], h.version);
        assert_eq!(buf[3], h.msg_id);
        assert_eq!(
            u32::from_le_bytes(buf[4..8].try_into().unwrap()),
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
