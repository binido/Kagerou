use serde::Serialize;

use crate::net::geo::GeoError;
use crate::singbox::{ConfigError, ProcessError};
use crate::storage::StorageError;
use crate::subscription::fetch::FetchError;
use crate::subscription::SubscriptionError;
use crate::usecase::import::ImportError;

use super::core::CoreError;
use super::subscriptions::SubscriptionsError;

/// The kinds of failure the interface distinguishes. Deliberately coarse:
/// each one has to be worth its own sentence in every locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    /// The database would not answer.
    Storage,
    /// Whatever was asked for is not there any more.
    NotFound,
    /// The request itself was not usable.
    InvalidInput,
    /// The core's config could not be built from what is stored.
    ConfigInvalid,
    /// The core would not start, would not stop, or is not there.
    CoreFailed,
    /// Something on the network did not answer.
    Network,
    /// The subscription answered with something that is not a subscription.
    SubscriptionInvalid,
    /// The provider would not hand the subscription to this app.
    SubscriptionRefused,
    /// The text is not an http(s) link.
    NotASubscriptionUrl,
    /// This group is not a subscription, so it has nothing to refresh.
    NotASubscription,
    /// The VPN in use belongs to what is being deleted.
    ActiveProfileInUse,
    /// A group test is already walking its profiles.
    TestRunInProgress,
    /// The lookup could not reach out through the tunnel.
    LookupFailed,
    /// The operating system refused a setting the app tried to change.
    SystemSetting,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    /// The backend's own wording. Not shown to the user - it is English and
    /// can name a path or a host - but kept so a report has something in it.
    pub detail: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail)
    }
}

impl AppError {
    pub fn new(code: ErrorCode, detail: impl std::fmt::Display) -> Self {
        Self {
            code,
            detail: detail.to_string(),
        }
    }
}

impl From<StorageError> for AppError {
    fn from(error: StorageError) -> Self {
        let code = match error {
            StorageError::NotFound => ErrorCode::NotFound,
            StorageError::InvalidInput(_) => ErrorCode::InvalidInput,
            _ => ErrorCode::Storage,
        };
        Self::new(code, error)
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::new(ErrorCode::ConfigInvalid, error)
    }
}

impl From<ProcessError> for AppError {
    fn from(error: ProcessError) -> Self {
        Self::new(ErrorCode::CoreFailed, error)
    }
}

impl From<CoreError> for AppError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::Storage(e) => e.into(),
            CoreError::Config(e) => e.into(),
            CoreError::Process(e) => e.into(),
            CoreError::WriteConfig { .. } => Self::new(ErrorCode::ConfigInvalid, error),
        }
    }
}

impl From<SubscriptionError> for AppError {
    fn from(error: SubscriptionError) -> Self {
        Self::new(ErrorCode::SubscriptionInvalid, error)
    }
}

impl From<FetchError> for AppError {
    fn from(error: FetchError) -> Self {
        let code = match error {
            FetchError::Refused(_) => ErrorCode::SubscriptionRefused,
            FetchError::Http(_) => ErrorCode::Network,
        };
        Self::new(code, error)
    }
}

impl From<ImportError> for AppError {
    fn from(error: ImportError) -> Self {
        match error {
            ImportError::Storage(e) => e.into(),
            ImportError::Subscription(e) => e.into(),
            ImportError::NotASubscription(_) => Self::new(ErrorCode::NotASubscription, error),
            ImportError::ActiveProfileInUse => Self::new(ErrorCode::ActiveProfileInUse, error),
        }
    }
}

impl From<SubscriptionsError> for AppError {
    fn from(error: SubscriptionsError) -> Self {
        match error {
            SubscriptionsError::Storage(e) => e.into(),
            SubscriptionsError::Import(e) => e.into(),
            SubscriptionsError::Parse(e) => e.into(),
            SubscriptionsError::Fetch(e) => e.into(),
            SubscriptionsError::NoGroup => Self::new(ErrorCode::NotASubscription, error),
            SubscriptionsError::NotASubscriptionUrl => {
                Self::new(ErrorCode::NotASubscriptionUrl, error)
            }
        }
    }
}

impl From<GeoError> for AppError {
    fn from(error: GeoError) -> Self {
        Self::new(ErrorCode::LookupFailed, error)
    }
}

#[cfg(test)]
mod tests;
