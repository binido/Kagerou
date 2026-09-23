use serde_json::Value;

use super::error::SubscriptionError;
use super::formats::{json_bool, json_str, Entries};
use super::model::{
    Hysteria2Outbound, ParsedOutbound, ShadowsocksOutbound, TrojanOutbound, VlessOutbound,
    VmessOutbound,
};

const NON_PROXY_PROTOCOLS: &[&str] = &["freedom", "blackhole", "dns", "loopback"];

/// Reads Xray JSON: a single config, or the array of configs panels serve to
/// Xray clients, one config per server the user picks from.
pub fn try_parse_xray_json(trimmed: &str) -> Option<Entries> {
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return None;
    }
    let configs = match serde_json::from_str::<Value>(trimmed).ok()? {
        Value::Array(configs) => configs,
        config => vec![config],
    };
    // sing-box names the protocol `type`, Xray names it `protocol`.
    let is_xray = |config: &Value| {
        config
            .get("outbounds")
            .and_then(Value::as_array)
            .is_some_and(|outbounds| outbounds.iter().any(|o| o.get("protocol").is_some()))
    };
    if configs.is_empty() || !configs.iter().all(is_xray) {
        return None;
    }
    Some(
        configs
            .iter()
            .enumerate()
            .map(|(index, config)| convert_config(index, config))
            .collect(),
    )
}

fn convert_config(index: usize, config: &Value) -> Result<ParsedOutbound, SubscriptionError> {
    let fail = |reason: &str| SubscriptionError::InvalidXrayConfig {
        index,
        reason: reason.to_string(),
    };
    let routing = config.get("routing").cloned().unwrap_or_default();
    let uses_balancer = routing
        .get("rules")
        .and_then(Value::as_array)
        .is_some_and(|rules| rules.iter().any(|rule| rule.get("balancerTag").is_some()));
    if uses_balancer {
        return Err(SubscriptionError::Balancer);
    }

    let proxies: Vec<&Value> = config
        .get("outbounds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|o| {
            json_str(o, "protocol").is_some_and(|p| !NON_PROXY_PROTOCOLS.contains(&p.as_str()))
        })
        .collect();
    let outbound = proxies
        .iter()
        .find(|o| json_str(o, "tag").as_deref() == Some("proxy"))
        .or(proxies.first())
        .ok_or_else(|| fail("no proxy outbound"))?;

    let stream = outbound.get("streamSettings").cloned().unwrap_or_default();
    let is_chained = outbound.get("proxySettings").is_some()
        || stream
            .get("sockopt")
            .and_then(|s| s.get("dialerProxy"))
            .is_some();
    if is_chained {
        return Err(SubscriptionError::Chain);
    }
    convert_outbound(outbound, &stream, json_str(config, "remarks"), fail)
}

fn convert_outbound(
    outbound: &Value,
    stream: &Value,
    remarks: Option<String>,
    fail: impl Fn(&str) -> SubscriptionError,
) -> Result<ParsedOutbound, SubscriptionError> {
    let protocol = json_str(outbound, "protocol").unwrap_or_default();
    let settings = outbound.get("settings").cloned().unwrap_or_default();
    // Older configs nest the server in `vnext` or `servers`, newer ones put
    // its fields straight into `settings`.
    let server_block = settings
        .get("vnext")
        .or_else(|| settings.get("servers"))
        .and_then(|list| list.get(0))
        .cloned()
        .unwrap_or_else(|| settings.clone());
    let user = server_block
        .get("users")
        .and_then(|users| users.get(0))
        .cloned()
        .unwrap_or_else(|| server_block.clone());

    let server = json_str(&server_block, "address").ok_or_else(|| fail("missing \"address\""))?;
    let port: u16 = json_str(&server_block, "port")
        .ok_or_else(|| fail("missing \"port\""))?
        .parse()
        .map_err(|_| fail("\"port\" is not a valid port number"))?;
    let name = remarks.unwrap_or_else(|| format!("{protocol} {server}"));

    let network = json_str(stream, "network").unwrap_or_else(|| "tcp".to_string());
    let security = json_str(stream, "security").unwrap_or_default();
    let tls = security == "tls" || security == "reality";
    let tls_settings = stream
        .get(if security == "reality" {
            "realitySettings"
        } else {
            "tlsSettings"
        })
        .cloned()
        .unwrap_or_default();
    let sni = json_str(&tls_settings, "serverName").filter(|s| !s.is_empty());
    let ws = stream.get("wsSettings").cloned().unwrap_or_default();
    let ws_path = json_str(&ws, "path");
    let ws_host = json_str(&ws, "host")
        .filter(|h| !h.is_empty())
        .or_else(|| ws.get("headers").and_then(|h| json_str(h, "Host")));
    let secret =
        |key: &str| json_str(&user, key).ok_or_else(|| fail(&format!("missing \"{key}\"")));

    match protocol.as_str() {
        "vless" => Ok(ParsedOutbound::Vless(VlessOutbound {
            name,
            server,
            port,
            uuid: secret("id")?,
            flow: json_str(&user, "flow").filter(|f| !f.is_empty()),
            network,
            tls,
            sni,
            ws_path,
            ws_host,
            grpc_service_name: stream
                .get("grpcSettings")
                .and_then(|g| json_str(g, "serviceName")),
            reality_public_key: json_str(&tls_settings, "publicKey"),
            reality_short_id: json_str(&tls_settings, "shortId"),
            fingerprint: json_str(&tls_settings, "fingerprint"),
        })),
        "vmess" => Ok(ParsedOutbound::Vmess(VmessOutbound {
            name,
            server,
            port,
            uuid: secret("id")?,
            alter_id: json_str(&user, "alterId")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            security: json_str(&user, "security").unwrap_or_else(|| "auto".to_string()),
            network,
            tls,
            sni,
            ws_path,
            ws_host,
        })),
        "trojan" => Ok(ParsedOutbound::Trojan(TrojanOutbound {
            name,
            server,
            port,
            password: secret("password")?,
            sni,
            network,
        })),
        "shadowsocks" => Ok(ParsedOutbound::Shadowsocks(ShadowsocksOutbound {
            name,
            server,
            port,
            method: secret("method")?,
            password: secret("password")?,
        })),
        "hysteria" => {
            let hysteria = stream.get("hysteriaSettings").cloned().unwrap_or_default();
            let version = json_str(&settings, "version").or_else(|| json_str(&hysteria, "version"));
            if version.as_deref() != Some("2") {
                return Err(SubscriptionError::UnsupportedProtocol(
                    "hysteria".to_string(),
                ));
            }
            Ok(ParsedOutbound::Hysteria2(Hysteria2Outbound {
                name,
                server,
                port,
                password: json_str(&hysteria, "auth").ok_or_else(|| fail("missing \"auth\""))?,
                sni,
                insecure: json_bool(&tls_settings, "allowInsecure"),
                obfs: None,
                obfs_password: None,
            }))
        }
        other => Err(SubscriptionError::UnsupportedProtocol(other.to_string())),
    }
}

#[cfg(test)]
mod tests;
