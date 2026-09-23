use std::time::Duration;

use thiserror::Error;

use super::provider::provider_info;
use crate::storage::models::ProviderInfo;

/// Long enough for a slow provider, short enough that a dead URL does not
/// leave the button spinning.
const TIMEOUT: Duration = Duration::from_secs(15);

/// Must match `VERSION` in `scripts/fetch-singbox.mjs`.
const SINGBOX_VERSION: &str = "1.14.0";

/// Panels pick the response format from this header. Remnawave's default rule
/// looks for `singbox` without the hyphen, and with no match most panels
/// answer with an Xray JSON array instead of something sing-box can run.
fn user_agent() -> String {
    format!("Kagerou singbox/{SINGBOX_VERSION}")
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("could not fetch the subscription: {0}")]
    Http(#[from] reqwest::Error),

    /// Some panels serve only the clients on their own User-Agent list.
    #[error("the provider refused this client: {0}")]
    Refused(reqwest::StatusCode),
}

pub struct Fetched {
    pub body: String,
    /// The provider's `profile-title` header, still encoded.
    pub title: Option<String>,
    pub provider: ProviderInfo,
}

pub async fn fetch(url: &str) -> Result<Fetched, FetchError> {
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent())
        .timeout(TIMEOUT)
        .send()
        .await?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(FetchError::Refused(status));
    }
    let response = response.error_for_status()?;
    let title = response
        .headers()
        .get("profile-title")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or_default();
    let provider = provider_info(response.headers(), now);
    Ok(Fetched {
        body: response.text().await?,
        title,
        provider,
    })
}

#[cfg(test)]
mod tests;
