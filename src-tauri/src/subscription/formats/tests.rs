use super::*;

#[test]
fn parses_a_plain_newline_uri_list() {
    let content = "vless://uuid@example.com:443#A\ntrojan://pw@relay.example.com:443#B\n";
    let outbounds = parse_subscription(content).unwrap().outbounds;
    assert_eq!(outbounds.len(), 2);
    assert_eq!(outbounds[0].protocol_label(), "VLESS");
    assert_eq!(outbounds[1].protocol_label(), "Trojan");
}

#[test]
fn parses_a_base64_encoded_uri_list() {
    let raw =
        "vless://uuid@example.com:443#A\nss://YWVzLTI1Ni1nY206aHVudGVyMg==@ss.example.com:8388#B";
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, raw);
    let outbounds = parse_subscription(&encoded).unwrap().outbounds;
    assert_eq!(outbounds.len(), 2);
}

#[test]
fn skips_blank_lines_in_a_uri_list() {
    let content = "\n\nvless://uuid@example.com:443#A\n\n\n";
    let outbounds = parse_subscription(content).unwrap().outbounds;
    assert_eq!(outbounds.len(), 1);
}

#[test]
fn a_bad_uri_is_left_out_and_the_rest_of_the_list_kept() {
    let content = "vless://uuid@example.com:443#A\nvless://missing-port@example.com#B\nssr://abc";
    let parsed = parse_subscription(content).unwrap();
    assert_eq!(parsed.outbounds.len(), 1);
    assert_eq!(
        parsed.unsupported,
        vec![Unsupported::Invalid, Unsupported::Protocol("ssr".into())]
    );
}

#[test]
fn a_list_of_only_bad_uris_fails_with_the_first_error() {
    let content = "vless://missing-port@example.com#B\nssr://abc";
    let err = parse_subscription(content).unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn a_list_starting_with_an_unknown_scheme_is_still_a_uri_list() {
    let parsed = parse_subscription("ssr://abc\ntrojan://pw@relay.example.com:443#B").unwrap();
    assert_eq!(parsed.outbounds.len(), 1);
    assert_eq!(
        parsed.unsupported,
        vec![Unsupported::Protocol("ssr".into())]
    );
}

#[test]
fn a_transport_sing_box_lacks_is_left_out() {
    let content =
        "vless://uuid@example.com:443?type=xhttp#A\nvless://uuid@example.com:443?type=ws#B";
    let parsed = parse_subscription(content).unwrap();
    assert_eq!(parsed.outbounds.len(), 1);
    assert_eq!(
        parsed.unsupported,
        vec![Unsupported::Transport("xhttp".into())]
    );
}

#[test]
fn xray_raw_transport_is_read_as_tcp() {
    let outbounds = parse_subscription("vless://uuid@example.com:443?type=raw#A")
        .unwrap()
        .outbounds;
    match &outbounds[0] {
        ParsedOutbound::Vless(v) => assert_eq!(v.network, "tcp"),
        other => panic!("expected Vless, got {other:?}"),
    }
}

#[test]
fn unsupported_reasons_keep_their_wire_format() {
    assert_eq!(
        serde_json::to_value(Unsupported::Transport("xhttp".into())).unwrap(),
        serde_json::json!({ "kind": "transport", "name": "xhttp" })
    );
    assert_eq!(
        serde_json::to_value(Unsupported::Invalid).unwrap(),
        serde_json::json!({ "kind": "invalid" })
    );
}

#[test]
fn parses_a_clash_yaml_subscription() {
    let yaml = r#"
proxies:
  - name: "Tokyo"
    type: vless
    server: jp.example.com
    port: 443
    uuid: 11111111-2222-3333-4444-555555555555
    network: ws
    tls: true
    servername: jp.example.com
    ws-opts:
      path: /ray
      headers:
        Host: jp.example.com
  - name: "Relay"
    type: trojan
    server: relay.example.com
    port: 443
    password: hunter2
    sni: relay.example.com
"#;
    let outbounds = parse_subscription(yaml).unwrap().outbounds;
    assert_eq!(outbounds.len(), 2);
    match &outbounds[0] {
        ParsedOutbound::Vless(v) => {
            assert_eq!(v.name, "Tokyo");
            assert_eq!(v.ws_path.as_deref(), Some("/ray"));
            assert_eq!(v.ws_host.as_deref(), Some("jp.example.com"));
        }
        other => panic!("expected Vless, got {other:?}"),
    }
}

#[test]
fn clash_yaml_rejects_a_proxy_missing_required_fields() {
    let yaml = "proxies:\n  - name: Broken\n    type: trojan\n    server: relay.example.com\n";
    let err = parse_subscription(yaml).unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidClashProxy { .. }));
}

