use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum SubscriptionError {
    #[error("empty subscription content")]
    Empty,

    #[error("could not recognize subscription format (not a base64 URI list, sing-box JSON, Xray JSON, or Clash YAML)")]
    UnrecognizedFormat,

    #[error("invalid {scheme} URI: {reason}")]
    InvalidUri { scheme: String, reason: String },

    #[error("unsupported URI scheme: {0}")]
    UnsupportedScheme(String),

    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("unsupported transport: {0}")]
    UnsupportedTransport(String),

    #[error("the entry picks among several servers automatically")]
    Balancer,

    #[error("the entry reaches its server through another proxy")]
    Chain,

    #[error("invalid Clash proxy entry at index {index}: {reason}")]
    InvalidClashProxy { index: usize, reason: String },

    #[error("invalid sing-box outbound entry at index {index}: {reason}")]
    InvalidSingBoxOutbound { index: usize, reason: String },

    #[error("invalid Xray config at index {index}: {reason}")]
    InvalidXrayConfig { index: usize, reason: String },
}
