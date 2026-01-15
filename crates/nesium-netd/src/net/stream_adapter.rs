use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BytesMut};
use futures_util::Stream;
use tokio::io::AsyncRead;
use tokio_tungstenite::tungstenite::{self, Message};
use tracing::warn;

/// Adapts a WebSocket stream (Message-based) to an `AsyncRead` byte stream.
///
/// Reading:
/// - Buffers incoming `Binary` messages and serves them as bytes.
/// - Ignores `Ping`/`Pong`/`Text` frames.
pub struct WebSocketStream<S> {
    inner: S,
    read_buf: BytesMut,
}

impl<S> WebSocketStream<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            read_buf: BytesMut::new(),
        }
    }
}

impl<S> AsyncRead for WebSocketStream<S>
where
    S: Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        loop {
            if !this.read_buf.is_empty() {
                let n = std::cmp::min(buf.remaining(), this.read_buf.len());
                buf.put_slice(&this.read_buf[..n]);
                this.read_buf.advance(n);
                return Poll::Ready(Ok(()));
            }

            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(msg))) => match msg {
                    Message::Binary(data) => {
                        this.read_buf.extend_from_slice(&data);
                    }
                    Message::Ping(_) | Message::Pong(_) => continue,
                    Message::Close(_) => return Poll::Ready(Ok(())),
                    Message::Text(_) => {
                        warn!("Received unexpected text message on WebSocket binary stream");
                        continue;
                    }
                    Message::Frame(_) => continue,
                },
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(io::Error::other(e))),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
