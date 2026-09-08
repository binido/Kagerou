//! Answers "where does the internet think I am", the way a user checks it on
//! 2ip: by asking a public service what it sees, over the tunnel.
//!
//! The proxy is the whole point. The same request sent directly reports the
//! user's real country while looking exactly as convincing on screen, so
//! `through_socks` is the only constructor the app uses and the socks address
//! is not optional.

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
/// listening — that refusal is a starting proxy, not a verdict. An HTTP status
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
    /// The only constructor the app uses: every lookup goes through the
    /// running core's mixed inbound. `socks5h`, not `socks5`, so the name is
    /// resolved at the far end like the rest of the tunnel's traffic.
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

    /// Lets the tests point a plain client at a local mock, which is the only
    /// way to exercise the response handling without a live tunnel.
    pub fn with_config(client: reqwest::Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }

    pub async fn lookup(&self, timeout: Duration) -> Result<ExitLocation, GeoError> {
        // Our own timeout rather than reqwest's, for the reason the Clash API
        // client documents: the client-level one does not reliably surface as
        // `is_timeout()` on the hyper 1.x backend.
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
mod tests {
    use super::*;
    use crate::clash_api::test_support::spawn_http_mock;

    fn ok_body(json: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{json}",
            json.len()
        )
        .into_bytes()
    }

    const SHORT: Duration = Duration::from_millis(500);

    #[tokio::test]
    async fn a_successful_lookup_carries_the_city_country_and_address() {
        let url = spawn_http_mock(|_| {
            ok_body(r#"{"ip":"81.2.69.142","success":true,"city":"London","country":"United Kingdom","country_code":"GB"}"#)
        })
        .await;
        let client = GeoClient::with_config(reqwest::Client::new(), url);

        assert_eq!(
            client.lookup(SHORT).await,
            Ok(ExitLocation {
                ip: "81.2.69.142".into(),
                city: "London".into(),
                country: "United Kingdom".into(),
                country_code: "GB".into(),
            })
        );
    }

    #[tokio::test]
    async fn a_refusal_arrives_as_http_200_and_must_not_read_as_an_answer() {
        let url = spawn_http_mock(|_| {
            ok_body(r#"{"success":false,"message":"Reserved range","ip":"127.0.0.1"}"#)
        })
        .await;
        let client = GeoClient::with_config(reqwest::Client::new(), url);

        assert_eq!(
            client.lookup(SHORT).await,
            Err(GeoError::Rejected("Reserved range".into()))
        );
    }

    #[tokio::test]
    async fn a_non_2xx_status_is_not_decoded() {
        let url = spawn_http_mock(|_| {
            b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n".to_vec()
        })
        .await;
        let client = GeoClient::with_config(reqwest::Client::new(), url);

        assert_eq!(client.lookup(SHORT).await, Err(GeoError::Http(429)));
    }

    #[tokio::test]
    async fn a_malformed_body_is_a_decode_error_and_not_a_panic() {
        let url = spawn_http_mock(|_| ok_body("not json at all")).await;
        let client = GeoClient::with_config(reqwest::Client::new(), url);

        assert!(matches!(
            client.lookup(SHORT).await,
            Err(GeoError::Decode(_))
        ));
    }

    #[tokio::test]
    async fn a_refused_connection_is_unreachable() {
        // Port 1 on loopback: nothing is listening, and binding it needs root.
        let client = GeoClient::with_config(reqwest::Client::new(), "http://127.0.0.1:1/");

        assert!(matches!(
            client.lookup(SHORT).await,
            Err(GeoError::Unreachable(_))
        ));
    }

    #[tokio::test]
    async fn a_server_that_never_answers_times_out() {
        let url = spawn_http_mock(|_| {
            std::thread::sleep(Duration::from_secs(2));
            ok_body("{}")
        })
        .await;
        let client = GeoClient::with_config(reqwest::Client::new(), url);

        assert_eq!(client.lookup(SHORT).await, Err(GeoError::Timeout));
    }

    #[test]
    fn only_a_proxy_that_never_answered_is_worth_asking_again() {
        assert!(is_transient(&GeoError::Unreachable("refused".into())));
        assert!(is_transient(&GeoError::Timeout));
        assert!(
            !is_transient(&GeoError::Rejected("Reserved range".into())),
            "the service answered; asking again gets the same answer"
        );
        assert!(
            !is_transient(&GeoError::Http(429)),
            "retrying a rate limit is how a rate limit becomes a ban"
        );
        assert!(!is_transient(&GeoError::Decode("bad json".into())));
    }

    #[test]
    fn a_missing_city_is_an_empty_string_rather_than_a_failed_lookup() {
        let body = serde_json::from_str::<Response>(
            r#"{"success":true,"ip":"1.1.1.1","country":"Australia","country_code":"AU"}"#,
        )
        .expect("partial payloads are still payloads");
        let location = to_location(body).expect("a lookup without a city still located the exit");
        assert_eq!(location.city, "");
        assert_eq!(location.country, "Australia");
    }
}
