//! Streaming NDJSON response reader for `StreamedListObjects`.
//!
//! OpenFGA returns each matching object as a single JSON object on its own line.
//! This module wraps the raw `reqwest::Response` byte stream in a
//! [`futures::Stream`] that yields decoded [`StreamedListObjectsResponse`] items.
//!
//! Only compiled when the `default-executor` feature is enabled.

#![cfg(feature = "default-executor")]

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Bytes, BytesMut};
use futures::Stream;
use pin_project_lite::pin_project;
use reqwest::Response;

use crate::{
    error::{OpenFgaError, Result},
    models::StreamedListObjectsResponse,
};

// ────────────────────────────────────────────────────────────────────────────
// Helper: parse the next complete NDJSON line from a BytesMut buffer.
// ────────────────────────────────────────────────────────────────────────────

/// Tries to extract and decode the next complete newline-terminated JSON line
/// from `buf`.  Returns:
/// - `Some(Ok(item))` - a complete, valid line was decoded.
/// - `Some(Err(e))` - a complete line was found but could not be decoded.
/// - `None` - no complete line is available yet.
fn try_next_line(buf: &mut BytesMut) -> Option<Result<StreamedListObjectsResponse>> {
    let newline_pos = buf.iter().position(|&b| b == b'\n')?;
    let line_bytes = buf.split_to(newline_pos + 1);
    let line = line_bytes[..newline_pos].trim_ascii();

    if line.is_empty() {
        return None;
    }

    // Attempt to decode as a StreamResult envelope first (server-side error case).
    if let Ok(envelope) =
        serde_json::from_slice::<crate::models::StreamResult<StreamedListObjectsResponse>>(line)
    {
        if let Some(err_status) = envelope.error {
            let msg = err_status
                .message
                .unwrap_or_else(|| "Unknown streaming error".to_string());
            return Some(Err(OpenFgaError::Configuration(format!(
                "Streaming error from server: {}",
                msg
            ))));
        }
        if let Some(result) = envelope.result {
            return Some(Ok(result));
        }
        return None;
    }

    // Fall back to direct decode.
    Some(serde_json::from_slice::<StreamedListObjectsResponse>(line).map_err(OpenFgaError::Json))
}

// ────────────────────────────────────────────────────────────────────────────
// NdJsonStream
// ────────────────────────────────────────────────────────────────────────────

pin_project! {
    /// An async stream of [`StreamedListObjectsResponse`] items decoded from
    /// an NDJSON (`application/x-ndjson`) HTTP response body.
    ///
    /// Each item is either a successfully decoded object string or an
    /// [`OpenFgaError`] if the line is malformed or the server sent an error
    /// envelope.
    pub struct NdJsonStream {
        // The underlying byte stream from reqwest.
        #[pin]
        inner: Pin<Box<dyn futures::Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send>>,
        // Accumulated buffer for incomplete lines.
        buf: BytesMut,
        // Set to true when the underlying stream has ended.
        done: bool,
    }
}

impl NdJsonStream {
    /// Wraps a `reqwest::Response` in an NDJSON stream.
    pub fn new(response: Response) -> Self {
        let inner = response.bytes_stream();
        Self {
            inner: Box::pin(inner),
            buf: BytesMut::with_capacity(4096),
            done: false,
        }
    }
}

impl Stream for NdJsonStream {
    type Item = Result<StreamedListObjectsResponse>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // First drain any complete lines already in the buffer.
            if let Some(item) = try_next_line(this.buf) {
                return Poll::Ready(Some(item));
            }

            // Buffer had no complete line. Try to get more bytes.
            if *this.done {
                // Drain any trailing bytes with no trailing newline.
                let trimmed = this.buf.trim_ascii();
                if !trimmed.is_empty() {
                    let item = serde_json::from_slice::<StreamedListObjectsResponse>(trimmed)
                        .map_err(OpenFgaError::Json);
                    this.buf.clear();
                    return Poll::Ready(Some(item));
                }
                return Poll::Ready(None);
            }

            match this.inner.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    *this.done = true;
                    // Loop to drain buf.
                }
                Poll::Ready(Some(Ok(chunk))) => {
                    this.buf.extend_from_slice(&chunk);
                    // Loop to try parsing.
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(OpenFgaError::Http(e.to_string()))));
                }
            }
        }
    }
}
