use super::error::SubscriptionError;
use super::model::{
    Hysteria2Outbound, ParsedOutbound, ShadowsocksOutbound, TrojanOutbound, TuicOutbound,
    VlessOutbound, VmessOutbound,
};
use super::uri::{decode_base64_flexible, parse_uri};

const KNOWN_SCHEMES: &[&str] = &[
    "vmess://",
    "vless://",
    "trojan://",
    "ss://",
    "hysteria2://",
    "hy2://",
    "tuic://",
];

fn looks_like_uri_list(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|first| {
            KNOWN_SCHEMES
                .iter()
                .any(|scheme| first.to_ascii_lowercase().starts_with(scheme))
        })
        .unwrap_or(false)
}

fn parse_uri_list(text: &str) -> Result<Vec<ParsedOutbound>, SubscriptionError> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(parse_uri)
        .collect()
}

/// Parses subscription content of any recognized shape: a plain or
/// base64-encoded newline list of proxy URIs, a Clash YAML document
/// (`proxies:`), or a sing-box JSON config (`outbounds`).
pub fn parse_subscription(content: &str) -> Result<Vec<ParsedOutbound>, SubscriptionError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(SubscriptionError::Empty);
    }

    if looks_like_uri_list(trimmed) {
        return parse_uri_list(trimmed);
    }

    if let Some(result) = try_parse_singbox_json(trimmed) {
        return result;
    }

    if let Some(result) = try_parse_clash_yaml(trimmed) {
        return result;
    }

    if let Some(decoded) = decode_base64_flexible(trimmed) {
        if let Ok(text) = String::from_utf8(decoded) {
            if looks_like_uri_list(&text) {
                return parse_uri_list(&text);
            }
        }
    }

    Err(SubscriptionError::UnrecognizedFormat)
}

fn try_parse_singbox_json(trimmed: &str) -> Option<Result<Vec<ParsedOutbound>, SubscriptionError>> {
    if !trimmed.starts_with('{') {
        return None;
    }
    let json: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let outbounds = json.get("outbounds")?.as_array()?;

    Some(
        outbounds
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                !matches!(
                    entry.get("type").and_then(|t| t.as_str()),
                    Some("direct")
                        | Some("block")
                        | Some("dns")
                        | Some("selector")
                        | Some("urltest")
                )
            })
            .map(|(index, entry)| convert_singbox_outbound(index, entry))
            .collect(),
    )
}

