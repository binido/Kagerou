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
#[derive(Clone)]
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

    /// Runs sing-box's built-in latency probe for one outbound (`GET
    /// /proxies/{name}/delay`), used to power the "test" action on a
    /// profile. Returns the round-trip time in milliseconds.
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
mod tests;