#[test]
fn clash_yaml_leaves_out_an_unsupported_proxy_type() {
    let yaml = "proxies:\n  - name: Unsupported\n    type: snell\n    server: s.example.com\n    port: 1\n  - name: Relay\n    type: trojan\n    server: relay.example.com\n    port: 443\n    password: hunter2\n";
    let parsed = parse_subscription(yaml).unwrap();
    assert_eq!(parsed.outbounds.len(), 1);
    assert_eq!(
        parsed.unsupported,
        vec![Unsupported::Protocol("snell".into())]
    );
}

#[test]
fn singbox_json_with_only_groups_and_direct_is_rejected() {
    let json = serde_json::json!({ "outbounds": [ { "type": "direct" }, { "type": "selector" } ] });
    assert_eq!(
        parse_subscription(&json.to_string()).unwrap_err(),
        SubscriptionError::Empty
    );
}

#[test]
fn parses_a_singbox_json_subscription() {
    let json = serde_json::json!({
        "outbounds": [
            { "type": "direct", "tag": "direct" },
            {
                "type": "hysteria2", "tag": "HY2 Node", "server": "hy.example.com", "server_port": 443,
                "password": "hunter2", "tls": { "enabled": true, "server_name": "hy.example.com", "insecure": false },
                "obfs": { "type": "salamander", "password": "obfspw" }
            },
            { "type": "shadowsocks", "tag": "SS Node", "server": "ss.example.com", "server_port": 8388, "method": "aes-256-gcm", "password": "hunter2" }
        ]
    });
    let outbounds = parse_subscription(&json.to_string()).unwrap().outbounds;
    assert_eq!(outbounds.len(), 2, "the direct outbound must be skipped");
    match &outbounds[0] {
        ParsedOutbound::Hysteria2(h) => {
            assert_eq!(h.name, "HY2 Node");
            assert_eq!(h.obfs.as_deref(), Some("salamander"));
        }
        other => panic!("expected Hysteria2, got {other:?}"),
    }
}

#[test]
fn singbox_json_rejects_a_missing_required_field() {
    let json = serde_json::json!({ "outbounds": [ { "type": "trojan", "server": "relay.example.com", "server_port": 443 } ] });
    let err = parse_subscription(&json.to_string()).unwrap_err();
    assert!(matches!(
        err,
        SubscriptionError::InvalidSingBoxOutbound { .. }
    ));
}

#[test]
fn empty_content_is_rejected() {
    assert!(matches!(
        parse_subscription(""),
        Err(SubscriptionError::Empty)
    ));
    assert!(matches!(
        parse_subscription("   \n  "),
        Err(SubscriptionError::Empty)
    ));
}

#[test]
fn completely_unrecognized_content_is_rejected() {
    let err =
        parse_subscription("<html><body>this is not a subscription</body></html>").unwrap_err();
    assert!(matches!(err, SubscriptionError::UnrecognizedFormat));
}

#[test]
fn garbage_that_happens_to_be_valid_base64_but_not_a_uri_list_is_rejected() {
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        "just some random plain text content here",
    );
    let err = parse_subscription(&encoded).unwrap_err();
    assert!(matches!(err, SubscriptionError::UnrecognizedFormat));
}
