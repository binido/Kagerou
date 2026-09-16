use super::{test_core_is_idle, to_dashboard_event, DashboardTrafficEvent};
use crate::clash_api::model::{
    ConnectionInfo, ConnectionMetadata, ConnectionsResponse, TrafficSample,
};
use crate::clash_api::TrafficEvent;
use std::time::{Duration, Instant};

#[test]
fn a_test_core_is_idle_only_after_the_timeout_has_passed() {
    let now = Instant::now();
    let timeout = Duration::from_secs(30);

    assert!(!test_core_is_idle(
        Some(now - Duration::from_secs(5)),
        now,
        timeout
    ));
    assert!(test_core_is_idle(
        Some(now - Duration::from_secs(31)),
        now,
        timeout
    ));
    assert!(
        test_core_is_idle(None, now, timeout),
        "a core with no recorded test outlived whatever started it"
    );
}

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
        to_dashboard_event(&event, Some(&totals())),
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
        to_dashboard_event(&event, None),
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
        to_dashboard_event(&TrafficEvent::Disconnected, Some(&totals())),
        DashboardTrafficEvent::Disconnected
    );
    assert_eq!(
        to_dashboard_event(&TrafficEvent::Reconnecting, None),
        DashboardTrafficEvent::Reconnecting
    );
}

#[test]
fn the_wire_format_matches_what_the_frontend_expects() {
    let event = to_dashboard_event(
        &TrafficEvent::Sample(TrafficSample { up: 1, down: 2 }),
        Some(&totals()),
    );
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(
        json,
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
