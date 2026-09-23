mod error;
pub mod fetch;
mod formats;
pub mod model;
mod uri;
mod xray;

pub use error::SubscriptionError;
pub use formats::{parse_subscription, Parsed, Unsupported};
pub use uri::{parse_uri, to_uri};
