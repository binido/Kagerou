use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClashApiError {
    #[error("could not reach the Clash API: {0}")]
    Connection(String),

    #[error("Clash API request timed out")]
    Timeout,

    #[error("Clash API returned HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("could not decode Clash API response: {0}")]
    Decode(String),
}

impl From<reqwest::Error> for ClashApiError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_decode() {
            ClashApiError::Decode(e.to_string())
        } else {
            ClashApiError::Connection(e.to_string())
        }
    }
}
