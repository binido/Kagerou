use super::*;
use crate::clash_api::model::{ConnectionInfo, ConnectionMetadata, TrafficSample};

fn connection(id: &str) -> ConnectionInfo {
    ConnectionInfo {
        id: id.to_string(),
        metadata: ConnectionMetadata {
            host: String::new(),
            network: String::new(),
            destination_port: String::new(),
        },
        upload: 0,
        download: 0,
        chains: vec![],
        rule: String::new(),
    }
}

fn totals() -> ConnectionsResponse {
    ConnectionsResponse {
        download_total: 1_900_000_000,
        upload_total: 250_000_000,
        connections: vec![connection("a"), connection("b")],
    }
}

#[test]
fn a_sample_event_carries_the_session_totals_when_connections_answered() {
    let event = TrafficEvent::Sample(TrafficSample { up: 100, down: 200 });
    assert_eq!(
        DashboardTrafficEvent::new(&event, Some(&totals())),
        DashboardTrafficEvent::Sample {
            up: 100,
            down: 200,
            upload_total: Some(250_000_000),
            download_total: Some(1_900_000_000),
            active_connections: Some(2),
        }
    );
}

#[test]
fn a_sample_event_omits_the_totals_when_connections_failed() {
    let event = TrafficEvent::Sample(TrafficSample { up: 100, down: 200 });
    assert_eq!(
        DashboardTrafficEvent::new(&event, None),
        DashboardTrafficEvent::Sample {
            up: 100,
            down: 200,
            upload_total: None,
            download_total: None,
            active_connections: None,
        },
        "the sample itself must still reach the dashboard"
    );
}

#[test]
fn non_sample_events_pass_through_untouched() {
    assert_eq!(
        DashboardTrafficEvent::new(&TrafficEvent::Disconnected, Some(&totals())),
        DashboardTrafficEvent::Disconnected
    );
    assert_eq!(
        DashboardTrafficEvent::new(&TrafficEvent::Reconnecting, None),
        DashboardTrafficEvent::Reconnecting
    );
}

#[test]
fn the_wire_format_matches_what_the_frontend_expects() {
    let event = DashboardTrafficEvent::new(
        &TrafficEvent::Sample(TrafficSample { up: 1, down: 2 }),
        Some(&totals()),
    );
    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        serde_json::json!({
            "kind": "sample",
            "up": 1,
            "down": 2,
            "uploadTotal": 250_000_000_u64,
            "downloadTotal": 1_900_000_000_u64,
            "activeConnections": 2,
        })
    );
}

/// The other half of this list is `app/src/lib/tauri-api.ts`; a rename on
/// one side that is not made on the other silently stops the UI updating.
#[test]
fn every_event_keeps_the_name_the_frontend_listens_on() {
    let names: Vec<&str> = vec![
        AppEvent::ConnectionChanged(true).name(),
        AppEvent::Crashed { exit_code: None }.name(),
        AppEvent::Log(String::new()).name(),
        AppEvent::Traffic(DashboardTrafficEvent::Disconnected).name(),
        AppEvent::TestProgress(TestProgress {
            profile_id: String::new(),
            result: crate::storage::models::TestOutcome::NotTested.into(),
            done: 0,
            total: 0,
        })
        .name(),
        AppEvent::TestFinished(TestFinished {
            done: 0,
            total: 0,
            cancelled: false,
        })
        .name(),
        AppEvent::TrayToggleConnection.name(),
        AppEvent::TraySelectProfile(String::new()).name(),
    ];
    assert_eq!(
        names,
        [
            "kagerou://connection-changed",
            "kagerou://crashed",
            "kagerou://log",
            "kagerou://traffic",
            "kagerou://test-progress",
            "kagerou://test-finished",
            "kagerou://tray-toggle-connection",
            "kagerou://tray-select-profile",
        ]
    );
}
