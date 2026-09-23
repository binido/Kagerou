use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::app_state::AppState;
use crate::net::geo;
use crate::net::updates;
use crate::singbox;
use crate::storage::models::{
    NewProfileGroup, NewRoutingRule, Profile, ProfileGroup, RoutingPreset, RoutingRule, Settings,
    Source, TestResult,
};
use crate::storage::{groups, profiles, routing, settings, sources};
use crate::subscription::Unsupported;
use crate::usecase::connection;
use crate::usecase::error::{AppError, ErrorCode};
use crate::usecase::import::{self, Imported};
use crate::usecase::subscriptions;
use crate::usecase::testing;

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

// ---------------------------------------------------------------------
// Full state snapshot (replaces mock-data.ts's initial* exports)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub connected: bool,
    /// Unix milliseconds, or `None` while disconnected.
    pub connected_since: Option<u64>,
    pub active_profile_id: String,
    pub profiles: Vec<Profile>,
    pub profile_groups: Vec<ProfileGroup>,
    pub sources: Vec<Source>,
    pub routing_presets: Vec<RoutingPreset>,
    pub routing_rules: Vec<RoutingRule>,
    pub settings: Settings,
}

#[tauri::command]
pub fn get_app_state(state: State<AppState>) -> Result<AppSnapshot, AppError> {
    // The connection-changed event fires before the WebView is listening
    // when the startup auto-connect wins the race (and on a mid-session
    // reload), so the snapshot carries the supervisor's own status as the
    // baseline and the events take over from there.
    let connected = state.is_connected();
    Ok(AppSnapshot {
        connected,
        connected_since: connected
            .then(|| connection::connected_since_millis(&state))
            .flatten(),
        active_profile_id: settings::get_active_profile_id(&state.db)?.unwrap_or_default(),
        profiles: profiles::list_all(&state.db)?,
        profile_groups: groups::list_all(&state.db)?,
        sources: sources::list_all(&state.db)?,
        routing_presets: routing::list_presets(&state.db)?,
        routing_rules: routing::list_rules(&state.db)?,
        settings: settings::get(&state.db)?,
    })
}

// ---------------------------------------------------------------------
// Connection lifecycle
// ---------------------------------------------------------------------

#[tauri::command]
pub async fn connect(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    connection::connect(&app, state.inner()).await
}

#[tauri::command]
pub async fn disconnect(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    connection::disconnect(&app, state.inner())
}

