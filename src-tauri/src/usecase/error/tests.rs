use super::*;

#[test]
fn a_missing_row_is_reported_as_missing_rather_than_as_a_database_problem() {
    let error: AppError = StorageError::NotFound.into();
    assert_eq!(error.code, ErrorCode::NotFound);
}

#[test]
fn a_corrupt_profile_key_reaches_the_window_as_a_config_problem() {
    let error: AppError = CoreError::Config(ConfigError::NoProfiles).into();
    assert_eq!(error.code, ErrorCode::ConfigInvalid);
}

#[test]
fn a_core_that_will_not_start_is_not_confused_with_a_bad_config() {
    let error: AppError = CoreError::Process(ProcessError::AlreadyRunning).into();
    assert_eq!(error.code, ErrorCode::CoreFailed);
}

/// The nested cases are the ones worth pinning: a storage failure that
/// happens to travel inside a subscription error is still a storage failure,
/// and telling the user their subscription is malformed would be a lie.
#[test]
fn a_nested_storage_failure_keeps_its_own_code() {
    let error: AppError = SubscriptionsError::Import(ImportError::Storage(
        StorageError::InvalidInput("blank".into()),
    ))
    .into();
    assert_eq!(error.code, ErrorCode::InvalidInput);
}

#[test]
fn deleting_a_subscription_the_active_vpn_belongs_to_says_exactly_that() {
    let error: AppError = ImportError::ActiveProfileInUse.into();
    assert_eq!(error.code, ErrorCode::ActiveProfileInUse);
}

#[test]
fn the_detail_keeps_the_backend_wording_for_the_console() {
    let error: AppError = StorageError::InvalidInput("label must not be blank".into()).into();
    assert!(error.detail.contains("label must not be blank"));
}

/// The frontend switches on these; a rename here that is not made in
/// `app/src/lib/errors.ts` silently falls back to a generic sentence.
#[test]
fn the_codes_keep_the_names_the_frontend_switches_on() {
    let codes = [
        ErrorCode::Storage,
        ErrorCode::NotFound,
        ErrorCode::InvalidInput,
        ErrorCode::ConfigInvalid,
        ErrorCode::CoreFailed,
        ErrorCode::Network,
        ErrorCode::SubscriptionInvalid,
        ErrorCode::SubscriptionRefused,
        ErrorCode::NotASubscriptionUrl,
        ErrorCode::NotASubscription,
        ErrorCode::ActiveProfileInUse,
        ErrorCode::TestRunInProgress,
        ErrorCode::LookupFailed,
        ErrorCode::SystemSetting,
    ];
    let names: Vec<String> = codes
        .iter()
        .map(|code| {
            serde_json::to_value(code)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        [
            "storage",
            "notFound",
            "invalidInput",
            "configInvalid",
            "coreFailed",
            "network",
            "subscriptionInvalid",
            "subscriptionRefused",
            "notASubscriptionUrl",
            "notASubscription",
            "activeProfileInUse",
            "testRunInProgress",
            "lookupFailed",
            "systemSetting",
        ]
    );
}

#[test]
fn a_provider_refusing_the_app_is_not_reported_as_a_network_failure() {
    let error: AppError = FetchError::Refused(reqwest::StatusCode::FORBIDDEN).into();
    assert_eq!(error.code, ErrorCode::SubscriptionRefused);
}
