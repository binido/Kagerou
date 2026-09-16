
/// A link that carries only `host=` still expects that name in the TLS
/// handshake. Losing it makes sing-box offer the server address instead,
/// which is usually a bare IP and gets the connection rejected.
#[test]
fn vless_falls_back_to_host_when_there_is_no_sni() {
    let out = parse_uri("vless://uuid@1.2.3.4:443?security=tls&type=ws&host=example.com&path=/p#n")
        .unwrap();
    match out {
        ParsedOutbound::Vless(o) => {
            assert_eq!(o.sni.as_deref(), Some("example.com"));
            assert_eq!(o.ws_host.as_deref(), Some("example.com"));
        }
        other => panic!("expected vless, got {other:?}"),
    }
}

#[test]
fn an_explicit_sni_wins_over_host() {
    let out =
        parse_uri("vless://uuid@1.2.3.4:443?security=tls&sni=real.example&host=other.example#n")
            .unwrap();
    match out {
        ParsedOutbound::Vless(o) => assert_eq!(o.sni.as_deref(), Some("real.example")),
        other => panic!("expected vless, got {other:?}"),
    }
}

#[test]
fn trojan_takes_the_same_fallback() {
    let out = parse_uri("trojan://pass@1.2.3.4:443?type=ws&host=example.com#n").unwrap();
    match out {
        ParsedOutbound::Trojan(o) => assert_eq!(o.sni.as_deref(), Some("example.com")),
        other => panic!("expected trojan, got {other:?}"),
    }
}

#[test]
fn vless_keeps_the_grpc_service_name() {
    let out = parse_uri("vless://uuid@1.2.3.4:443?security=tls&type=grpc&serviceName=grpc-svc#n")
        .unwrap();
    match out {
        ParsedOutbound::Vless(o) => {
            assert_eq!(o.grpc_service_name.as_deref(), Some("grpc-svc"))
        }
        other => panic!("expected vless, got {other:?}"),
    }
}

#[test]
fn an_empty_sni_or_service_name_is_treated_as_absent() {
    let out =
        parse_uri("vless://uuid@1.2.3.4:443?security=tls&type=grpc&sni=&serviceName=#n").unwrap();
    match out {
        ParsedOutbound::Vless(o) => {
            assert_eq!(o.sni, None);
            assert_eq!(o.grpc_service_name, None);
        }
        other => panic!("expected vless, got {other:?}"),
    }
}
use super::*;

#[test]
fn parses_a_vless_uri_with_reality_params() {
    let uri = "vless://11111111-2222-3333-4444-555555555555@example.com:443?encryption=none&security=reality&sni=cdn.example.com&type=tcp&flow=xtls-rprx-vision&pbk=abc123&sid=de&fp=chrome#My%20Node";
    let outbound = parse_uri(uri).unwrap();
    match outbound {
        ParsedOutbound::Vless(v) => {
            assert_eq!(v.uuid, "11111111-2222-3333-4444-555555555555");
            assert_eq!(v.server, "example.com");
            assert_eq!(v.port, 443);
            assert_eq!(v.sni.as_deref(), Some("cdn.example.com"));
            assert_eq!(v.flow.as_deref(), Some("xtls-rprx-vision"));
            assert_eq!(v.reality_public_key.as_deref(), Some("abc123"));
            assert_eq!(v.reality_short_id.as_deref(), Some("de"));
            assert!(v.tls);
            assert_eq!(v.name, "My Node");
        }
        other => panic!("expected Vless, got {other:?}"),
    }
}

