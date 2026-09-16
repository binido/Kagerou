use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// https, no API key, and city-level data in one response. Swapping providers
/// is this constant plus `Response`'s field names.
const LOOKUP_URL: &str = "https://ipwho.is/";

#[derive(Debug, Error, PartialEq)]
pub enum GeoError {
    #[error("the lookup never left the tunnel: {0}")]
    Unreachable(String),

    #[error("the lookup timed out")]
    Timeout,

    #[error("the lookup service answered HTTP {0}")]
    Http(u16),

    #[error("the lookup service refused to answer: {0}")]
    Rejected(String),

    #[error("could not decode the lookup response: {0}")]
    Decode(String),
}

/// Where the exit node appears to be, plus the address the internet sees.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitLocation {
    pub ip: String,
    pub city: String,
    pub country: String,
    /// ISO 3166-1 alpha-2, so the frontend can keep drawing its flag.
    pub country_code: String,
}

/// The provider's own shape. `success: false` comes back with HTTP 200 and a
/// `message`, so the status code alone does not tell us whether we have an
/// answer.
#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    ip: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    country_code: String,
}

fn to_location(body: Response) -> Result<ExitLocation, GeoError> {
    if !body.success {
        return Err(GeoError::Rejected(
            body.message.unwrap_or_else(|| "no reason given".into()),
        ));
    }
    Ok(ExitLocation {
        ip: body.ip,
        city: body.city,
        country: body.country,
        country_code: body.country_code,
    })
}

/// Whether another attempt could plausibly answer differently. The core is
/// started and the connection announced in the same breath, so the first
/// lookup after connecting routinely arrives before sing-box has its inbound
/// listening - that refusal is a starting proxy, not a verdict. An HTTP status
/// or a refusal from the service itself is an answer, and repeating the
/// request would only ask the same question again.
pub fn is_transient(error: &GeoError) -> bool {
    matches!(error, GeoError::Unreachable(_) | GeoError::Timeout)
}

pub struct GeoClient {
    client: reqwest::Client,
    url: String,
}

impl GeoClient {
    /// Builds a client whose every request goes through the running core's
    /// mixed inbound.
    ///
    /// The only constructor the app uses. The same request sent directly
    /// reports the user's real country while looking exactly as convincing on
    /// screen, which is why the socks address is not optional here.
    /// `socks5h`, not `socks5`, so the name is resolved at the far end like
    /// the rest of the tunnel's traffic.
    pub fn through_socks(socks_addr: &str) -> Result<Self, GeoError> {
        let proxy = reqwest::Proxy::all(format!("socks5h://{socks_addr}"))
            .map_err(|e| GeoError::Unreachable(e.to_string()))?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .map_err(|e| GeoError::Unreachable(e.to_string()))?;
        Ok(Self {
            client,
            url: LOOKUP_URL.to_string(),
        })
    }

    /// Points a plain client at a given URL. Lets the tests exercise the
    /// response handling without a live tunnel, which nothing else can do.
    pub fn with_config(client: reqwest::Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }

    pub async fn lookup(&self, timeout: Duration) -> Result<ExitLocation, GeoError> {
        // Our own timeout rather than reqwest's. The client-level one does
        // not reliably surface as `is_timeout()` on the hyper 1.x backend.
        let response = tokio::time::timeout(timeout, self.client.get(&self.url).send())
            .await
            .map_err(|_| GeoError::Timeout)?
            .map_err(|e| GeoError::Unreachable(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(GeoError::Http(status.as_u16()));
        }

        let body = tokio::time::timeout(timeout, response.json::<Response>())
            .await
            .map_err(|_| GeoError::Timeout)?
            .map_err(|e| GeoError::Decode(e.to_string()))?;
        to_location(body)
    }
}

#[cfg(test)]
mod tests;
