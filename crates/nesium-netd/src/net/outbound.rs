use bytes::Bytes;
use nesium_netproto::codec_tcp::encode_tcp_frame;
use nesium_netproto::error::ProtoError;
use nesium_netproto::header::Header;
use nesium_netproto::msg_id::MsgId;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// Outbound channel sender type.
/// Data must already be framed for TCP (length prefix + header + payload).
pub type OutboundTx = mpsc::Sender<bytes::Bytes>;

/// Spawn a writer task that writes framed bytes to the TCP stream.
///
/// Current behavior:
/// - Exits when the channel is closed.
/// - Returns an error if socket write fails.
pub fn spawn_tcp_writer(
    mut write: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::Receiver<bytes::Bytes>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            write.write_all(&frame).await?;
        }
        Ok(())
    })
}

/// Encode a message into a single TCP frame and send it to the outbound queue.
///
/// This is a convenience helper; it keeps encoding policy near the net boundary.
/// Upper layers may also choose to build bytes themselves and call `send_bytes`.
pub async fn send_msg_tcp<T: serde::Serialize>(
    tx: &OutboundTx,
    header: Header,
    msg_id: MsgId,
    payload: &T,
    max_payload: usize,
) -> Result<(), ProtoError> {
    let frame = encode_tcp_frame(header, msg_id, payload, max_payload)?;
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
