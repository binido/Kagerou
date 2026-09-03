//! Minimal single-connection HTTP mock server used only by this crate's
//! own tests, so the Clash API client's error handling (bad status,
//! malformed body, refused/dropped connections, timeouts) can be tested
//! without a real sing-box process or an external mocking dependency.
#![cfg(test)]

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Spawns a background task that accepts connections forever, reading one
/// HTTP request per connection (headers + body, if `Content-Length` is
/// present) and handing the raw request text to `handler`, whose return
/// value is written back verbatim as the response before the connection
/// is closed. Returns the mock server's base URL.
pub async fn spawn_http_mock<F>(handler: F) -> String
where
    F: Fn(String) -> Vec<u8> + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock server");
    let addr = listener.local_addr().expect("local addr");
    let handler = Arc::new(handler);

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let handler = Arc::clone(&handler);
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let headers_end = loop {
                    let Ok(n) = socket.read(&mut chunk).await else {
                        return;
                    };
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                        break pos + 4;
                    }
                    if buf.len() > 64 * 1024 {
                        return;
                    }
                };

                let header_text = String::from_utf8_lossy(&buf[..headers_end]).to_string();
                let content_length: usize = header_text
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().to_string())
                    })
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);

                while buf.len() < headers_end + content_length {
                    let Ok(n) = socket.read(&mut chunk).await else {
                        return;
                    };
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }

                let request_text = String::from_utf8_lossy(&buf).to_string();
                let response = handler(request_text);
                let _ = socket.write_all(&response).await;
                let _ = socket.shutdown().await;
            });
        }
    });

    format!("http://{addr}")
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Spawns a background WebSocket mock server. `scripts[i]` is the ordered
/// list of text frames sent to the i-th accepted connection before it is
/// closed; connections beyond `scripts.len()` are closed immediately.
/// Returns the mock server's `ws://` base URL.
pub async fn spawn_ws_mock(scripts: Vec<Vec<String>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws mock server");
    let addr = listener.local_addr().expect("local addr");
    let scripts = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::from(
        scripts,
    )));

    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let script = scripts.lock().unwrap().pop_front().unwrap_or_default();
            tokio::spawn(async move {
                use futures_util::SinkExt;
                let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else {
                    return;
                };
                for message in script {
                    if ws
                        .send(tokio_tungstenite::tungstenite::Message::Text(message))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                let _ = ws.close(None).await;
            });
        }
    });

    format!("ws://{addr}/traffic")
}
