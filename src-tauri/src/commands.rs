use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::app_state::AppState;
use crate::clash_api::{self, ClashApiClient};
use crate::singbox;
use crate::storage::models::{
    NewProfile, NewProfileGroup, NewRoutingRule, NewSource, Profile, ProfileGroup, Protocol,
    RoutingPreset, RoutingRule, Settings, Source, TestResult, Tone,
};
use crate::storage::{groups, profiles, routing, settings, sources};
use crate::subscription;

fn to_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

/// Best-effort protocol detection from a raw connection URI, used when
/// storing a profile: a successful parse gives the real protocol, and an
/// unparseable-but-plausible-looking key still gets a scheme-based guess
/// rather than rejecting the add outright (mirrors the frontend's existing
/// leniency). Note: `Protocol` has no `Tuic` variant yet (tracked in
/// CLAUDE.md's stage 3 note — the frontend type needs it added first), so
/// a tuic:// key is stored as VLESS for now, same as the old mock did for
/// any unrecognized scheme.
fn detect_protocol(key: &str) -> Protocol {
    if let Ok(parsed) = subscription::parse_uri(key) {
        return match parsed.protocol_label() {
            "VMess" => Protocol::VMess,
            "Trojan" => Protocol::Trojan,
            "Shadowsocks" => Protocol::Shadowsocks,
            "Hysteria2" => Protocol::Hysteria2,
            _ => Protocol::VLESS,
        };
    }
    match key
        .trim()
        .split("://")
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "vmess" => Protocol::VMess,
        "trojan" => Protocol::Trojan,
        "ss" => Protocol::Shadowsocks,
        "hysteria2" | "hy2" => Protocol::Hysteria2,
        _ => Protocol::VLESS,
    }
}

// ---------------------------------------------------------------------
// Full state snapshot (replaces mock-data.ts's initial* exports)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub active_profile_id: String,
    pub profiles: Vec<Profile>,
    pub profile_groups: Vec<ProfileGroup>,
    pub sources: Vec<Source>,
    pub routing_presets: Vec<RoutingPreset>,
    pub routing_rules: Vec<RoutingRule>,
    pub settings: Settings,
}

#[tauri::command]
pub fn get_app_state(state: State<AppState>) -> Result<AppSnapshot, String> {
    Ok(AppSnapshot {
        active_profile_id: settings::get_active_profile_id(&state.db)
            .map_err(to_err)?
            .unwrap_or_default(),
        profiles: profiles::list_all(&state.db).map_err(to_err)?,
        profile_groups: groups::list_all(&state.db).map_err(to_err)?,
        sources: sources::list_all(&state.db).map_err(to_err)?,
        routing_presets: routing::list_presets(&state.db).map_err(to_err)?,
        routing_rules: routing::list_rules(&state.db).map_err(to_err)?,
        settings: settings::get(&state.db).map_err(to_err)?,
    })
}

// ---------------------------------------------------------------------
// Connection lifecycle
// ---------------------------------------------------------------------

#[tauri::command]
pub async fn connect(tun: bool, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let all_profiles = profiles::list_all(&state.db).map_err(to_err)?;
    let routing_rules = routing::list_rules(&state.db).map_err(to_err)?;
    let active_profile_id = settings::get_active_profile_id(&state.db)
        .map_err(to_err)?
        .unwrap_or_default();

    let config = singbox::generate(&singbox::ConfigInput {
        profiles: &all_profiles,
        active_profile_id: &active_profile_id,
        routing_rules: &routing_rules,
        mixed_listen_port: state.paths.mixed_listen_port,
        clash_api_listen: &state.paths.clash_api_listen,
        tun,
    })
    .map_err(to_err)?;

    let config_bytes = serde_json::to_vec_pretty(&config).map_err(to_err)?;
    std::fs::write(&state.paths.config_path, config_bytes).map_err(to_err)?;

    state
        .supervisor
        .lock()
        .unwrap()
        .start(&state.paths.config_path)
        .map_err(to_err)?;

    let clash = ClashApiClient::new(format!("http://{}", state.paths.clash_api_listen));
    *state.clash.lock().unwrap() = Some(clash);

    let watcher = clash_api::watch_traffic(
        format!("ws://{}/traffic", state.paths.clash_api_listen),
        Duration::from_secs(2),
    );
    let (mut events, stop) = watcher.into_parts();
    *state.traffic_stop.lock().unwrap() = Some(stop);
    let traffic_app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.recv().await {
            let _ = traffic_app.emit("kagerou://traffic", &event);
        }
    });

    let _ = app.emit("kagerou://connection-changed", true);
    Ok(())
}

