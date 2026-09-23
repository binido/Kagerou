use serde_json::json;

use super::{live_connections, worth_auto_connecting, ConnectionExit};
use crate::clash_api::model::ConnectionsResponse;
use crate::storage::models::{Profile, Protocol, TestOutcome};

fn profile(id: &str, name: &str) -> Profile {
    Profile {
        id: id.into(),
        name: name.into(),
        region: "r".into(),
        protocol: Protocol::VLESS,
        origin: "local".into(),
        group_id: "g".into(),
        source_id: None,
        selected: false,
        url: TestOutcome::NotTested.into(),
        key: String::new(),
    }
}

/// Builds a `/connections` body with one connection per chain.
fn response(chains: &[&[&str]]) -> ConnectionsResponse {
    let connections: Vec<_> = chains
        .iter()
        .enumerate()
        .map(|(i, chain)| {
            json!({
                "id": format!("c{i}"),
                "metadata": {
                    "host": "example.com",
                    "destinationIP": "93.184.216.34",
                    "network": "tcp",
                    "destinationPort": "443"
                },
                "upload": 10,
                "download": 20,
                "chains": chain,
                "rule": "final",
                "start": "2026-09-23T10:00:00.123456789+03:00"
            })
        })
        .collect();
    serde_json::from_value(json!({ "connections": connections })).unwrap()
}

#[test]
fn auto_connect_needs_both_a_profile_to_use_and_profiles_to_pick_from() {
    assert!(worth_auto_connecting("p1", 3));
    assert!(
        !worth_auto_connecting("", 3),
        "nothing was selected last session"
    );
    assert!(
        !worth_auto_connecting("p1", 0),
        "the selected profile is gone, so there is nothing to connect to"
    );
}

#[test]
fn a_connection_through_a_profile_is_named_after_it_wherever_it_sits_in_the_chain() {
    let profiles = [profile("p1", "Tokyo")];
    let listed = live_connections(response(&[&["p1", "proxy"], &["proxy", "p1"]]), &profiles);
    let expected = ConnectionExit::Profile {
        name: "Tokyo".into(),
    };
    assert_eq!(listed[0].exit, expected);
    assert_eq!(listed[1].exit, expected);
}

#[test]
fn direct_block_and_unknown_exits_are_told_apart() {
    let listed = live_connections(response(&[&["direct"], &["block"], &["dns"], &[]]), &[]);
    assert_eq!(listed[0].exit, ConnectionExit::Direct);
    assert_eq!(listed[1].exit, ConnectionExit::Block);
    assert_eq!(listed[2].exit, ConnectionExit::Other { tag: "dns".into() });
    assert_eq!(listed[3].exit, ConnectionExit::Other { tag: String::new() });
}

#[test]
fn a_profile_deleted_mid_session_is_shown_by_its_tag() {
    let listed = live_connections(response(&[&["gone", "proxy"]]), &[]);
    assert_eq!(listed[0].exit, ConnectionExit::Other { tag: "gone".into() });
}

#[test]
fn a_connection_without_a_host_shows_the_address_it_went_to() {
    let mut body = response(&[&["direct"]]);
    body.connections[0].metadata.host.clear();
    let listed = live_connections(body, &[]);
    assert_eq!(listed[0].host, "93.184.216.34");
}

#[test]
fn the_wire_format_keeps_the_names_the_frontend_reads() {
    let listed = live_connections(response(&[&["p1", "proxy"]]), &[profile("p1", "Tokyo")]);
    assert_eq!(
        serde_json::to_value(&listed[0]).unwrap(),
        json!({
            "id": "c0",
            "host": "example.com",
            "port": "443",
            "network": "tcp",
            "rule": "final",
            "exit": { "kind": "profile", "name": "Tokyo" },
            "upload": 10,
            "download": 20,
            "start": "2026-09-23T10:00:00.123456789+03:00"
        })
    );
    assert_eq!(
        serde_json::to_value(ConnectionExit::Other { tag: "dns".into() }).unwrap(),
        json!({ "kind": "other", "tag": "dns" })
    );
}
