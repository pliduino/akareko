use std::io::{self};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tracing::info;

// A wrapper around any AsyncRead + AsyncWrite that logs everything
pub struct LoggingStream<S>(pub S);

impl<S: AsyncRead + Unpin> AsyncRead for LoggingStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let filled_before = buf.filled().len();
        let poll = Pin::new(&mut self.0).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &poll {
            let filled_after = buf.filled().len();
            let new_data = &buf.filled()[filled_before..filled_after];
            info!("IN: {:?}", new_data);
        }
        poll
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for LoggingStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let res = Pin::new(&mut self.0).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = &res {
            info!("OUT: {:?}", &buf[..*n]);
        }
        res
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