fn json_str(value: &serde_json::Value, key: &str) -> Option<String> {
    match value.get(key)? {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn json_bool(value: &serde_json::Value, key: &str) -> bool {
    value.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn convert_singbox_outbound(
    index: usize,
    entry: &serde_json::Value,
) -> Result<ParsedOutbound, SubscriptionError> {
    let fail = |reason: &str| SubscriptionError::InvalidSingBoxOutbound {
        index,
        reason: reason.to_string(),
    };
    let kind = json_str(entry, "type").ok_or_else(|| fail("missing \"type\""))?;
    let server = json_str(entry, "server").ok_or_else(|| fail("missing \"server\""))?;
    let port: u16 = json_str(entry, "server_port")
        .ok_or_else(|| fail("missing \"server_port\""))?
        .parse()
        .map_err(|_| fail("\"server_port\" is not a valid port number"))?;
    let name = json_str(entry, "tag").unwrap_or_else(|| format!("{kind} {server}"));
    let tls = entry.get("tls").cloned().unwrap_or_default();
    let tls_enabled = json_bool(&tls, "enabled");
    let sni = json_str(&tls, "server_name");
    let transport = entry.get("transport").cloned().unwrap_or_default();
    let network = json_str(&transport, "type").unwrap_or_else(|| "tcp".to_string());
    let ws_path = json_str(&transport, "path");
    let ws_host = transport
        .get("headers")
        .and_then(|h| h.get("Host"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    match kind.as_str() {
        "vless" => {
            let reality = tls.get("reality").cloned().unwrap_or_default();
            Ok(ParsedOutbound::Vless(VlessOutbound {
                name,
                server,
                port,
                uuid: json_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
                flow: json_str(entry, "flow"),
                network,
                tls: tls_enabled,
                sni,
                ws_path,
                ws_host,
                grpc_service_name: json_str(&transport, "service_name"),
                reality_public_key: json_str(&reality, "public_key"),
                reality_short_id: json_str(&reality, "short_id"),
                fingerprint: tls.get("utls").and_then(|u| json_str(u, "fingerprint")),
            }))
        }
        "vmess" => Ok(ParsedOutbound::Vmess(VmessOutbound {
            name,
            server,
            port,
            uuid: json_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
            alter_id: json_str(entry, "alter_id")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            security: json_str(entry, "security").unwrap_or_else(|| "auto".to_string()),
            network,
            tls: tls_enabled,
            sni,
            ws_path,
            ws_host,
        })),
        "trojan" => Ok(ParsedOutbound::Trojan(TrojanOutbound {
            name,
            server,
            port,
            password: json_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
            sni,
            network,
        })),
        "shadowsocks" => Ok(ParsedOutbound::Shadowsocks(ShadowsocksOutbound {
            name,
            server,
            port,
            method: json_str(entry, "method").ok_or_else(|| fail("missing \"method\""))?,
            password: json_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
        })),
        "hysteria2" => {
            let obfs = entry.get("obfs").cloned().unwrap_or_default();
            Ok(ParsedOutbound::Hysteria2(Hysteria2Outbound {
                name,
                server,
                port,
                password: json_str(entry, "password")
                    .ok_or_else(|| fail("missing \"password\""))?,
                sni,
                insecure: json_bool(&tls, "insecure"),
                obfs: json_str(&obfs, "type"),
                obfs_password: json_str(&obfs, "password"),
            }))
        }
        "tuic" => Ok(ParsedOutbound::Tuic(TuicOutbound {
            name,
            server,
            port,
            uuid: json_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
            password: json_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
            sni,
            congestion_control: json_str(entry, "congestion_control"),
            alpn: tls
                .get("alpn")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        })),
        other => Err(fail(&format!("unsupported outbound type \"{other}\""))),
    }
}

fn try_parse_clash_yaml(trimmed: &str) -> Option<Result<Vec<ParsedOutbound>, SubscriptionError>> {
    let doc: serde_yaml::Value = serde_yaml::from_str(trimmed).ok()?;
    let proxies = doc.get("proxies")?.as_sequence()?;

    Some(
        proxies
            .iter()
            .enumerate()
            .map(|(index, entry)| convert_clash_proxy(index, entry))
            .collect(),
    )
}

fn yaml_str(value: &serde_yaml::Value, key: &str) -> Option<String> {
    match value.get(key)? {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn yaml_bool(value: &serde_yaml::Value, key: &str) -> bool {
    value.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn convert_clash_proxy(
    index: usize,
    entry: &serde_yaml::Value,
) -> Result<ParsedOutbound, SubscriptionError> {
    let fail = |reason: &str| SubscriptionError::InvalidClashProxy {
        index,
        reason: reason.to_string(),
    };
    let kind = yaml_str(entry, "type").ok_or_else(|| fail("missing \"type\""))?;
    let server = yaml_str(entry, "server").ok_or_else(|| fail("missing \"server\""))?;
    let port: u16 = yaml_str(entry, "port")
        .ok_or_else(|| fail("missing \"port\""))?
        .parse()
        .map_err(|_| fail("\"port\" is not a valid port number"))?;
    let name = yaml_str(entry, "name").unwrap_or_else(|| format!("{kind} {server}"));
    let sni = yaml_str(entry, "servername").or_else(|| yaml_str(entry, "sni"));
    let network = yaml_str(entry, "network").unwrap_or_else(|| "tcp".to_string());
    let ws_opts = entry
        .get("ws-opts")
        .cloned()
        .unwrap_or(serde_yaml::Value::Null);
    let ws_path = yaml_str(&ws_opts, "path");
    let ws_host = ws_opts
        .get("headers")
        .and_then(|h| h.get("Host"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    match kind.as_str() {
        "vless" => {
            let reality_opts = entry
                .get("reality-opts")
                .cloned()
                .unwrap_or(serde_yaml::Value::Null);
            Ok(ParsedOutbound::Vless(VlessOutbound {
                name,
                server,
                port,
                uuid: yaml_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
                flow: yaml_str(entry, "flow"),
                network,
                tls: yaml_bool(entry, "tls"),
                sni,
                ws_path,
                ws_host,
                grpc_service_name: entry
                    .get("grpc-opts")
                    .and_then(|o| yaml_str(o, "grpc-service-name")),
                reality_public_key: yaml_str(&reality_opts, "public-key"),
                reality_short_id: yaml_str(&reality_opts, "short-id"),
                fingerprint: yaml_str(entry, "client-fingerprint"),
            }))
        }
        "vmess" => Ok(ParsedOutbound::Vmess(VmessOutbound {
            name,
            server,
            port,
            uuid: yaml_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
            alter_id: yaml_str(entry, "alterId")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            security: yaml_str(entry, "cipher").unwrap_or_else(|| "auto".to_string()),
            network,
            tls: yaml_bool(entry, "tls"),
            sni,
            ws_path,
            ws_host,
        })),
        "trojan" => Ok(ParsedOutbound::Trojan(TrojanOutbound {
            name,
            server,
            port,
            password: yaml_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
            sni,
            network,
        })),
        "ss" => Ok(ParsedOutbound::Shadowsocks(ShadowsocksOutbound {
            name,
            server,
            port,
            method: yaml_str(entry, "cipher").ok_or_else(|| fail("missing \"cipher\""))?,
            password: yaml_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
        })),
        "hysteria2" => Ok(ParsedOutbound::Hysteria2(Hysteria2Outbound {
            name,
            server,
            port,
            password: yaml_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
            sni,
            insecure: yaml_bool(entry, "insecure") || yaml_bool(entry, "skip-cert-verify"),
            obfs: yaml_str(entry, "obfs"),
            obfs_password: yaml_str(entry, "obfs-password"),
        })),
        "tuic" => Ok(ParsedOutbound::Tuic(TuicOutbound {
            name,
            server,
            port,
            uuid: yaml_str(entry, "uuid").ok_or_else(|| fail("missing \"uuid\""))?,
            password: yaml_str(entry, "password").ok_or_else(|| fail("missing \"password\""))?,
            sni,
            congestion_control: yaml_str(entry, "congestion-controller"),
            alpn: entry
                .get("alpn")
                .and_then(|v| v.as_sequence())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        })),
        other => Err(fail(&format!("unsupported proxy type \"{other}\""))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_newline_uri_list() {
        let content = "vless://uuid@example.com:443#A\ntrojan://pw@relay.example.com:443#B\n";
        let outbounds = parse_subscription(content).unwrap();
        assert_eq!(outbounds.len(), 2);
        assert_eq!(outbounds[0].protocol_label(), "VLESS");
        assert_eq!(outbounds[1].protocol_label(), "Trojan");
    }

    #[test]
    fn parses_a_base64_encoded_uri_list() {
        let raw = "vless://uuid@example.com:443#A\nss://YWVzLTI1Ni1nY206aHVudGVyMg==@ss.example.com:8388#B";
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, raw);
        let outbounds = parse_subscription(&encoded).unwrap();
        assert_eq!(outbounds.len(), 2);
    }

    #[test]
    fn skips_blank_lines_in_a_uri_list() {
        let content = "\n\nvless://uuid@example.com:443#A\n\n\n";
        let outbounds = parse_subscription(content).unwrap();
        assert_eq!(outbounds.len(), 1);
    }

    #[test]
    fn a_single_bad_uri_fails_the_whole_list() {
        let content = "vless://uuid@example.com:443#A\nvless://missing-port@example.com#B";
        let err = parse_subscription(content).unwrap_err();
        assert!(matches!(err, SubscriptionError::InvalidUri { .. }));
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
        let outbounds = parse_subscription(yaml).unwrap();
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
    fn clash_yaml_rejects_an_unsupported_proxy_type() {
        let yaml = "proxies:\n  - name: Unsupported\n    type: snell\n    server: s.example.com\n    port: 1\n";
        let err = parse_subscription(yaml).unwrap_err();
        assert!(matches!(err, SubscriptionError::InvalidClashProxy { .. }));
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
        let outbounds = parse_subscription(&json.to_string()).unwrap();
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
}
