//! Measures how long a proxy takes to answer, the way a user experiences it:
//! by sending real traffic through the tunnel.
//!
//! sing-box's Clash `/delay` endpoint was doing this job and answering two
//! different questions badly. It times the whole path — the TCP connect, the
//! TLS or REALITY handshake, the proxy's own dial to the target — so its
//! number is several times the round trip and comparable to no other client.
//! Worse, it reports working servers as dead: a hysteria2 node in the
//! subscription this was built against failed it at 5, 10 and 20 second
//! timeouts while answering through the tunnel in 53 ms, five times running.
//!
//! This measures what NekoBox's `speedtest.UrlTest` measures in its RTT mode:
//! two requests over one kept-alive connection, timed from the second request
//! being written to its first response byte. Every handshake is excluded, and
//! a result means the tunnel actually carried a request.

use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    /// The proxy refused or never finished setting up the tunnel.
    Unreachable,
    /// The tunnel opened but the target never answered in time.
    Timeout,
}

/// Opens a tunnel through the SOCKS5 inbound at `socks_addr` to
/// `target_host:target_port`. Only the address forms sing-box's mixed inbound
/// actually replies with are handled; anything else is a protocol error and
/// reported as unreachable.
async fn socks5_connect(
    socks_addr: &str,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, ProbeError> {
    let mut stream = TcpStream::connect(socks_addr)
        .await
        .map_err(|_| ProbeError::Unreachable)?;

    // Greeting: one method offered, "no authentication".
    stream
        .write_all(&[0x05, 0x01, 0x00])
        .await
        .map_err(|_| ProbeError::Unreachable)?;
    let mut greeting = [0u8; 2];
    stream
        .read_exact(&mut greeting)
        .await
        .map_err(|_| ProbeError::Unreachable)?;
    if greeting != [0x05, 0x00] {
        return Err(ProbeError::Unreachable);
    }

    // CONNECT, addressed by name so the proxy resolves it at the far end.
    let host = target_host.as_bytes();
    if host.len() > u8::MAX as usize {
        return Err(ProbeError::Unreachable);
    }
    let mut request = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    request.extend_from_slice(host);
    request.extend_from_slice(&target_port.to_be_bytes());
    stream
        .write_all(&request)
        .await
        .map_err(|_| ProbeError::Unreachable)?;

    let mut head = [0u8; 4];
    stream
        .read_exact(&mut head)
        .await
        .map_err(|_| ProbeError::Unreachable)?;
    if head[0] != 0x05 || head[1] != 0x00 {
        return Err(ProbeError::Unreachable);
    }
    // The bound address comes back in the reply and has to be drained before
    // the tunnel carries our own bytes.
    let bound_len = match head[3] {
        0x01 => 4,
        0x04 => 16,
        0x03 => {
            let mut len = [0u8; 1];
            stream
                .read_exact(&mut len)
                .await
                .map_err(|_| ProbeError::Unreachable)?;
            len[0] as usize
        }
        _ => return Err(ProbeError::Unreachable),
    };
    let mut rest = vec![0u8; bound_len + 2];
    stream
        .read_exact(&mut rest)
        .await
        .map_err(|_| ProbeError::Unreachable)?;
    Ok(stream)
}

/// Round trip of a second request on an already-open connection, in
/// milliseconds. The first request is the warm-up whose cost we are
/// deliberately not counting.
async fn timed_round_trip(
    stream: &mut TcpStream,
    host: &str,
    path: &str,
) -> Result<u32, ProbeError> {
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: keep-alive\r\n\r\n");
    let mut scratch = vec![0u8; 4096];

    for warm_up in [true, false] {
        let started = Instant::now();
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|_| ProbeError::Timeout)?;
        let read = stream
            .read(&mut scratch)
            .await
            .map_err(|_| ProbeError::Timeout)?;
        if read == 0 {
            return Err(ProbeError::Timeout);
        }
        if !warm_up {
            return Ok(started.elapsed().as_millis().min(u32::MAX as u128) as u32);
        }
    }
    unreachable!("the loop returns on its second pass")
}

