//! A plain TCP reachability probe, used for the "ping" half of profile
//! testing.
//!
//! Deliberately not routed through sing-box: this measures the round trip to
//! the proxy server itself, answers whether the server is up at all, and
//! needs no running core — so it works while disconnected and is cheap enough
//! to sweep a large subscription. It says nothing about whether the proxy
//! works, which is what the URL test is for.

use std::time::{Duration, Instant};

use tokio::net::TcpStream;

/// Milliseconds to complete a TCP handshake with `server:port`, or `None` if
/// the connection could not be made within `timeout` — refused, unresolvable,
/// filtered, or simply too slow. The distinction does not survive into the UI,
/// where all of them mean the same thing: no answer.
pub async fn tcp_ping(server: &str, port: u16, timeout: Duration) -> Option<u32> {
    let started = Instant::now();
    match tokio::time::timeout(timeout, TcpStream::connect((server, port))).await {
        Ok(Ok(_stream)) => Some(started.elapsed().as_millis().min(u32::MAX as u128) as u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_listening_port_answers_with_a_duration() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        let delay = tcp_ping("127.0.0.1", port, Duration::from_secs(2)).await;
        assert!(delay.is_some(), "a listening port must answer");
    }

    #[tokio::test]
    async fn a_closed_port_does_not() {
        // Bound and dropped, so the port is almost certainly free and refuses.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        assert_eq!(
            tcp_ping("127.0.0.1", port, Duration::from_secs(2)).await,
            None
        );
    }

    #[tokio::test]
    async fn a_name_that_does_not_resolve_is_a_failure_not_a_hang() {
        assert_eq!(
            tcp_ping("no-such-host.invalid", 443, Duration::from_secs(5)).await,
            None
        );
    }
}
