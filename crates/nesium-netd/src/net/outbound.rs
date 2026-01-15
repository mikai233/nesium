use bytes::Bytes;
use futures_util::{Sink, SinkExt};
use nesium_netproto::codec_tcp::encode_tcp_frame_auto;
use nesium_netproto::error::ProtoError;
use nesium_netproto::header::Header;
use nesium_netproto::msg_id::MsgId;
use tokio::sync::mpsc;

/// Outbound channel sender type.
/// Data must already be framed for TCP (length prefix + header + payload).
pub type OutboundTx = mpsc::Sender<bytes::Bytes>;

/// Spawn a writer task that writes framed bytes to the TCP stream.
///
/// Current behavior:
/// - Exits when the channel is closed.
/// - Returns an error if socket write fails.
pub fn spawn_tcp_writer<S>(
    mut write: S,
    mut rx: mpsc::Receiver<bytes::Bytes>,
) -> tokio::task::JoinHandle<anyhow::Result<()>>
where
    S: Sink<bytes::Bytes, Error = std::io::Error> + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            write.send(frame).await?;
        }
        Ok(())
    })
}

/// Encode a message into a single TCP frame and send it to the outbound queue.
///
/// The payload size limit is automatically selected based on the message type:
/// - Data messages (ROM, snapshots, etc.): up to 2 MB
/// - Control messages (handshake, inputs, etc.): up to 4 KB
///
/// This is the recommended function for sending messages.
pub async fn send_msg_tcp<T: serde::Serialize>(
    tx: &OutboundTx,
    header: Header,
    msg_id: MsgId,
    payload: &T,
) -> Result<(), ProtoError> {
    let frame = encode_tcp_frame_auto(header, msg_id, payload)?;
    // The channel carries owned bytes; convert Vec<u8> -> Bytes.
    let bytes = Bytes::from(frame);
    // If receiver is gone, treat it as "connection closed".
    // Map it to a protocol error type you already have.
    tx.send(bytes)
        .await
        .map_err(|_| ProtoError::LengthMismatch)?;
    Ok(())
}

/// Send already-framed bytes (TCP) to the outbound queue.
pub async fn send_bytes(tx: &OutboundTx, frame: Bytes) -> anyhow::Result<()> {
    tx.send(frame).await?;
    Ok(())
}
