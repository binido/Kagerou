use std::time::Duration;

use thiserror::Error;

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
#[error("could not fetch the subscription: {0}")]
pub struct FetchError(#[from] reqwest::Error);

pub struct Fetched {
    pub body: String,
    /// The provider's `profile-title` header, still encoded.
    pub title: Option<String>,
}

pub async fn fetch(url: &str) -> Result<Fetched, FetchError> {
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent())
        .timeout(TIMEOUT)
        .send()
        .await?
        .error_for_status()?;
    let title = response
        .headers()
        .get("profile-title")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    Ok(Fetched {
        body: response.text().await?,
        title,
    })
}

#[cfg(test)]
mod tests;
