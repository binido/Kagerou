use super::*;
use crate::subscription::{parse_subscription, Unsupported};

const FIXTURE: &str = include_str!("fixture.json");

fn only(config: serde_json::Value) -> Result<ParsedOutbound, SubscriptionError> {
    let mut entries = try_parse_xray_json(&config.to_string()).expect("recognised as Xray");
    assert_eq!(entries.len(), 1);
    entries.remove(0)
}

#[test]
fn an_array_of_configs_becomes_one_profile_per_plain_config() {
    let parsed = parse_subscription(FIXTURE).unwrap();

    let names: Vec<&str> = parsed.outbounds.iter().map(|o| o.name()).collect();
    assert_eq!(
        names,
        vec!["🇩🇪 Germany", "🇳🇱 Netherlands GAMING", "🇮🇹 Italy SS"]
    );
    assert_eq!(
        parsed.unsupported,
        vec![
            Unsupported::Balancer,
            Unsupported::Transport("xhttp".into()),
            Unsupported::Chain,
        ]
    );
}

#[test]
fn a_reality_vless_config_keeps_everything_the_handshake_needs() {
    let parsed = parse_subscription(FIXTURE).unwrap();
    let ParsedOutbound::Vless(v) = &parsed.outbounds[0] else {
        panic!("expected Vless, got {:?}", parsed.outbounds[0]);
    };
    assert_eq!((v.server.as_str(), v.port), ("de.example.com", 8443));
    assert_eq!(v.uuid, "11111111-2222-3333-4444-555555555555");
    assert_eq!(v.flow.as_deref(), Some("xtls-rprx-vision"));
    assert!(v.tls);
    assert_eq!(v.sni.as_deref(), Some("cdn.example.org"));
    assert_eq!(
        v.reality_public_key.as_deref(),
        Some("synthetic-public-key")
    );
    assert_eq!(v.reality_short_id.as_deref(), Some("abcd"));
    assert_eq!(v.fingerprint.as_deref(), Some("firefox"));
}

#[test]
fn xray_hysteria_version_2_is_read_as_hysteria2() {
    let parsed = parse_subscription(FIXTURE).unwrap();
    let ParsedOutbound::Hysteria2(h) = &parsed.outbounds[1] else {
        panic!("expected Hysteria2, got {:?}", parsed.outbounds[1]);
    };
    assert_eq!(h.password, "synthetic-auth");
    assert_eq!(h.sni.as_deref(), Some("nl.example.com"));
}

#[test]
fn xray_hysteria_version_1_is_left_out() {
    let result = only(serde_json::json!({
        "outbounds": [{ "protocol": "hysteria", "settings": { "address": "h.example.com", "port": 1, "version": 1 } }]
    }));
    assert_eq!(
        result.unwrap_err(),
        SubscriptionError::UnsupportedProtocol("hysteria".into())
    );
}

#[test]
fn a_single_config_object_is_read_too() {
    let result = only(serde_json::json!({
        "outbounds": [{
            "protocol": "vless",
            "settings": { "address": "flat.example.com", "port": 443, "id": "flat-id" }
        }]
    }));
    let ParsedOutbound::Vless(v) = result.unwrap() else {
        panic!("expected Vless");
    };
    assert_eq!(v.uuid, "flat-id");
    assert_eq!(v.name, "vless flat.example.com");
}

#[test]
fn a_config_without_a_proxy_outbound_is_invalid() {
    let result = only(serde_json::json!({ "outbounds": [{ "protocol": "freedom" }] }));
    assert!(matches!(
        result,
        Err(SubscriptionError::InvalidXrayConfig { .. })
    ));
}

#[test]
fn a_config_with_a_broken_port_is_invalid() {
    let result = only(serde_json::json!({
        "outbounds": [{ "protocol": "trojan", "settings": { "servers": [{ "address": "t.example.com", "port": "http", "password": "p" }] } }]
    }));
    assert!(matches!(
        result,
        Err(SubscriptionError::InvalidXrayConfig { .. })
    ));
}

#[test]
fn an_unknown_xray_protocol_is_left_out_by_name() {
    let result = only(serde_json::json!({
        "outbounds": [{ "protocol": "wireguard", "settings": { "address": "w.example.com", "port": 51820 } }]
    }));
    assert_eq!(
        result.unwrap_err(),
        SubscriptionError::UnsupportedProtocol("wireguard".into())
    );
}

#[test]
fn json_that_is_not_xray_is_not_claimed() {
    assert!(try_parse_xray_json("[1, 2, 3]").is_none());
    assert!(try_parse_xray_json("[]").is_none());
    assert!(try_parse_xray_json("[{\"outbounds\": []}]").is_none());
    assert!(try_parse_xray_json("{\"outbounds\": [{\"type\": \"vless\"}]}").is_none());
    assert!(try_parse_xray_json("[{\"outbounds\": [{\"protocol\": \"vless\"}]").is_none());
    assert!(try_parse_xray_json("proxies: []").is_none());
}
