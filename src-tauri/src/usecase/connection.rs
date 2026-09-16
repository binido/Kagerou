//! Bringing the user's tunnel up and down, and keeping the window told
//! about it while it is up.
//!
//! Two background pumps run for the life of a connection: one turns the
//! Clash API's traffic samples into dashboard events, the other forwards the
//! core's log output and notices when the core dies on its own.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use super::core::{self, CoreSpec};
use super::events::{AppEvent, DashboardTrafficEvent, Events};
use crate::app_state::AppState;
use crate::clash_api::{self, ClashApiClient, TrafficEvent};
use crate::singbox::Status;
use crate::storage::{profiles, settings};

/// How often the log pump drains the supervisor.
const LOG_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// How long the traffic websocket waits before trying again.
const TRAFFIC_RECONNECT_DELAY: Duration = Duration::from_secs(2);

/// Brings the connection up, shared by the connect command and the startup
/// auto-connect so the two can never drift.
///
/// TUN and the log level are stored preferences, not per-call arguments:
/// they are toggled in settings and take effect on the next connection.
pub async fn connect(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let stored = settings::get(&state.db).map_err(|e| e.to_string())?;
    core::start(
        &state.db,
        &mut state.supervisor.lock().unwrap(),
        &CoreSpec::connection(&state.paths, &stored),
    )
    .map_err(|e| e.to_string())?;

    let clash = ClashApiClient::new(format!("http://{}", state.paths.clash_api_listen));
    *state.clash.lock().unwrap() = Some(clash.clone());

    spawn_traffic_pump(app.clone(), state, clash);
    spawn_log_pump(app.clone());

    *state.connected_since.lock().unwrap() = Some(SystemTime::now());
    Events::emit(app, AppEvent::ConnectionChanged(true));
    crate::tray::refresh(app, true);
    Ok(())
}

pub fn disconnect(app: &AppHandle, state: &AppState) -> Result<(), String> {
    if let Some(stop) = state.traffic_stop.lock().unwrap().take() {
        let _ = stop.send(true);
    }
    *state.clash.lock().unwrap() = None;
    *state.connected_since.lock().unwrap() = None;
    state
        .supervisor
        .lock()
        .unwrap()
        .stop()
        .map_err(|e| e.to_string())?;
    Events::emit(app, AppEvent::ConnectionChanged(false));
    crate::tray::refresh(app, false);
    Ok(())
}

/// Whether there is anything to auto-connect to. Nothing to connect to is
/// not an error: the app just starts disconnected.
fn worth_auto_connecting(active_profile_id: &str, profile_count: usize) -> bool {
    !active_profile_id.is_empty() && profile_count > 0
}

/// The startup side of auto-connect. `.setup()` reads the setting and only
/// calls this when it is on, so the remaining job is to skip quietly when
/// there is nothing to connect to and otherwise take the path the connect
/// command takes.
pub async fn auto_connect(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let active_profile_id = settings::get_active_profile_id(&state.db)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let profile_count = profiles::list_all(&state.db)
        .map_err(|e| e.to_string())?
        .len();
    if !worth_auto_connecting(&active_profile_id, profile_count) {
        return Ok(());
    }
    connect(app, state.inner()).await
}

/// The connection's start time as unix milliseconds. A clock set before 1970
/// is the only way this fails, and an absent uptime beats a panic.
pub fn connected_since_millis(state: &AppState) -> Option<u64> {
    let since = (*state.connected_since.lock().unwrap())?;
    since
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

/// Turns the Clash API's traffic samples into what the dashboard draws.
///
/// Each sample is followed by a `/connections` read for the session totals;
/// a failure there sends the sample with empty totals rather than dropping
/// it, so one API hiccup does not blank the panel.
fn spawn_traffic_pump(app: AppHandle, state: &AppState, clash: ClashApiClient) {
    let watcher = clash_api::watch_traffic(
        format!("ws://{}/traffic", state.paths.clash_api_listen),
        TRAFFIC_RECONNECT_DELAY,
    );
    let (mut events, stop) = watcher.into_parts();
    *state.traffic_stop.lock().unwrap() = Some(stop);
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.recv().await {
            let totals = if matches!(event, TrafficEvent::Sample(_)) {
                clash.get_connections().await.ok()
            } else {
                None
            };
            Events::emit(
                &app,
                AppEvent::Traffic(DashboardTrafficEvent::new(&event, totals.as_ref())),
            );
        }
    });
}

/// Forwards the core's output to the log viewer, and is also what notices a
/// core that died on its own: nothing else polls the supervisor.
fn spawn_log_pump(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut forwarded = 0usize;
        let mut ticker = tokio::time::interval(LOG_POLL_INTERVAL);
        loop {
            ticker.tick().await;
            let state = app.state::<AppState>();
            let (lines, status) = {
                let mut supervisor = state.supervisor.lock().unwrap();
                supervisor.poll_events();
                let (lines, produced) = supervisor.logs_since(forwarded);
                let lines: Vec<String> = lines.cloned().collect();
                forwarded = produced;
                (lines, supervisor.status().clone())
            };
            for line in lines {
                Events::emit(&app, AppEvent::Log(line));
            }
            match status {
                Status::Crashed { exit_code } => {
                    *state.connected_since.lock().unwrap() = None;
                    Events::emit(&app, AppEvent::ConnectionChanged(false));
                    crate::tray::refresh(&app, false);
                    Events::emit(&app, AppEvent::Crashed { exit_code });
                    return;
                }
                // A requested stop: `disconnect` has already said so.
                Status::Stopped => return,
                Status::Running => {}
            }
        }
    });
}

#[cfg(test)]
mod tests;
