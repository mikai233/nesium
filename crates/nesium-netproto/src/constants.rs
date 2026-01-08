//! Protocol constants for nesium-netproto.
//!
//! For message size limits, see the [`limits`](crate::limits) module.

/// Magic bytes at the beginning of every packet.
/// Used to quickly reject unrelated or corrupted data.
pub const MAGIC: [u8; 2] = *b"NS";

/// Wire-format protocol version.
/// Bump this only for breaking changes to the header layout or message formats.
pub const VERSION: u8 = 1;

/// Fixed header length in bytes (wire format).
/// Changed from 28 to 30 to accommodate u32 payload_len for large messages.
pub const HEADER_LEN: usize = 30;

/// TCP framing prefix length in bytes.
///
/// TCP is a byte stream, so each packet is framed as:
/// `[u32 frame_len_le][Header][Payload]`,
/// where `frame_len_le` is the length of `[Header][Payload]` in bytes.
pub const TCP_LEN_PREFIX: usize = 4;

/// Player index marker used to represent a spectator on the wire.
///
/// This value appears in session messages such as `JoinAck`, `PlayerJoined`, and `RoleChanged`.
pub const SPECTATOR_PLAYER_INDEX: u8 = 0xFF;
