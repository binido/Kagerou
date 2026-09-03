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

fn transport_block(network: &str, ws_path: Option<&str>, ws_host: Option<&str>) -> Option<Value> {
    if network == "tcp" || network.is_empty() {
        return None;
    }
    let mut transport = json!({ "type": network });
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
            if let Some(transport) =
                transport_block(&o.network, o.ws_path.as_deref(), o.ws_host.as_deref())
            {
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
                transport_block(&o.network, o.ws_path.as_deref(), o.ws_host.as_deref())
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
            if let Some(transport) = transport_block(&o.network, None, None) {
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
mod tests {
    use super::*;
    use crate::subscription::model::{Hysteria2Outbound, VlessOutbound};

    #[test]
    fn vless_with_reality_includes_the_reality_block() {
        let parsed = ParsedOutbound::Vless(VlessOutbound {
            name: "n".into(),
            server: "example.com".into(),
            port: 443,
            uuid: "uuid".into(),
            flow: Some("xtls-rprx-vision".into()),
            network: "tcp".into(),
            tls: true,
            sni: Some("cdn.example.com".into()),
            ws_path: None,
            ws_host: None,
            reality_public_key: Some("pbk".into()),
            reality_short_id: Some("sid".into()),
            fingerprint: None,
        });
        let json = to_singbox_outbound(&parsed, "profile-1");
        assert_eq!(json["type"], "vless");
        assert_eq!(json["tag"], "profile-1");
        assert_eq!(json["tls"]["reality"]["public_key"], "pbk");
        assert_eq!(json["tls"]["reality"]["short_id"], "sid");
        assert_eq!(json["flow"], "xtls-rprx-vision");
        assert_eq!(json["tls"]["utls"]["fingerprint"], "chrome");
        assert!(
            json.get("transport").is_none(),
            "tcp network should not emit a transport block"
        );
    }

    #[test]
    fn a_plaintext_vless_outbound_omits_the_tls_block_entirely() {
        let parsed = ParsedOutbound::Vless(VlessOutbound {
            name: "n".into(),
            server: "127.0.0.1".into(),
            port: 18443,
            uuid: "uuid".into(),
            flow: None,
            network: "tcp".into(),
            tls: false,
            sni: None,
            ws_path: None,
            ws_host: None,
            reality_public_key: None,
            reality_short_id: None,
            fingerprint: None,
        });
        let json = to_singbox_outbound(&parsed, "profile-1");
        assert!(
            json.get("tls").is_none(),
            "sing-box panics on a disabled-but-present tls block: {json}"
        );
    }

    #[test]
    fn hysteria2_includes_obfs_only_when_present() {
        let with_obfs = ParsedOutbound::Hysteria2(Hysteria2Outbound {
            name: "n".into(),
            server: "s".into(),
            port: 443,
            password: "pw".into(),
            sni: None,
            insecure: true,
            obfs: Some("salamander".into()),
            obfs_password: Some("op".into()),
        });
        let json = to_singbox_outbound(&with_obfs, "t");
        assert_eq!(json["obfs"]["type"], "salamander");
        assert_eq!(json["tls"]["insecure"], true);

        let without_obfs = ParsedOutbound::Hysteria2(Hysteria2Outbound {
            name: "n".into(),
            server: "s".into(),
            port: 443,
            password: "pw".into(),
            sni: None,
            insecure: false,
            obfs: None,
            obfs_password: None,
        });
        let json = to_singbox_outbound(&without_obfs, "t");
        assert!(json.get("obfs").is_none());
    }
}
