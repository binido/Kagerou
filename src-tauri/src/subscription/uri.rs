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
        // `host` is the fallback SNI, as it already is for VMess above:
        // a link that carries only `host=` still expects that name in the
        // TLS handshake, and without it sing-box sends the server address —
        // an IP, usually — which the server rejects.
        sni: params
            .get("sni")
            .or_else(|| params.get("host"))
            .cloned()
            .filter(|s| !s.is_empty()),
        ws_path: params.get("path").cloned(),
        ws_host: params.get("host").cloned(),
        grpc_service_name: params.get("serviceName").cloned().filter(|s| !s.is_empty()),
        reality_public_key: params.get("pbk").cloned(),
        reality_short_id: params.get("sid").cloned(),
        fingerprint: params.get("fp").cloned(),
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
        sni: params
            .get("sni")
            .or_else(|| params.get("host"))
            .cloned()
            .filter(|s| !s.is_empty()),
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
        sni: params
            .get("sni")
            .or_else(|| params.get("host"))
            .cloned()
            .filter(|s| !s.is_empty()),
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
        sni: params
            .get("sni")
            .or_else(|| params.get("host"))
            .cloned()
            .filter(|s| !s.is_empty()),
        congestion_control: params.get("congestion_control").cloned(),
        alpn: params
            .get("alpn")
            .map(|v| v.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
    })
}

// --- serialization -------------------------------------------------------

/// Fragment percent-encode set, plus `%` so an already-encoded name is not
/// double-decoded on the way back in.
const FRAGMENT: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'#')
    .add(b'%');

/// Userinfo percent-encode set: everything that would otherwise end the
/// userinfo component or split it into user/password.
const USERINFO: &percent_encoding::AsciiSet = &FRAGMENT
    .add(b'/')
    .add(b'?')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'{')
    .add(b'}');

fn enc(value: &str, set: &'static percent_encoding::AsciiSet) -> String {
    percent_encoding::utf8_percent_encode(value, set).to_string()
}

fn query(pairs: Vec<(&str, String)>) -> String {
    let filtered: Vec<_> = pairs.into_iter().filter(|(_, v)| !v.is_empty()).collect();
    if filtered.is_empty() {
        return String::new();
    }
    format!(
        "?{}",
        url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(filtered)
            .finish()
    )
}

fn opt(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

/// Serializes a parsed outbound back into a subscription URI, carrying
/// every field `parse_uri` understands — TLS/REALITY, flow, SNI, transport
/// — so `parse_uri(to_uri(x)) == x`. This is what gets stored as a
/// profile's key, and `singbox::config::generate` re-parses it to build the
/// outbound, so anything dropped here is silently dropped from the tunnel.
pub fn to_uri(outbound: &ParsedOutbound) -> String {
    match outbound {
        ParsedOutbound::Vless(o) => {
            let security = if o.tls {
                if o.reality_public_key.is_some() {
                    "reality"
                } else {
                    "tls"
                }
            } else {
                ""
            };
            format!(
                "vless://{}@{}:{}{}#{}",
                enc(&o.uuid, USERINFO),
                o.server,
                o.port,
                query(vec![
                    ("encryption", "none".to_string()),
                    ("security", security.to_string()),
                    ("type", o.network.clone()),
                    ("flow", opt(&o.flow)),
                    ("sni", opt(&o.sni)),
                    ("path", opt(&o.ws_path)),
                    ("host", opt(&o.ws_host)),
                    ("pbk", opt(&o.reality_public_key)),
                    ("sid", opt(&o.reality_short_id)),
                    ("fp", opt(&o.fingerprint)),
                ]),
                enc(&o.name, FRAGMENT),
            )
        }
        ParsedOutbound::Vmess(o) => {
            use base64::Engine;
            let mut json = serde_json::json!({
                "v": "2", "ps": o.name, "add": o.server, "port": o.port.to_string(),
                "id": o.uuid, "aid": o.alter_id.to_string(), "net": o.network,
                "tls": if o.tls { "tls" } else { "" },
            });
            for (key, value) in [
                ("scy", o.security.clone()),
                ("sni", opt(&o.sni)),
                ("path", opt(&o.ws_path)),
                ("host", opt(&o.ws_host)),
            ] {
                if !value.is_empty() {
                    json[key] = serde_json::json!(value);
                }
            }
            format!(
                "vmess://{}",
                base64::engine::general_purpose::STANDARD.encode(json.to_string())
            )
        }
        ParsedOutbound::Trojan(o) => format!(
            "trojan://{}@{}:{}{}#{}",
            enc(&o.password, USERINFO),
            o.server,
            o.port,
            query(vec![("sni", opt(&o.sni)), ("type", o.network.clone()),]),
            enc(&o.name, FRAGMENT),
        ),
        ParsedOutbound::Shadowsocks(o) => {
            use base64::Engine;
            // SIP002. Unpadded so the `=` padding does not have to survive a
            // round trip through the URL parser's userinfo encoding.
            let userinfo = base64::engine::general_purpose::STANDARD_NO_PAD
                .encode(format!("{}:{}", o.method, o.password));
            format!(
                "ss://{}@{}:{}#{}",
                userinfo,
                o.server,
                o.port,
                enc(&o.name, FRAGMENT)
            )
        }
        ParsedOutbound::Hysteria2(o) => format!(
            "hysteria2://{}@{}:{}{}#{}",
            enc(&o.password, USERINFO),
            o.server,
            o.port,
            query(vec![
                ("sni", opt(&o.sni)),
                (
                    "insecure",
                    if o.insecure {
                        "1".to_string()
                    } else {
                        String::new()
                    }
                ),
                ("obfs", opt(&o.obfs)),
                ("obfs-password", opt(&o.obfs_password)),
            ]),
            enc(&o.name, FRAGMENT),
        ),
        ParsedOutbound::Tuic(o) => format!(
            "tuic://{}:{}@{}:{}{}#{}",
            enc(&o.uuid, USERINFO),
            enc(&o.password, USERINFO),
            o.server,
            o.port,
            query(vec![
                ("sni", opt(&o.sni)),
                ("congestion_control", opt(&o.congestion_control)),
                ("alpn", o.alpn.join(",")),
            ]),
            enc(&o.name, FRAGMENT),
        ),
    }
}

#[cfg(test)]
mod tests;