#[test]
fn vless_without_uuid_is_an_error() {
    let err = parse_uri("vless://@example.com:443").unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn vless_without_port_is_an_error() {
    let err = parse_uri("vless://uuid@example.com").unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn vless_falls_back_to_a_generated_name_without_a_fragment() {
    let outbound = parse_uri("vless://uuid@example.com:443").unwrap();
    assert_eq!(outbound.name(), "VLESS example.com");
}

#[test]
fn parses_a_vmess_uri() {
    let json = serde_json::json!({
        "v": "2", "ps": "Tokyo 01", "add": "jp.example.com", "port": "8443",
        "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", "aid": "0", "net": "ws",
        "type": "none", "host": "cdn.example.com", "path": "/ray", "tls": "tls", "scy": "auto",
    });
    let payload = base64::engine::general_purpose::STANDARD.encode(json.to_string());
    let outbound = parse_uri(&format!("vmess://{payload}")).unwrap();
    match outbound {
        ParsedOutbound::Vmess(v) => {
            assert_eq!(v.name, "Tokyo 01");
            assert_eq!(v.server, "jp.example.com");
            assert_eq!(v.port, 8443);
            assert_eq!(v.network, "ws");
            assert!(v.tls);
            assert_eq!(v.ws_path.as_deref(), Some("/ray"));
        }
        other => panic!("expected Vmess, got {other:?}"),
    }
}

#[test]
fn vmess_rejects_invalid_base64() {
    let err = parse_uri("vmess://not-base64-!!!@@@").unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn vmess_rejects_base64_that_is_not_json() {
    let payload = base64::engine::general_purpose::STANDARD.encode("just some text, not json");
    let err = parse_uri(&format!("vmess://{payload}")).unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn vmess_rejects_missing_required_fields() {
    let json = serde_json::json!({ "ps": "no server or port or id" });
    let payload = base64::engine::general_purpose::STANDARD.encode(json.to_string());
    let err = parse_uri(&format!("vmess://{payload}")).unwrap_err();
    assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
}

#[test]
fn parses_a_trojan_uri() {
    let outbound =
        parse_uri("trojan://s3cr3t@relay.example.com:443?sni=relay.example.com&type=tcp#Relay")
            .unwrap();
    match outbound {
        ParsedOutbound::Trojan(t) => {
            assert_eq!(t.password, "s3cr3t");
            assert_eq!(t.server, "relay.example.com");
            assert_eq!(t.sni.as_deref(), Some("relay.example.com"));
            assert_eq!(t.name, "Relay");
        }
        other => panic!("expected Trojan, got {other:?}"),
    }
}

#[test]
fn trojan_without_password_is_an_error() {
    assert!(parse_uri("trojan://@relay.example.com:443").is_err());
}

#[test]
fn parses_a_sip002_shadowsocks_uri() {
    let userinfo = base64::engine::general_purpose::STANDARD.encode("aes-256-gcm:hunter2");
    let uri = format!("ss://{userinfo}@ss.example.com:8388#SS%20Node");
    let outbound = parse_uri(&uri).unwrap();
    match outbound {
        ParsedOutbound::Shadowsocks(s) => {
            assert_eq!(s.method, "aes-256-gcm");
            assert_eq!(s.password, "hunter2");
            assert_eq!(s.server, "ss.example.com");
            assert_eq!(s.port, 8388);
            assert_eq!(s.name, "SS Node");
        }
        other => panic!("expected Shadowsocks, got {other:?}"),
    }
}

#[test]
fn parses_a_legacy_fully_encoded_shadowsocks_uri() {
    let payload = base64::engine::general_purpose::STANDARD
        .encode("chacha20-ietf-poly1305:p4ss@legacy.example.com:8989");
    let uri = format!("ss://{payload}#Legacy");
    let outbound = parse_uri(&uri).unwrap();
    match outbound {
        ParsedOutbound::Shadowsocks(s) => {
            assert_eq!(s.method, "chacha20-ietf-poly1305");
            assert_eq!(s.password, "p4ss");
            assert_eq!(s.server, "legacy.example.com");
            assert_eq!(s.port, 8989);
        }
        other => panic!("expected Shadowsocks, got {other:?}"),
    }
}

#[test]
fn shadowsocks_rejects_garbage() {
    assert!(parse_uri("ss://%%%not-valid%%%").is_err());
}

#[test]
fn parses_a_hysteria2_uri_and_its_hy2_alias() {
    let outbound = parse_uri("hysteria2://p4ss@hy.example.com:443?insecure=1&sni=hy.example.com&obfs=salamander&obfs-password=obfspw#HY2").unwrap();
    match outbound {
        ParsedOutbound::Hysteria2(h) => {
            assert_eq!(h.password, "p4ss");
            assert!(h.insecure);
            assert_eq!(h.obfs.as_deref(), Some("salamander"));
            assert_eq!(h.obfs_password.as_deref(), Some("obfspw"));
        }
        other => panic!("expected Hysteria2, got {other:?}"),
    }

    let alias = parse_uri("hy2://p4ss@hy.example.com:443").unwrap();
    assert!(matches!(alias, ParsedOutbound::Hysteria2(_)));
}

#[test]
fn hysteria2_without_password_is_an_error() {
    assert!(parse_uri("hysteria2://@hy.example.com:443").is_err());
}

#[test]
fn parses_a_tuic_uri() {
    let outbound = parse_uri("tuic://uuid-value:pw-value@tuic.example.com:443?congestion_control=bbr&alpn=h3,h3-29&sni=tuic.example.com#TUIC").unwrap();
    match outbound {
        ParsedOutbound::Tuic(t) => {
            assert_eq!(t.uuid, "uuid-value");
            assert_eq!(t.password, "pw-value");
            assert_eq!(t.congestion_control.as_deref(), Some("bbr"));
            assert_eq!(t.alpn, vec!["h3", "h3-29"]);
        }
        other => panic!("expected Tuic, got {other:?}"),
    }
}

#[test]
fn tuic_without_uuid_is_an_error() {
    assert!(parse_uri("tuic://:pw@tuic.example.com:443").is_err());
}

#[test]
fn unknown_scheme_is_reported_explicitly() {
    let err = parse_uri("wireguard://key@example.com:51820").unwrap_err();
    assert!(matches!(err, SubscriptionError::UnsupportedScheme(s) if s == "wireguard"));
}

#[test]
fn empty_and_malformed_lines_do_not_panic() {
    assert!(parse_uri("").is_err());
    assert!(parse_uri("not a uri at all").is_err());
    assert!(parse_uri("vless://").is_err());
}

/// `to_uri` is what gets stored as a profile key, and the config
/// generator re-parses that key, so anything dropped here is dropped
/// from the tunnel. Round-tripping every field the parser understands
/// is the only thing keeping that honest.
#[test]
fn to_uri_round_trips_every_parsed_field() {
    let uris = [
            "vless://11111111-2222-3333-4444-555555555555@example.com:443?encryption=none&security=reality&sni=cdn.example.com&type=tcp&flow=xtls-rprx-vision&pbk=abc123&sid=de&fp=chrome#My%20Node",
            "vless://uuid-1@ws.example.com:443?encryption=none&security=tls&type=ws&path=/ray&host=cdn.example.com&sni=cdn.example.com#WS",
            "vless://uuid-1@plain.example.com:80?encryption=none#Plain",
            "trojan://p%40ss@example.com:443?sni=cdn.example.com&type=tcp#Trojan%20Node",
            "ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#SS%20Node",
            "hysteria2://pw@example.com:443?sni=cdn.example.com&insecure=1&obfs=salamander&obfs-password=op#Hy2",
            "tuic://uuid-1:pw@example.com:443?sni=cdn.example.com&congestion_control=bbr&alpn=h3,spdy/3.1#Tuic",
        ];
    for uri in uris {
        let parsed = parse_uri(uri).unwrap();
        let reparsed = parse_uri(&to_uri(&parsed)).unwrap();
        assert_eq!(parsed, reparsed, "lost fields re-serializing {uri}");
    }
}

#[test]
fn to_uri_round_trips_vmess_including_transport() {
    let parsed = ParsedOutbound::Vmess(VmessOutbound {
        name: "VMess Node".into(),
        server: "example.com".into(),
        port: 443,
        uuid: "uuid-1".into(),
        alter_id: 4,
        security: "aes-128-gcm".into(),
        network: "ws".into(),
        tls: true,
        sni: Some("cdn.example.com".into()),
        ws_path: Some("/ray".into()),
        ws_host: Some("cdn.example.com".into()),
    });
    assert_eq!(parse_uri(&to_uri(&parsed)).unwrap(), parsed);
}
