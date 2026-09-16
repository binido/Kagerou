use serde_json::{json, Value};

use crate::subscription::model::ParsedOutbound;

/// `None` for a plaintext outbound: sing-box 1.14 still builds a TLS
/// dialer from a present-but-disabled `tls` object and then panics on the
/// nil client config, so the key has to be absent rather than `enabled:
/// false`.
fn tls_block(enabled: bool, sni: Option<&str>, insecure: bool) -> Option<Value> {
    if !enabled {
        return None;
    }
    let mut tls = json!({ "enabled": true });
    if let Some(sni) = sni {
        tls["server_name"] = json!(sni);
    }
    if insecure {
        tls["insecure"] = json!(true);
    }
    Some(tls)
}

fn transport_block(
    network: &str,
    ws_path: Option<&str>,
    ws_host: Option<&str>,
    grpc_service_name: Option<&str>,
) -> Option<Value> {
    if network == "tcp" || network.is_empty() {
        return None;
    }
    let mut transport = json!({ "type": network });
    if network == "grpc" {
        if let Some(service) = grpc_service_name {
            transport["service_name"] = json!(service);
        }
    }
    if network == "ws" {
        if let Some(path) = ws_path {
            transport["path"] = json!(path);
        }
        if let Some(host) = ws_host {
            transport["headers"] = json!({ "Host": host });
        }
    }
    Some(transport)
}

/// Converts a parsed proxy outbound into a sing-box outbound config object,
/// tagged with `tag` so it can be referenced from `route.rules` and the
/// `selector` outbound.
pub fn to_singbox_outbound(parsed: &ParsedOutbound, tag: &str) -> Value {
    match parsed {
        ParsedOutbound::Vless(o) => {
            let mut value = json!({
                "type": "vless", "tag": tag, "server": o.server, "server_port": o.port,
                "uuid": o.uuid,
            });
            if let Some(mut tls) = tls_block(o.tls, o.sni.as_deref(), false) {
                if let Some(pbk) = &o.reality_public_key {
                    let mut reality = json!({ "enabled": true, "public_key": pbk });
                    if let Some(sid) = &o.reality_short_id {
                        reality["short_id"] = json!(sid);
                    }
                    tls["reality"] = reality;
                }
                // sing-box rejects a REALITY client that has no uTLS
                // fingerprint, and subscriptions routinely omit `fp=`;
                // chrome is what every other client defaults to.
                if o.reality_public_key.is_some() || o.fingerprint.is_some() {
                    tls["utls"] = json!({
                        "enabled": true,
                        "fingerprint": o.fingerprint.clone().unwrap_or_else(|| "chrome".into()),
                    });
                }
                value["tls"] = tls;
            }
            if let Some(flow) = &o.flow {
                value["flow"] = json!(flow);
            }
            if let Some(transport) = transport_block(
                &o.network,
                o.ws_path.as_deref(),
                o.ws_host.as_deref(),
                o.grpc_service_name.as_deref(),
            ) {
                value["transport"] = transport;
            }
            value
        }
        ParsedOutbound::Vmess(o) => {
            let mut value = json!({
                "type": "vmess", "tag": tag, "server": o.server, "server_port": o.port,
                "uuid": o.uuid, "alter_id": o.alter_id, "security": o.security,
            });
            if let Some(tls) = tls_block(o.tls, o.sni.as_deref(), false) {
                value["tls"] = tls;
            }
            if let Some(transport) =
                transport_block(&o.network, o.ws_path.as_deref(), o.ws_host.as_deref(), None)
            {
                value["transport"] = transport;
            }
            value
        }
        ParsedOutbound::Trojan(o) => {
            let mut value = json!({
                "type": "trojan", "tag": tag, "server": o.server, "server_port": o.port,
                "password": o.password, "tls": tls_block(true, o.sni.as_deref(), false).unwrap(),
            });
            if let Some(transport) = transport_block(&o.network, None, None, None) {
                value["transport"] = transport;
            }
            value
        }
        ParsedOutbound::Shadowsocks(o) => json!({
            "type": "shadowsocks", "tag": tag, "server": o.server, "server_port": o.port,
            "method": o.method, "password": o.password,
        }),
        ParsedOutbound::Hysteria2(o) => {
            let mut value = json!({
                "type": "hysteria2", "tag": tag, "server": o.server, "server_port": o.port,
                "password": o.password, "tls": tls_block(true, o.sni.as_deref(), o.insecure).unwrap(),
            });
            if o.obfs.is_some() || o.obfs_password.is_some() {
                value["obfs"] = json!({ "type": o.obfs.clone().unwrap_or_default(), "password": o.obfs_password.clone().unwrap_or_default() });
            }
            value
        }
        ParsedOutbound::Tuic(o) => {
            let mut tls = tls_block(true, o.sni.as_deref(), false).unwrap();
            if !o.alpn.is_empty() {
                tls["alpn"] = json!(o.alpn);
            }
            let mut value = json!({
                "type": "tuic", "tag": tag, "server": o.server, "server_port": o.port,
                "uuid": o.uuid, "password": o.password, "tls": tls,
            });
            if let Some(cc) = &o.congestion_control {
                value["congestion_control"] = json!(cc);
            }
            value
        }
    }
}

#[cfg(test)]
mod tests;
