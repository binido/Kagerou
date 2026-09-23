use serde::Serialize;

use crate::clash_api::model::ConnectionsResponse;
use crate::clash_api::TrafficEvent;
use crate::storage::models::TestResult;

/// What the dashboard receives on `kagerou://traffic`: the raw `/traffic`
/// sample plus, when the Clash API answered, the session-wide byte totals
/// from `/connections`.
///
/// Backend-sourced totals survive a frontend reload mid-session (sing-box
/// keeps counting while the WebView is gone); a client-side accumulator
/// would not. A failed `/connections` fetch yields `None` fields and the UI
/// keeps its previous totals, so one API hiccup does not blank the panel.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DashboardTrafficEvent {
    #[serde(rename_all = "camelCase")]
    Sample {
        up: u64,
        down: u64,
        upload_total: Option<u64>,
        download_total: Option<u64>,
        active_connections: Option<usize>,
    },
    Disconnected,
    Reconnecting,
}

impl DashboardTrafficEvent {
    pub fn new(event: &TrafficEvent, totals: Option<&ConnectionsResponse>) -> Self {
        match event {
            TrafficEvent::Sample(sample) => Self::Sample {
                up: sample.up,
                down: sample.down,
                upload_total: totals.map(|t| t.upload_total),
                download_total: totals.map(|t| t.download_total),
                active_connections: totals.map(|t| t.connections.len()),
            },
            TrafficEvent::Disconnected => Self::Disconnected,
            TrafficEvent::Reconnecting => Self::Reconnecting,
        }
    }
}

/// One profile's result, as a group run produces them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProgress {
    pub profile_id: String,
    pub result: TestResult,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinished {
    pub done: usize,
    pub total: usize,
    pub cancelled: bool,
}

/// How far the download of a new version has got, in bytes.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub downloaded: u64,
    /// `None` when the server sends no Content-Length.
    pub total: Option<u64>,
}

/// Everything the backend pushes into the window.
///
/// The other half of this list is the frontend's listener table in
/// `app/src/lib/tauri-api.ts`, and a test below pins the names against it.
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    ConnectionChanged(bool),
    Crashed {
        exit_code: Option<i32>,
    },
    Log(String),
    Traffic(DashboardTrafficEvent),
    TestProgress(TestProgress),
    TestFinished(TestFinished),
    /// The tray asks the frontend to act rather than acting itself:
    /// connecting and switching profiles are already implemented there, on
    /// top of the same commands, and a second path would be a second set of
    /// bugs.
    TrayToggleConnection,
    TraySelectProfile(String),
    UpdateProgress(UpdateProgress),
}

impl AppEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConnectionChanged(_) => "kagerou://connection-changed",
            Self::Crashed { .. } => "kagerou://crashed",
            Self::Log(_) => "kagerou://log",
            Self::Traffic(_) => "kagerou://traffic",
            Self::TestProgress(_) => "kagerou://test-progress",
            Self::TestFinished(_) => "kagerou://test-finished",
            Self::TrayToggleConnection => "kagerou://tray-toggle-connection",
            Self::TraySelectProfile(_) => "kagerou://tray-select-profile",
            Self::UpdateProgress(_) => "kagerou://update-progress",
        }
    }
}

/// Where events go. A dropped event is not worth failing a connection over,
/// so nothing here returns a result.
pub trait Events: Clone + Send + Sync + 'static {
    fn emit(&self, event: AppEvent);
}

impl<R: tauri::Runtime> Events for tauri::AppHandle<R> {
    fn emit(&self, event: AppEvent) {
        use tauri::Emitter as TauriEmitter;
        let name = event.name();
        let _ = match event {
            AppEvent::ConnectionChanged(connected) => TauriEmitter::emit(self, name, connected),
            AppEvent::Crashed { exit_code } => TauriEmitter::emit(self, name, exit_code),
            AppEvent::Log(line) => TauriEmitter::emit(self, name, line),
            AppEvent::Traffic(payload) => TauriEmitter::emit(self, name, payload),
            AppEvent::TestProgress(payload) => TauriEmitter::emit(self, name, payload),
            AppEvent::TestFinished(payload) => TauriEmitter::emit(self, name, payload),
            AppEvent::TrayToggleConnection => TauriEmitter::emit(self, name, ()),
            AppEvent::TraySelectProfile(profile_id) => TauriEmitter::emit(self, name, profile_id),
            AppEvent::UpdateProgress(payload) => TauriEmitter::emit(self, name, payload),
        };
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{AppEvent, Events};
    use std::sync::{Arc, Mutex};

    /// Collects what was emitted, so logic that reports through `Events` can
    /// be asserted on without a window.
    #[derive(Clone, Default)]
    pub(crate) struct RecordedEvents(Arc<Mutex<Vec<AppEvent>>>);

    impl RecordedEvents {
        pub(crate) fn all(&self) -> Vec<AppEvent> {
            self.0.lock().unwrap().clone()
        }
    }

    impl Events for RecordedEvents {
        fn emit(&self, event: AppEvent) {
            self.0.lock().unwrap().push(event);
        }
    }
}

#[cfg(test)]
mod tests;
