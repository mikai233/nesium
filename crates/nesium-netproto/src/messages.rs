use crate::msg_id::MsgId;
use serde::de::DeserializeOwned;

pub mod input;
pub mod resync;
pub mod session;
pub mod sync;

/// A trait for all netplay protocol messages.
///
/// Each message type implements this trait to declare its associated `MsgId`,
/// enabling automatic header construction during encoding.
///
/// Note: This trait is automatically implemented by the `define_protocol!` macro.
/// Do not implement this trait manually.
pub trait Message: serde::Serialize + DeserializeOwned + Send + 'static {
    /// Returns the message identifier for this message type.
    fn msg_id() -> MsgId;
}
