use serde::Serialize;

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

/// A subscription entry left out of the import, and why.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "name", rename_all = "camelCase")]
pub enum Unsupported {
    Protocol(String),
    Transport(String),
    Invalid,
}

impl From<&SubscriptionError> for Unsupported {
    fn from(error: &SubscriptionError) -> Self {
        match error {
            SubscriptionError::UnsupportedScheme(name)
            | SubscriptionError::UnsupportedProtocol(name) => Self::Protocol(name.clone()),
            SubscriptionError::UnsupportedTransport(name) => Self::Transport(name.clone()),
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Parsed {
    pub outbounds: Vec<ParsedOutbound>,
    pub unsupported: Vec<Unsupported>,
}

type Entries = Vec<Result<ParsedOutbound, SubscriptionError>>;

/// Transports sing-box has, by the names subscriptions use for them.
const KNOWN_NETWORKS: &[&str] = &["tcp", "ws", "grpc", "http", "httpupgrade", "quic"];

/// Rejects a transport sing-box does not have, so it never reaches the
/// generated config, where it would stop the core from starting.
fn check_transport(mut outbound: ParsedOutbound) -> Result<ParsedOutbound, SubscriptionError> {
    let network = match &mut outbound {
        ParsedOutbound::Vless(o) => &mut o.network,
        ParsedOutbound::Vmess(o) => &mut o.network,
        ParsedOutbound::Trojan(o) => &mut o.network,
        _ => return Ok(outbound),
    };
    // Xray renamed plain TCP to `raw`.
    if network.is_empty() || network == "raw" {
        *network = "tcp".to_string();
    }
    if !KNOWN_NETWORKS.contains(&network.as_str()) {
        return Err(SubscriptionError::UnsupportedTransport(network.clone()));
    }
    Ok(outbound)
}

fn looks_like_uri_list(text: &str) -> bool {
    text.lines().map(str::trim).any(|line| {
        KNOWN_SCHEMES
            .iter()
            .any(|scheme| line.to_ascii_lowercase().starts_with(scheme))
    })
}

fn parse_uri_list(text: &str) -> Entries {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(parse_uri)
        .collect()
}

/// Parses subscription content of any recognized shape: a plain or
/// base64-encoded newline list of proxy URIs, a Clash YAML document
/// (`proxies:`), or a sing-box JSON config (`outbounds`).
///
/// Entries this app cannot run are left out and listed in `unsupported`.
/// When nothing is left, the first entry's error is returned, so a single
/// bad key still says what is wrong with it.
pub fn parse_subscription(content: &str) -> Result<Parsed, SubscriptionError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(SubscriptionError::Empty);
    }
    let entries = recognize(trimmed).ok_or(SubscriptionError::UnrecognizedFormat)?;

    let mut parsed = Parsed::default();
    let mut first_error = None;
    for entry in entries {
        match entry.and_then(check_transport) {
            Ok(outbound) => parsed.outbounds.push(outbound),
            Err(error) => {
                parsed.unsupported.push(Unsupported::from(&error));
                first_error.get_or_insert(error);
            }
        }
    }
    if parsed.outbounds.is_empty() {
        return Err(first_error.unwrap_or(SubscriptionError::Empty));
    }
    Ok(parsed)
}

fn recognize(trimmed: &str) -> Option<Entries> {
    if looks_like_uri_list(trimmed) {
        return Some(parse_uri_list(trimmed));
    }
    if let Some(entries) = try_parse_singbox_json(trimmed) {
        return Some(entries);
    }
    if let Some(entries) = try_parse_clash_yaml(trimmed) {
        return Some(entries);
    }
    let text = String::from_utf8(decode_base64_flexible(trimmed)?).ok()?;
    looks_like_uri_list(&text).then(|| parse_uri_list(&text))
}

fn try_parse_singbox_json(trimmed: &str) -> Option<Entries> {
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
        other => Err(SubscriptionError::UnsupportedProtocol(other.to_string())),
    }
}

fn try_parse_clash_yaml(trimmed: &str) -> Option<Entries> {
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
        other => Err(SubscriptionError::UnsupportedProtocol(other.to_string())),
    }
}

#[cfg(test)]
mod tests;
