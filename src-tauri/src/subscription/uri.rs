use base64::Engine;
use url::Url;

use super::error::SubscriptionError;
use super::model::{
    Hysteria2Outbound, ParsedOutbound, ShadowsocksOutbound, TrojanOutbound, TuicOutbound,
    VlessOutbound, VmessOutbound,
};

fn err(scheme: &str, reason: impl Into<String>) -> SubscriptionError {
    SubscriptionError::InvalidUri {
        scheme: scheme.to_string(),
        reason: reason.into(),
    }
}

pub(crate) fn decode_base64_flexible(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
    base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(trimmed))
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(trimmed))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(trimmed))
        .ok()
}

fn fragment_name(url: &Url, fallback: &str) -> String {
    url.fragment()
        .map(urlencoding_decode)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn urlencoding_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .into_owned()
}

fn query_map(url: &Url) -> std::collections::HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

/// Parses a single subscription line into a normalized outbound. Dispatches
/// on the URI scheme; unrecognized schemes are reported explicitly rather
/// than silently skipped, so a caller processing a whole list can decide
/// whether one bad line should fail the whole import.
pub fn parse_uri(line: &str) -> Result<ParsedOutbound, SubscriptionError> {
    let line = line.trim();
    let scheme = line
        .split("://")
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match scheme.as_str() {
        "vmess" => parse_vmess(line).map(ParsedOutbound::Vmess),
        "vless" => parse_vless(line).map(ParsedOutbound::Vless),
        "trojan" => parse_trojan(line).map(ParsedOutbound::Trojan),
        "ss" => parse_shadowsocks(line).map(ParsedOutbound::Shadowsocks),
        "hysteria2" | "hy2" => parse_hysteria2(line).map(ParsedOutbound::Hysteria2),
        "tuic" => parse_tuic(line).map(ParsedOutbound::Tuic),
        other => Err(SubscriptionError::UnsupportedScheme(other.to_string())),
    }
}

fn parse_vmess(line: &str) -> Result<VmessOutbound, SubscriptionError> {
    let payload = line
        .strip_prefix("vmess://")
        .ok_or_else(|| err("vmess", "missing vmess:// prefix"))?;
    let decoded = decode_base64_flexible(payload)
        .ok_or_else(|| err("vmess", "payload is not valid base64"))?;
    let json: serde_json::Value = serde_json::from_slice(&decoded)
        .map_err(|e| err("vmess", format!("payload is not valid JSON: {e}")))?;

    let get_str = |key: &str| -> Option<String> {
        match json.get(key) {
            Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
            Some(serde_json::Value::Number(n)) => Some(n.to_string()),
            _ => None,
        }
    };

    let server = get_str("add").ok_or_else(|| err("vmess", "missing \"add\" (server host)"))?;
    let port: u16 = get_str("port")
        .ok_or_else(|| err("vmess", "missing \"port\""))?
        .parse()
        .map_err(|_| err("vmess", "\"port\" is not a valid port number"))?;
    let uuid = get_str("id").ok_or_else(|| err("vmess", "missing \"id\" (uuid)"))?;

    Ok(VmessOutbound {
        name: get_str("ps").unwrap_or_else(|| format!("VMess {server}")),
        server,
        port,
        uuid,
        alter_id: get_str("aid").and_then(|v| v.parse().ok()).unwrap_or(0),
        security: get_str("scy").unwrap_or_else(|| "auto".to_string()),
        network: get_str("net").unwrap_or_else(|| "tcp".to_string()),
        tls: get_str("tls").map(|v| v == "tls").unwrap_or(false),
        sni: get_str("sni").or_else(|| get_str("host")),
        ws_path: get_str("path"),
        ws_host: get_str("host"),
    })
}

fn parse_vless(line: &str) -> Result<VlessOutbound, SubscriptionError> {
    let url = Url::parse(line).map_err(|e| err("vless", e.to_string()))?;
    let uuid = url.username();
    if uuid.is_empty() {
        return Err(err("vless", "missing uuid before @"));
    }
    let server = url
        .host_str()
        .ok_or_else(|| err("vless", "missing host"))?
        .to_string();
    let port = url.port().ok_or_else(|| err("vless", "missing port"))?;
    let params = query_map(&url);

    Ok(VlessOutbound {
        name: fragment_name(&url, &format!("VLESS {server}")),
        server,
        port,
        uuid: urlencoding_decode(uuid),
        flow: params.get("flow").cloned().filter(|s| !s.is_empty()),
        network: params
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
        tls: matches!(
            params.get("security").map(String::as_str),
            Some("tls") | Some("reality")
        ),
        sni: params.get("sni").cloned(),
        ws_path: params.get("path").cloned(),
        ws_host: params.get("host").cloned(),
        reality_public_key: params.get("pbk").cloned(),
        reality_short_id: params.get("sid").cloned(),
    })
}

