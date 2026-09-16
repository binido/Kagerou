//! Pulling a subscription's body off the network.
//!
//! Separate from the parsers so everything that reads a subscription can be
//! tested against text, and only this file needs a server.

use std::time::Duration;

use thiserror::Error;

/// Long enough for a slow provider, short enough that a dead URL does not
/// leave the button spinning.
const TIMEOUT: Duration = Duration::from_secs(15);

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
