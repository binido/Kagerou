use std::time::Duration;

use super::error::ClashApiError;
use super::model::{ConnectionsResponse, ProxiesResponse, SelectOutboundBody, VersionInfo};

/// A thin async client for sing-box's embedded Clash-compatible HTTP API
/// (`experimental.clash_api.external_controller`).
///
/// The request deadline is enforced by wrapping every call in
/// `tokio::time::timeout` ourselves rather than relying on reqwest's own
/// `Client::builder().timeout(..)`: with the hyper 1.x backend, a deadline
/// firing while a connection is open-but-idle can surface as a generic
/// "canceled"/"incomplete message" `reqwest::Error` that doesn't reliably
/// report `is_timeout() == true`, which would misclassify a real timeout
/// as a plain connection error.
pub struct ClashApiClient {
    base_url: String,
    http: reqwest::Client,
    timeout: Duration,
}

impl ClashApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_config(base_url, reqwest::Client::new(), Duration::from_secs(5))
    }

    pub fn with_config(
        base_url: impl Into<String>,
        http: reqwest::Client,
        timeout: Duration,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http,
            timeout,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn send(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ClashApiError> {
        match tokio::time::timeout(self.timeout, request.send()).await {
            Ok(result) => result.map_err(ClashApiError::from),
            Err(_elapsed) => Err(ClashApiError::Timeout),
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ClashApiError> {
        let response = self.send(self.http.get(self.url(path))).await?;
        parse_json_response(response).await
    }

    pub async fn get_version(&self) -> Result<VersionInfo, ClashApiError> {
        self.get_json("/version").await
    }

    pub async fn get_proxies(&self) -> Result<ProxiesResponse, ClashApiError> {
        self.get_json("/proxies").await
    }

    pub async fn get_connections(&self) -> Result<ConnectionsResponse, ClashApiError> {
        self.get_json("/connections").await
    }

    /// Switches a `selector`-type proxy group (typically our `proxy` group)
    /// to `target`.
    pub async fn select_outbound(&self, selector: &str, target: &str) -> Result<(), ClashApiError> {
        let request = self
            .http
            .put(self.url(&format!(
                "/proxies/{}",
                percent_encoding::utf8_percent_encode(selector, percent_encoding::NON_ALPHANUMERIC)
            )))
            .json(&SelectOutboundBody { name: target });
        let response = self.send(request).await?;
        ensure_success(response).await?;
        Ok(())
    }

    pub async fn close_connection(&self, id: &str) -> Result<(), ClashApiError> {
        let request = self.http.delete(self.url(&format!(
            "/connections/{}",
            percent_encoding::utf8_percent_encode(id, percent_encoding::NON_ALPHANUMERIC)
        )));
        let response = self.send(request).await?;
        ensure_success(response).await?;
        Ok(())
    }

    pub async fn close_all_connections(&self) -> Result<(), ClashApiError> {
        let response = self
            .send(self.http.delete(self.url("/connections")))
            .await?;
        ensure_success(response).await?;
        Ok(())
    }
}

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response, ClashApiError> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        Err(ClashApiError::Http { status, body })
    }
}

async fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ClashApiError> {
    let response = ensure_success(response).await?;
    let bytes = response.bytes().await?;
    serde_json::from_slice(&bytes).map_err(|e| ClashApiError::Decode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clash_api::test_support::spawn_http_mock;

    #[tokio::test]
    async fn version_happy_path() {
        let body = br#"{"version":"1.8.0","premium":true}"#;
        let base_url = spawn_http_mock(move |_req| http_response(200, "OK", body)).await;
        let client = ClashApiClient::new(base_url);
        let version = client.get_version().await.unwrap();
        assert_eq!(version.version, "1.8.0");
        assert!(version.premium);
    }

    #[tokio::test]
    async fn non_success_status_becomes_an_http_error() {
        let base_url = spawn_http_mock(|_req| http_response(404, "Not Found", b"not found")).await;
        let client = ClashApiClient::new(base_url);
        let err = client.get_version().await.unwrap_err();
        match err {
            ClashApiError::Http { status, body } => {
                assert_eq!(status, 404);
                assert_eq!(body, "not found");
            }
            other => panic!("expected Http error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn malformed_json_body_becomes_a_decode_error() {
        let base_url = spawn_http_mock(|_req| http_response(200, "OK", b"this is not json")).await;
        let client = ClashApiClient::new(base_url);
        let err = client.get_version().await.unwrap_err();
        assert!(matches!(err, ClashApiError::Decode(_)));
    }

    #[tokio::test]
    async fn connection_refused_is_a_connection_error() {
        // Nothing is listening on this port.
        let client = ClashApiClient::new("http://127.0.0.1:1");
        let err = client.get_version().await.unwrap_err();
        assert!(matches!(err, ClashApiError::Connection(_)));
    }

    #[tokio::test]
    async fn a_server_that_never_responds_times_out() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            // Accept and hold the connection open (the accepted socket
            // must stay alive, not be dropped) without ever writing a
            // response, so the client has something to time out on.
            if let Ok((_socket, _)) = listener.accept().await {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
        let client = ClashApiClient::with_config(
            format!("http://{addr}"),
            reqwest::Client::new(),
            Duration::from_millis(200),
        );
        let err = client.get_version().await.unwrap_err();
        assert!(matches!(err, ClashApiError::Timeout));
    }

    #[tokio::test]
    async fn select_outbound_sends_a_put_with_the_target_name() {
        use std::sync::{Arc, Mutex};
        let captured = Arc::new(Mutex::new(String::new()));
        let captured_clone = Arc::clone(&captured);
        let base_url = spawn_http_mock(move |req| {
            *captured_clone.lock().unwrap() = req;
            http_response(204, "No Content", b"")
        })
        .await;
        let client = ClashApiClient::new(base_url);
        client
            .select_outbound("proxy", "profile-tokyo")
            .await
            .unwrap();
        let request = captured.lock().unwrap().clone();
        assert!(
            request.starts_with("PUT /proxies/proxy"),
            "request was: {request}"
        );
        assert!(
            request.contains("profile-tokyo"),
            "request body should carry the target name: {request}"
        );
    }

    #[tokio::test]
    async fn select_outbound_on_an_unknown_selector_is_reported() {
        let base_url =
            spawn_http_mock(|_req| http_response(404, "Not Found", b"proxy group not found")).await;
        let client = ClashApiClient::new(base_url);
        let err = client.select_outbound("proxy", "ghost").await.unwrap_err();
        assert!(matches!(err, ClashApiError::Http { status: 404, .. }));
    }

    fn http_response(status: u16, reason: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        response.extend_from_slice(body);
        response
    }
}