/// Sends a real request through the proxy and reports the round trip.
pub async fn rtt_through_socks(
    socks_addr: &str,
    target_host: &str,
    target_port: u16,
    path: &str,
    timeout: Duration,
) -> Result<u32, ProbeError> {
    let work = async {
        let mut stream = socks5_connect(socks_addr, target_host, target_port).await?;
        timed_round_trip(&mut stream, target_host, path).await
    };
    tokio::time::timeout(timeout, work)
        .await
        .unwrap_or(Err(ProbeError::Timeout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    /// The smallest SOCKS5 server that will satisfy the probe: accepts the
    /// no-auth greeting, accepts any CONNECT, then answers every request with
    /// a canned response. `replies` caps how many it will answer.
    async fn spawn_socks_mock(replies: usize, bound_atyp: u8) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut greeting = [0u8; 3];
            stream.read_exact(&mut greeting).await.unwrap();
            stream.write_all(&[0x05, 0x00]).await.unwrap();

            let mut head = [0u8; 5];
            stream.read_exact(&mut head).await.unwrap();
            let mut host = vec![0u8; head[4] as usize + 2];
            stream.read_exact(&mut host).await.unwrap();

            let mut reply = vec![0x05, 0x00, 0x00, bound_atyp];
            match bound_atyp {
                0x01 => reply.extend_from_slice(&[0, 0, 0, 0]),
                0x03 => {
                    reply.push(3);
                    reply.extend_from_slice(b"abc");
                }
                _ => reply.extend_from_slice(&[0u8; 16]),
            }
            reply.extend_from_slice(&[0, 0]);
            stream.write_all(&reply).await.unwrap();

            let mut scratch = vec![0u8; 1024];
            for _ in 0..replies {
                if stream.read(&mut scratch).await.unwrap_or(0) == 0 {
                    return;
                }
                let _ = stream
                    .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
                    .await;
            }
        });
        addr
    }

    #[tokio::test]
    async fn a_working_tunnel_reports_a_round_trip() {
        let addr = spawn_socks_mock(2, 0x01).await;
        let rtt = rtt_through_socks(&addr, "example.com", 80, "/", Duration::from_secs(5))
            .await
            .unwrap();
        assert!(rtt < 1000, "a loopback round trip should be immediate");
    }

    /// The bound address in the CONNECT reply comes in three shapes and all of
    /// them have to be drained, or the first byte of the HTTP response is read
    /// as part of the handshake.
    #[tokio::test]
    async fn every_bound_address_form_is_drained() {
        for atyp in [0x01, 0x03, 0x04] {
            let addr = spawn_socks_mock(2, atyp).await;
            assert!(
                rtt_through_socks(&addr, "example.com", 80, "/", Duration::from_secs(5))
                    .await
                    .is_ok(),
                "atyp {atyp:#x} should be handled"
            );
        }
    }

    /// The warm-up is not optional: a proxy that answers once and then hangs
    /// up has not proved it can carry traffic.
    #[tokio::test]
    async fn one_reply_is_not_enough() {
        let addr = spawn_socks_mock(1, 0x01).await;
        assert_eq!(
            rtt_through_socks(&addr, "example.com", 80, "/", Duration::from_secs(2)).await,
            Err(ProbeError::Timeout)
        );
    }

    #[tokio::test]
    async fn nothing_listening_is_unreachable() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        drop(listener);
        assert_eq!(
            rtt_through_socks(&addr, "example.com", 80, "/", Duration::from_secs(2)).await,
            Err(ProbeError::Unreachable)
        );
    }

    #[tokio::test]
    async fn a_refused_connect_is_unreachable() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut greeting = [0u8; 3];
            stream.read_exact(&mut greeting).await.unwrap();
            stream.write_all(&[0x05, 0x00]).await.unwrap();
            let mut head = [0u8; 5];
            stream.read_exact(&mut head).await.unwrap();
            let mut host = vec![0u8; head[4] as usize + 2];
            stream.read_exact(&mut host).await.unwrap();
            // 0x05 = connection refused by the proxy.
            let _ = stream
                .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await;
        });
        assert_eq!(
            rtt_through_socks(&addr, "example.com", 80, "/", Duration::from_secs(2)).await,
            Err(ProbeError::Unreachable)
        );
    }
}