fn parse_trojan(line: &str) -> Result<TrojanOutbound, SubscriptionError> {
    let url = Url::parse(line).map_err(|e| err("trojan", e.to_string()))?;
    let password = url.username();
    if password.is_empty() {
        return Err(err("trojan", "missing password before @"));
    }
    let server = url
        .host_str()
        .ok_or_else(|| err("trojan", "missing host"))?
        .to_string();
    let port = url.port().ok_or_else(|| err("trojan", "missing port"))?;
    let params = query_map(&url);

    Ok(TrojanOutbound {
        name: fragment_name(&url, &format!("Trojan {server}")),
        server,
        port,
        password: urlencoding_decode(password),
        sni: params.get("sni").cloned(),
        network: params
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
    })
}

fn parse_shadowsocks(line: &str) -> Result<ShadowsocksOutbound, SubscriptionError> {
    // SIP002 form: ss://base64(method:password)@host:port[/...][?...]# name
    if let Ok(url) = Url::parse(line) {
        if let Some(server) = url.host_str() {
            if let Some(port) = url.port() {
                let userinfo = url.username();
                if !userinfo.is_empty() {
                    // The URL parser percent-encodes '=' (and other sub-delims) in
                    // userinfo per the WHATWG userinfo percent-encode set, so the
                    // SIP002 base64 payload must be percent-decoded before it can
                    // be base64-decoded.
                    let userinfo_raw = urlencoding_decode(userinfo);
                    let decoded = decode_base64_flexible(&userinfo_raw)
                        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                        .ok_or_else(|| err("ss", "userinfo is not valid base64"))?;
                    let (method, password) = decoded
                        .split_once(':')
                        .ok_or_else(|| err("ss", "userinfo is not method:password"))?;
                    return Ok(ShadowsocksOutbound {
                        name: fragment_name(&url, &format!("Shadowsocks {server}")),
                        server: server.to_string(),
                        port,
                        method: method.to_string(),
                        password: password.to_string(),
                    });
                }
            }
        }
    }

    // Legacy form: ss://base64(method:password@host:port)#name
    let without_scheme = line
        .strip_prefix("ss://")
        .ok_or_else(|| err("ss", "missing ss:// prefix"))?;
    let (body, fragment) = without_scheme
        .split_once('#')
        .unwrap_or((without_scheme, ""));
    let decoded = decode_base64_flexible(body)
        .ok_or_else(|| err("ss", "payload is not valid base64 and not a SIP002 URI"))?;
    let decoded =
        String::from_utf8(decoded).map_err(|_| err("ss", "decoded payload is not valid UTF-8"))?;
    let (credentials, host_port) = decoded
        .split_once('@')
        .ok_or_else(|| err("ss", "decoded payload missing '@host:port'"))?;
    let (method, password) = credentials
        .split_once(':')
        .ok_or_else(|| err("ss", "decoded payload missing 'method:password'"))?;
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| err("ss", "decoded payload missing ':port'"))?;
    let port: u16 = port
        .parse()
        .map_err(|_| err("ss", "invalid port in decoded payload"))?;

    Ok(ShadowsocksOutbound {
        name: if fragment.is_empty() {
            format!("Shadowsocks {host}")
        } else {
            urlencoding_decode(fragment)
        },
        server: host.to_string(),
        port,
        method: method.to_string(),
        password: password.to_string(),
    })
}

fn parse_hysteria2(line: &str) -> Result<Hysteria2Outbound, SubscriptionError> {
    let url = Url::parse(line).map_err(|e| err("hysteria2", e.to_string()))?;
    let password = url.username();
    if password.is_empty() {
        return Err(err("hysteria2", "missing password before @"));
    }
    let server = url
        .host_str()
        .ok_or_else(|| err("hysteria2", "missing host"))?
        .to_string();
    let port = url.port().ok_or_else(|| err("hysteria2", "missing port"))?;
    let params = query_map(&url);

    Ok(Hysteria2Outbound {
        name: fragment_name(&url, &format!("Hysteria2 {server}")),
        server,
        port,
        password: urlencoding_decode(password),
        sni: params.get("sni").cloned(),
        insecure: matches!(
            params.get("insecure").map(String::as_str),
            Some("1") | Some("true")
        ),
        obfs: params.get("obfs").cloned(),
        obfs_password: params.get("obfs-password").cloned(),
    })
}

fn parse_tuic(line: &str) -> Result<TuicOutbound, SubscriptionError> {
    let url = Url::parse(line).map_err(|e| err("tuic", e.to_string()))?;
    let uuid = url.username();
    if uuid.is_empty() {
        return Err(err("tuic", "missing uuid before @"));
    }
    let password = url.password().unwrap_or_default();
    let server = url
        .host_str()
        .ok_or_else(|| err("tuic", "missing host"))?
        .to_string();
    let port = url.port().ok_or_else(|| err("tuic", "missing port"))?;
    let params = query_map(&url);

    Ok(TuicOutbound {
        name: fragment_name(&url, &format!("Tuic {server}")),
        server,
        port,
        uuid: urlencoding_decode(uuid),
        password: urlencoding_decode(password),
        sni: params.get("sni").cloned(),
        congestion_control: params.get("congestion_control").cloned(),
        alpn: params
            .get("alpn")
            .map(|v| v.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_vless_uri_with_reality_params() {
        let uri = "vless://11111111-2222-3333-4444-555555555555@example.com:443?encryption=none&security=reality&sni=cdn.example.com&type=tcp&flow=xtls-rprx-vision&pbk=abc123&sid=de#My%20Node";
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
}
