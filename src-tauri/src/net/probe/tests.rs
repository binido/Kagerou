use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// The smallest SOCKS5 proxy that will satisfy the probe: accepts the
/// no-auth greeting, accepts any CONNECT, then answers HTTP requests with
/// a canned response. `replies` is a budget shared across every
/// connection, so a budget of one models a server that dies after a single
/// request rather than one that merely drops the pooled connection —
/// reqwest would silently reconnect for the latter.
async fn spawn_socks_mock(replies: usize) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let budget = Arc::new(AtomicUsize::new(replies));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let budget = Arc::clone(&budget);
            tokio::spawn(async move {
                let mut greeting = [0u8; 3];
                if stream.read_exact(&mut greeting).await.is_err() {
                    return;
                }
                let _ = stream.write_all(&[0x05, 0x00]).await;

                let mut head = [0u8; 5];
                if stream.read_exact(&mut head).await.is_err() {
                    return;
                }
                let mut rest = vec![0u8; head[4] as usize + 2];
                let _ = stream.read_exact(&mut rest).await;
                let _ = stream
                    .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                    .await;

                let mut scratch = vec![0u8; 2048];
                loop {
                    match stream.read(&mut scratch).await {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {}
                    }
                    if budget.fetch_sub(1, Ordering::SeqCst) == 0 {
                        budget.fetch_add(1, Ordering::SeqCst);
                        return;
                    }
                    let _ = stream
                        .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
                        .await;
                }
            });
        }
    });
    addr
}

#[tokio::test]
async fn a_working_tunnel_reports_a_round_trip() {
    let addr = spawn_socks_mock(4).await;
    let rtt = rtt_through_socks(&addr, "http://example.com/", Duration::from_secs(5))
        .await
        .unwrap();
    assert!(rtt < 1000, "a loopback round trip should be immediate");
}

/// The warm-up is not optional. A proxy that answers once and hangs up has
/// not shown it can carry traffic, and must not be reported as working.
#[tokio::test]
async fn one_reply_is_not_enough() {
    let addr = spawn_socks_mock(1).await;
    assert!(
        rtt_through_socks(&addr, "http://example.com/", Duration::from_secs(3))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn nothing_listening_is_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    drop(listener);
    assert_eq!(
        rtt_through_socks(&addr, "http://example.com/", Duration::from_secs(3)).await,
        Err(ProbeError::Unreachable)
    );
}

#[tokio::test]
async fn a_proxy_that_refuses_the_connect_is_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut greeting = [0u8; 3];
        let _ = stream.read_exact(&mut greeting).await;
        let _ = stream.write_all(&[0x05, 0x00]).await;
        let mut head = [0u8; 5];
        let _ = stream.read_exact(&mut head).await;
        let mut rest = vec![0u8; head[4] as usize + 2];
        let _ = stream.read_exact(&mut rest).await;
        // 0x05: connection refused by the proxy.
        let _ = stream
            .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await;
    });
    assert_eq!(
        rtt_through_socks(&addr, "http://example.com/", Duration::from_secs(3)).await,
        Err(ProbeError::Unreachable)
    );
}

/// An https target must work too: the settings field takes any URL, and a
/// probe that only spoke plaintext would call every server dead the moment
/// someone typed one.
#[tokio::test]
async fn an_https_url_is_carried_as_a_tunnel_rather_than_spoken_in_the_clear() {
    let addr = spawn_socks_mock(4).await;
    // The mock cannot complete a TLS handshake, so this must fail as
    // unreachable — not succeed by sending plaintext to port 443.
    assert_eq!(
        rtt_through_socks(&addr, "https://example.com/", Duration::from_secs(3)).await,
        Err(ProbeError::Unreachable)
    );
}
