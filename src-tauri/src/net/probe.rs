use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    /// The tunnel never carried the request: refused, unresolvable, or the
    /// proxy failed to reach the target.
    Unreachable,
    /// It carried it, but nothing came back in time.
    Timeout,
}

/// Sends `url` through the SOCKS5 proxy at `socks_addr` twice and reports the
/// round trip of the second, in milliseconds.
///
/// Twice over one kept-alive connection, so the TCP connect and the TLS or
/// REALITY handshake are paid by the warm-up and excluded from the result.
/// A result at all means the tunnel carried a real request.
pub async fn rtt_through_socks(
    socks_addr: &str,
    url: &str,
    timeout: Duration,
) -> Result<u32, ProbeError> {
    // socks5h, not socks5. The proxy resolves the name at the far end, which
    // is what carrying real traffic through it looks like.
    let proxy = reqwest::Proxy::all(format!("socks5h://{socks_addr}"))
        .map_err(|_| ProbeError::Unreachable)?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        // Keep-alive is the whole mechanism: without a reused connection the
        // second request would pay for its own handshakes and measure the
        // same thing the Clash endpoint does.
        .pool_max_idle_per_host(1)
        .build()
        .map_err(|_| ProbeError::Unreachable)?;

    let warm_up = tokio::time::timeout(timeout, client.get(url).send()).await;
    match warm_up {
        Ok(Ok(_)) => {}
        Ok(Err(_)) => return Err(ProbeError::Unreachable),
        Err(_) => return Err(ProbeError::Timeout),
    }

    let started = Instant::now();
    // `send` resolves once the response head is in, so this is the round trip
    // to the first byte and not the cost of draining a body.
    match tokio::time::timeout(timeout, client.get(url).send()).await {
        Ok(Ok(_)) => Ok(started.elapsed().as_millis().min(u32::MAX as u128) as u32),
        Ok(Err(_)) => Err(ProbeError::Unreachable),
        Err(_) => Err(ProbeError::Timeout),
    }
}

#[cfg(test)]
mod tests;
