/// Magic bytes at the beginning of every packet.
/// Used to quickly reject unrelated or corrupted data.
pub const MAGIC: [u8; 2] = *b"NS";

/// Wire-format protocol version.
/// Bump this only for breaking changes to the header layout or message formats.
pub const VERSION: u8 = 1;

/// Fixed header length in bytes (wire format).
pub const HEADER_LEN: usize = 28;

/// Maximum payload size allowed for UDP packets (in bytes).
/// Keep this below typical path MTU to reduce fragmentation risk.
pub const MAX_UDP_PAYLOAD: usize = 1200;

/// Maximum size of a single framed TCP packet (header + payload), in bytes.
/// This limit is enforced to avoid unbounded allocations.
pub const MAX_TCP_FRAME: usize = 64 * 1024;

/// Maximum size for "control-plane" TCP packets (header + payload), in bytes.
/// Intended for small handshake/control messages; larger transfers should use
/// dedicated message types and/or chunking.
pub const MAX_TCP_CONTROL_FRAME: usize = 4096;

/// TCP framing prefix length in bytes.
///
/// TCP is a byte stream, so each packet is framed as:
/// `[u16 frame_len_le][Header][Payload]`,
/// where `frame_len_le` is the length of `[Header][Payload]` in bytes.
pub const TCP_LEN_PREFIX: usize = 2;