/// A cold sing-box takes a moment to listen. Four attempts two seconds apart
/// covers that without leaving the location spinning for long if the tunnel is
/// genuinely dead.
const GEO_LOOKUP_ATTEMPTS: u32 = 4;
const GEO_LOOKUP_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Asks a public service, through the running tunnel, where the exit node
/// appears to be. `None` - rather than an error - when there is nothing to
/// ask through or the user has turned the lookup off: neither is a failure,
/// and the UI falls back to the profile's own flag in both cases.
#[tauri::command]
pub async fn lookup_exit_location(
    state: State<'_, AppState>,
) -> Result<Option<geo::ExitLocation>, AppError> {
    if !settings::get(&state.db)?.geo_lookup {
        return Ok(None);
    }
    if !state.is_connected() {
        return Ok(None);
    }

    let socks = format!("127.0.0.1:{}", state.paths.mixed_listen_port);
    let client = geo::GeoClient::through_socks(&socks)?;

    // `connect` announces the connection as soon as the process is spawned,
    // so this runs while sing-box is still opening its inbound and the first
    // attempt is usually refused. Without the retry the dashboard silently
    // keeps the profile's flag until someone presses refresh by hand.
    let mut attempts = 0;
    loop {
        attempts += 1;
        match client.lookup(Duration::from_secs(8)).await {
            Ok(location) => return Ok(Some(location)),
            Err(error) if attempts < GEO_LOOKUP_ATTEMPTS && geo::is_transient(&error) => {
                tokio::time::sleep(GEO_LOOKUP_RETRY_DELAY).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

// ---------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------

#[tauri::command]
pub async fn select_profile(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    profiles::select_profile(&state.db, &id)?;
    settings::set_active_profile_id(&state.db, Some(&id))?;
    // The recent list and which entry is greyed out both just changed.
    crate::tray::refresh(&app, state.clash_client().is_some());

    // Hot-switch the running sing-box instance without a restart when
    // already connected, instead of leaving it pointed at the old outbound
    // until the next connect().
    if let Some(clash) = state.clash_client() {
        let _ = clash.select_outbound("proxy", &id).await;
    }
    Ok(())
}

#[tauri::command]
pub fn rename_profile(id: String, name: String, state: State<AppState>) -> Result<(), AppError> {
    profiles::rename(&state.db, &id, &name).map_err(AppError::from)
}

#[tauri::command]
pub fn delete_profile(id: String, state: State<AppState>) -> Result<(), AppError> {
    profiles::delete(&state.db, &id).map_err(AppError::from)
}

/// Tests every profile in `group_id`, or the whole app when it is `None`,
/// reporting each result as it lands.
///
/// The run lives in the backend rather than the frontend firing one command
/// per profile: aiming a test means pointing the test core's selector at one
/// profile, so they cannot overlap, and a sequence of hundreds needs somewhere
/// to report progress from and something to stop it with.
#[tauri::command]
pub async fn start_group_test(
    group_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, AppError> {
    let profile_ids: Vec<String> = match &group_id {
        Some(id) => groups::get(&state.db, id)?.profile_ids,
        None => profiles::list_all(&state.db)?
            .into_iter()
            .map(|p| p.id)
            .collect(),
    };
    if profile_ids.is_empty() {
        return Ok(0);
    }

    let total = profile_ids.len();
    let cancel = testing::begin_run(&state.test_core)?;
    let run_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = run_app.state::<AppState>();
        let (db, paths, core) = (&state.db, &state.paths, &state.test_core);
        testing::run_group(
            db,
            core,
            &run_app,
            cancel,
            profile_ids,
            |profile_id| async move {
                match testing::ensure_running(db, paths, core).await {
                    Ok(clash) => testing::measure_profile(
                        db,
                        paths.test_mixed_listen_port,
                        &clash,
                        &profile_id,
                    )
                    .await
                    .map(testing::Measured::Outcome)
                    .unwrap_or(testing::Measured::CoreUnavailable),
                    Err(_) => testing::Measured::CoreUnavailable,
                }
            },
        )
        .await;
    });

    Ok(total)
}

#[tauri::command]
pub fn cancel_group_test(state: State<AppState>) -> Result<(), AppError> {
    state.test_core.cancel_run();
    Ok(())
}

#[tauri::command]
pub fn clear_test_results(group_id: String, state: State<AppState>) -> Result<(), AppError> {
    profiles::clear_test_results(&state.db, &group_id).map_err(AppError::from)
}

#[tauri::command]
pub fn delete_unavailable_profiles(
    group_id: String,
    state: State<AppState>,
) -> Result<usize, AppError> {
    // The active profile is skipped at the command layer: storage stays
    // ignorant of settings, and generate()'s silent fallback to the first
    // profile never gets a chance to happen.
    let active = settings::get_active_profile_id(&state.db)?;
    profiles::delete_unavailable(&state.db, &group_id, active.as_deref()).map_err(AppError::from)
}

#[tauri::command]
pub fn move_profile_to_group(
    profile_id: String,
    target_group_id: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    profiles::move_to_group(&state.db, &profile_id, &target_group_id).map_err(AppError::from)
}

#[tauri::command]
pub fn move_profile(id: String, direction: String, state: State<AppState>) -> Result<(), AppError> {
    Ok(profiles::move_within_group(
        &state.db,
        &id,
        direction.parse()?,
    )?)
}

#[tauri::command]
pub fn reorder_profiles(
    from_id: String,
    to_id: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    Ok(profiles::move_before(&state.db, &from_id, &to_id)?)
}

#[tauri::command]
pub async fn run_profile_test(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<TestResult, AppError> {
    let clash = testing::ensure_running(&state.db, &state.paths, &state.test_core).await?;
    let outcome = testing::measure_profile(
        &state.db,
        state.paths.test_mixed_listen_port,
        &clash,
        &profile_id,
    )
    .await?;
    let _ = profiles::set_test_outcome(&state.db, &profile_id, outcome);
    Ok(outcome.into())
}

// ---------------------------------------------------------------------
// Profile groups
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_profile_group_open(
    id: String,
    open: bool,
    state: State<AppState>,
) -> Result<(), AppError> {
    groups::set_open(&state.db, &id, open).map_err(AppError::from)
}

#[tauri::command]
pub fn add_profile_group(label: String, state: State<AppState>) -> Result<String, AppError> {
    let id = new_id("group");
    groups::insert(
        &state.db,
        &NewProfileGroup {
            id: id.clone(),
            label,
            kind: "custom".to_string(),
            source_id: None,
        },
    )?;
    Ok(id)
}

#[tauri::command]
pub fn rename_profile_group(
    id: String,
    label: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    groups::rename(&state.db, &id, &label).map_err(AppError::from)
}

// ---------------------------------------------------------------------
// Subscription sources
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSourceInput {
    pub name: Option<String>,
    pub value: Option<String>,
}

#[tauri::command]
pub fn update_source(
    id: String,
    patch: UpdateSourceInput,
    state: State<AppState>,
) -> Result<(), AppError> {
    subscriptions::update_source(
        &state.db,
        &id,
        patch.name.as_deref(),
        patch.value.as_deref(),
    )
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn refresh_source(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Unsupported>, AppError> {
    subscriptions::refresh(&state.db, &id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn import_from_text(
    text: String,
    state: State<'_, AppState>,
) -> Result<Imported, AppError> {
    subscriptions::import_text(&state.db, &text)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub fn delete_subscription(
    group_id: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), AppError> {
    let connected = state.is_connected();
    let active = settings::get_active_profile_id(&state.db)?;
    import::remove_subscription(&state.db, &group_id, active.as_deref(), connected)?;
    // The recent list in the tray may have just lost entries.
    crate::tray::refresh(&app, connected);
    Ok(())
}

// ---------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_preset(id: String, enabled: bool, state: State<AppState>) -> Result<(), AppError> {
    routing::set_preset(&state.db, &id, enabled).map_err(AppError::from)
}

#[tauri::command]
pub fn select_rule(id: String, state: State<AppState>) -> Result<(), AppError> {
    routing::select_rule(&state.db, &id).map_err(AppError::from)
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RulePatchInput {
    #[serde(rename = "match")]
    pub match_value: Option<String>,
    pub outbound: Option<String>,
}

#[tauri::command]
pub fn update_rule(
    id: String,
    patch: RulePatchInput,
    state: State<AppState>,
) -> Result<(), AppError> {
    routing::update_rule(
        &state.db,
        &id,
        &routing::RulePatch {
            match_value: patch.match_value.as_deref(),
            outbound: patch.outbound.as_deref(),
        },
    )
    .map_err(AppError::from)
}

#[tauri::command]
pub fn add_routing_rule(
    match_value: String,
    outbound: String,
    state: State<AppState>,
) -> Result<String, AppError> {
    let id = new_id("rule");
    routing::insert_rule(
        &state.db,
        &NewRoutingRule {
            id: id.clone(),
            match_value,
            outbound,
        },
    )?;
    Ok(id)
}

/// Runs the generator's own classifier, so the hint under the match field
/// and the rule that ends up in the config can never disagree.
#[tauri::command]
pub fn analyze_rule_match(value: String) -> singbox::match_spec::MatchAnalysis {
    singbox::match_spec::analyze(&value)
}

#[tauri::command]
pub fn delete_routing_rule(id: String, state: State<AppState>) -> Result<(), AppError> {
    routing::delete_rule(&state.db, &id).map_err(AppError::from)
}

// ---------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_theme(theme_id: String, state: State<AppState>) -> Result<(), AppError> {
    settings::update(
        &state.db,
        &settings::SettingsPatch {
            theme: Some(theme_id),
            ..Default::default()
        },
    )
    .map_err(AppError::from)
}

/// Silent by design: a failed check is not something to surface, and before
/// the first release GitHub answers 404, which simply means "nothing newer".
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Option<updates::UpdateInfo> {
    updates::check(&app.package_info().version).await
}

/// Makes the OS launch-at-login registration agree with `enabled`. The DB is
/// the source of truth: a failure here leaves the two briefly out of sync and
/// the reconcile at startup or the next toggle converges on the DB. No-op in
/// dev - `tauri dev` would otherwise register the debug binary as a real
/// login item.
pub fn apply_startup_flag(app: &AppHandle, enabled: bool) -> Result<(), AppError> {
    if tauri::is_dev() {
        return Ok(());
    }
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart
            .enable()
            .map_err(|e| AppError::new(ErrorCode::SystemSetting, e))
    } else {
        autostart
            .disable()
            .map_err(|e| AppError::new(ErrorCode::SystemSetting, e))
    }
}

#[tauri::command]
pub fn update_settings(
    patch: settings::SettingsPatch,
    state: State<AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let startup = patch.startup;
    settings::update(&state.db, &patch)?;
    if let Some(startup) = startup {
        apply_startup_flag(&app, startup)?;
    }
    Ok(())
}