#[tauri::command]
pub async fn disconnect(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(stop) = state.traffic_stop.lock().unwrap().take() {
        let _ = stop.send(true);
    }
    *state.clash.lock().unwrap() = None;
    state.supervisor.lock().unwrap().stop().map_err(to_err)?;
    let _ = app.emit("kagerou://connection-changed", false);
    Ok(())
}

// ---------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------

#[tauri::command]
pub async fn select_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    profiles::select_profile(&state.db, &id).map_err(to_err)?;
    settings::set_active_profile_id(&state.db, Some(&id)).map_err(to_err)?;

    // Hot-switch the running sing-box instance without a restart when
    // already connected, instead of leaving it pointed at the old outbound
    // until the next connect().
    if let Some(clash) = state.clash_client() {
        let _ = clash.select_outbound("proxy", &id).await;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLocalProfileInput {
    pub name: String,
    pub key: String,
    pub group_id: Option<String>,
    pub source_id: Option<String>,
}

#[tauri::command]
pub fn add_local_profile(
    input: AddLocalProfileInput,
    state: State<AppState>,
) -> Result<String, String> {
    let id = new_id("local");
    let group_id = input.group_id.unwrap_or_else(|| "default".to_string());
    profiles::insert(
        &state.db,
        &NewProfile {
            id: id.clone(),
            name: input.name.trim().to_string(),
            region: "Local profile".to_string(),
            protocol: detect_protocol(&input.key),
            origin: "local".to_string(),
            group_id,
            source_id: input.source_id,
            key: input.key.trim().to_string(),
        },
    )
    .map_err(to_err)?;
    Ok(id)
}

#[tauri::command]
pub fn rename_profile(id: String, name: String, state: State<AppState>) -> Result<(), String> {
    profiles::rename(&state.db, &id, &name).map_err(to_err)
}

#[tauri::command]
pub fn delete_profile(id: String, state: State<AppState>) -> Result<(), String> {
    profiles::delete(&state.db, &id).map_err(to_err)
}

#[tauri::command]
pub fn move_profile_to_group(
    profile_id: String,
    target_group_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    profiles::move_to_group(&state.db, &profile_id, &target_group_id).map_err(to_err)
}

#[tauri::command]
pub fn move_profile(id: String, direction: String, state: State<AppState>) -> Result<(), String> {
    let profile = profiles::get(&state.db, &id).map_err(to_err)?;
    let group = groups::get(&state.db, &profile.group_id).map_err(to_err)?;
    let index = group
        .profile_ids
        .iter()
        .position(|candidate| candidate == &id)
        .ok_or("profile not found in its own group")?;
    let target_index = if direction == "up" {
        index.checked_sub(1)
    } else {
        index
            .checked_add(1)
            .filter(|i| *i < group.profile_ids.len())
    };
    let Some(target_index) = target_index else {
        return Err("cannot move past the edge of the group".to_string());
    };
    let mut ordered = group.profile_ids;
    ordered.swap(index, target_index);
    profiles::reorder(&state.db, &group.id, &ordered).map_err(to_err)
}

#[tauri::command]
pub fn reorder_profiles(
    from_id: String,
    to_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let from = profiles::get(&state.db, &from_id).map_err(to_err)?;
    let group = groups::get(&state.db, &from.group_id).map_err(to_err)?;
    let mut ordered = group.profile_ids;
    let from_index = ordered
        .iter()
        .position(|c| c == &from_id)
        .ok_or("profile not in group")?;
    let to_index = ordered
        .iter()
        .position(|c| c == &to_id)
        .ok_or("target profile not in the same group")?;
    let id = ordered.remove(from_index);
    ordered.insert(to_index, id);
    profiles::reorder(&state.db, &group.id, &ordered).map_err(to_err)
}

#[tauri::command]
pub async fn run_profile_test(
    profile_id: String,
    method: String,
    state: State<'_, AppState>,
) -> Result<TestResult, String> {
    let Some(clash) = state.clash_client() else {
        return Ok(TestResult {
            value: "Not connected".to_string(),
            tone: Tone::Muted,
        });
    };

    match method.as_str() {
        "tcp" => match clash
            .test_delay(&profile_id, "http://www.gstatic.com/generate_204", 5000)
            .await
        {
            Ok(delay_ms) => {
                let tone = if delay_ms < 150 {
                    Tone::Good
                } else if delay_ms < 400 {
                    Tone::Warn
                } else {
                    Tone::Bad
                };
                let result = TestResult {
                    value: format!("{delay_ms} ms"),
                    tone,
                };
                let _ = profiles::set_test_result(
                    &state.db,
                    &profile_id,
                    profiles::TestMethod::Tcp,
                    &result,
                );
                Ok(result)
            }
            Err(_) => {
                let result = TestResult {
                    value: "No response".to_string(),
                    tone: Tone::Bad,
                };
                let _ = profiles::set_test_result(
                    &state.db,
                    &profile_id,
                    profiles::TestMethod::Tcp,
                    &result,
                );
                Ok(result)
            }
        },
        "url" => match clash
            .test_delay(&profile_id, "http://www.gstatic.com/generate_204", 5000)
            .await
        {
            Ok(_) => {
                let result = TestResult {
                    value: "200 OK".to_string(),
                    tone: Tone::Good,
                };
                let _ = profiles::set_test_result(
                    &state.db,
                    &profile_id,
                    profiles::TestMethod::Url,
                    &result,
                );
                Ok(result)
            }
            Err(_) => {
                let result = TestResult {
                    value: "Timeout".to_string(),
                    tone: Tone::Bad,
                };
                let _ = profiles::set_test_result(
                    &state.db,
                    &profile_id,
                    profiles::TestMethod::Url,
                    &result,
                );
                Ok(result)
            }
        },
        other => Err(format!("unknown test method: {other}")),
    }
}

// ---------------------------------------------------------------------
// Profile groups
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_profile_group_open(
    id: String,
    open: bool,
    state: State<AppState>,
) -> Result<(), String> {
    groups::set_open(&state.db, &id, open).map_err(to_err)
}

#[tauri::command]
pub fn add_profile_group(label: String, state: State<AppState>) -> Result<String, String> {
    let id = new_id("group");
    groups::insert(
        &state.db,
        &NewProfileGroup {
            id: id.clone(),
            label,
            kind: "custom".to_string(),
            source_id: None,
        },
    )
    .map_err(to_err)?;
    Ok(id)
}

#[tauri::command]
pub fn rename_profile_group(
    id: String,
    label: String,
    state: State<AppState>,
) -> Result<(), String> {
    groups::rename(&state.db, &id, &label).map_err(to_err)
}

// ---------------------------------------------------------------------
// Subscription sources
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSourceInput {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: Option<String>,
    pub value: String,
}

#[tauri::command]
pub fn validate_source(kind: String, value: String) -> Option<String> {
    let value = value.trim();
    if kind == "url" {
        return if url::Url::parse(value)
            .map(|u| u.scheme() == "http" || u.scheme() == "https")
            .unwrap_or(false)
        {
            None
        } else {
            Some("invalidUrl".to_string())
        };
    }
    if subscription::parse_uri(value).is_ok() {
        None
    } else {
        Some("invalidKey".to_string())
    }
}

#[tauri::command]
pub async fn add_source(
    input: AddSourceInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let source_id = new_id("source");
    let value = input.value.trim().to_string();

    if input.kind == "key" {
        let name = input
            .name
            .unwrap_or_else(|| format!("{} key", detect_protocol(&value).as_str()));
        sources::insert(
            &state.db,
            &NewSource {
                id: source_id.clone(),
                name: name.clone(),
                kind: "key".to_string(),
                value: value.clone(),
                status: "ready".to_string(),
                last_refresh: "Added just now".to_string(),
                origin_label: "Local key".to_string(),
            },
        )
        .map_err(to_err)?;
        profiles::insert(
            &state.db,
            &NewProfile {
                id: new_id("local"),
                name,
                region: "Local profile".to_string(),
                protocol: detect_protocol(&value),
                origin: "local".to_string(),
                group_id: "default".to_string(),
                source_id: Some(source_id.clone()),
                key: value,
            },
        )
        .map_err(to_err)?;
        return Ok(source_id);
    }

    let body = fetch_subscription(&value).await.map_err(to_err)?;
    let parsed = subscription::parse_subscription(&body).map_err(to_err)?;
    let name = input.name.unwrap_or_else(|| "Subscription".to_string());
    let group_id = format!("subscription-{source_id}");

    sources::insert(
        &state.db,
        &NewSource {
            id: source_id.clone(),
            name: name.clone(),
            kind: "url".to_string(),
            value,
            status: "up-to-date".to_string(),
            last_refresh: "Updated just now".to_string(),
            origin_label: "Remote URL".to_string(),
        },
    )
    .map_err(to_err)?;
    groups::insert(
        &state.db,
        &NewProfileGroup {
            id: group_id.clone(),
            label: name,
            kind: "subscription".to_string(),
            source_id: Some(source_id.clone()),
        },
    )
    .map_err(to_err)?;
    for outbound in &parsed {
        profiles::insert(
            &state.db,
            &parsed_outbound_to_new_profile(outbound, &group_id, &source_id),
        )
        .map_err(to_err)?;
    }
    Ok(source_id)
}

#[tauri::command]
pub async fn refresh_source(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let source = sources::get(&state.db, &id).map_err(to_err)?;
    if source.kind != "url" {
        return Ok(());
    }
    let body = fetch_subscription(&source.value).await.map_err(to_err)?;
    let parsed = subscription::parse_subscription(&body).map_err(to_err)?;
    let group = groups::list_all(&state.db)
        .map_err(to_err)?
        .into_iter()
        .find(|g| g.source_id.as_deref() == Some(id.as_str()))
        .ok_or("no group for this source")?;

    let existing_by_key: std::collections::HashMap<String, String> = profiles::list_all(&state.db)
        .map_err(to_err)?
        .into_iter()
        .filter(|p| p.group_id == group.id)
        .map(|p| (p.key, p.id))
        .collect();

    for old_id in &group.profile_ids {
        let _ = profiles::delete(&state.db, old_id);
    }
    for outbound in &parsed {
        let mut new_profile = parsed_outbound_to_new_profile(outbound, &group.id, &id);
        if let Some(existing_id) = existing_by_key.get(&new_profile.key) {
            new_profile.id = existing_id.clone();
        }
        profiles::insert(&state.db, &new_profile).map_err(to_err)?;
    }
    sources::update(
        &state.db,
        &id,
        &sources::SourcePatch {
            name: None,
            value: None,
            status: Some("up-to-date"),
            last_refresh: Some("Updated just now"),
        },
    )
    .map_err(to_err)?;
    Ok(())
}

#[tauri::command]
pub fn remove_source(id: String, state: State<AppState>) -> Result<(), String> {
    sources::delete(&state.db, &id).map_err(to_err)
}

fn parsed_outbound_to_new_profile(
    outbound: &subscription::model::ParsedOutbound,
    group_id: &str,
    source_id: &str,
) -> NewProfile {
    let protocol = match outbound.protocol_label() {
        "VMess" => Protocol::VMess,
        "Trojan" => Protocol::Trojan,
        "Shadowsocks" => Protocol::Shadowsocks,
        "Hysteria2" => Protocol::Hysteria2,
        _ => Protocol::VLESS,
    };
    NewProfile {
        id: new_id("imported"),
        name: outbound.name().to_string(),
        region: outbound.server().to_string(),
        protocol,
        origin: "imported".to_string(),
        group_id: group_id.to_string(),
        source_id: Some(source_id.to_string()),
        key: reconstruct_key(outbound),
    }
}

/// The subscription parser normalizes a URI into structured fields; the
/// config generator later re-parses `profile.key` (see
/// singbox::config::generate's doc comment) rather than trusting cached
/// fields, so what's stored here just needs to be *a* URI that round-trips
/// to the same outbound, not byte-identical to the original subscription
/// line. Re-serializing from the parsed form keeps this independent of
/// whatever the origin server's exact formatting was.
fn reconstruct_key(outbound: &subscription::model::ParsedOutbound) -> String {
    use subscription::model::ParsedOutbound;
    match outbound {
        ParsedOutbound::Vless(o) => format!(
            "vless://{}@{}:{}?encryption=none#{}",
            o.uuid, o.server, o.port, o.name
        ),
        ParsedOutbound::Vmess(o) => {
            format!("vmess://{}@{}:{}#{}", o.uuid, o.server, o.port, o.name)
        }
        ParsedOutbound::Trojan(o) => {
            format!("trojan://{}@{}:{}#{}", o.password, o.server, o.port, o.name)
        }
        ParsedOutbound::Shadowsocks(o) => {
            use base64::Engine;
            let userinfo = base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", o.method, o.password));
            format!("ss://{}@{}:{}#{}", userinfo, o.server, o.port, o.name)
        }
        ParsedOutbound::Hysteria2(o) => format!(
            "hysteria2://{}@{}:{}#{}",
            o.password, o.server, o.port, o.name
        ),
        ParsedOutbound::Tuic(o) => format!(
            "tuic://{}:{}@{}:{}#{}",
            o.uuid, o.password, o.server, o.port, o.name
        ),
    }
}

async fn fetch_subscription(url: &str) -> Result<String, String> {
    reqwest::Client::new()
        .get(url)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(to_err)?
        .error_for_status()
        .map_err(to_err)?
        .text()
        .await
        .map_err(to_err)
}

// ---------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_preset(id: String, enabled: bool, state: State<AppState>) -> Result<(), String> {
    routing::set_preset(&state.db, &id, enabled).map_err(to_err)
}

#[tauri::command]
pub fn select_rule(id: String, state: State<AppState>) -> Result<(), String> {
    routing::select_rule(&state.db, &id).map_err(to_err)
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
) -> Result<(), String> {
    routing::update_rule(
        &state.db,
        &id,
        &routing::RulePatch {
            match_value: patch.match_value.as_deref(),
            outbound: patch.outbound.as_deref(),
        },
    )
    .map_err(to_err)
}

#[tauri::command]
pub fn add_routing_rule(
    match_value: String,
    outbound: String,
    state: State<AppState>,
) -> Result<String, String> {
    let id = new_id("rule");
    routing::insert_rule(
        &state.db,
        &NewRoutingRule {
            id: id.clone(),
            match_value,
            outbound,
        },
    )
    .map_err(to_err)?;
    Ok(id)
}

// ---------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------

#[tauri::command]
pub fn set_theme(theme_id: String, state: State<AppState>) -> Result<(), String> {
    settings::update(
        &state.db,
        &settings::SettingsPatch {
            theme: Some(&theme_id),
            ..Default::default()
        },
    )
    .map_err(to_err)
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatchInput {
    pub language: Option<String>,
    pub startup: Option<bool>,
    pub tun_interface: Option<String>,
    pub auto_update_subscriptions: Option<bool>,
    pub subscription_update_interval: Option<String>,
    pub custom_subscription_update_minutes: Option<i64>,
    pub group_sort: Option<String>,
}

#[tauri::command]
pub fn update_settings(patch: SettingsPatchInput, state: State<AppState>) -> Result<(), String> {
    settings::update(
        &state.db,
        &settings::SettingsPatch {
            theme: None,
            language: patch.language.as_deref(),
            startup: patch.startup,
            tun_interface: patch.tun_interface.as_deref(),
            auto_update_subscriptions: patch.auto_update_subscriptions,
            subscription_update_interval: patch.subscription_update_interval.as_deref(),
            custom_subscription_update_minutes: patch.custom_subscription_update_minutes,
            group_sort: patch.group_sort.as_deref(),
        },
    )
    .map_err(to_err)
}
