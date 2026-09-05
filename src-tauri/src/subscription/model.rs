use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VmessOutbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub alter_id: u32,
    pub security: String,
    pub network: String,
    pub tls: bool,
    pub sni: Option<String>,
    pub ws_path: Option<String>,
    pub ws_host: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VlessOutbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub flow: Option<String>,
    pub network: String,
    pub tls: bool,
    pub sni: Option<String>,
    pub ws_path: Option<String>,
    pub ws_host: Option<String>,
    pub reality_public_key: Option<String>,
    pub reality_short_id: Option<String>,
    /// uTLS fingerprint (`fp=`). sing-box refuses to start a REALITY client
    /// without one, so `to_singbox_outbound` falls back to a default.
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrojanOutbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub sni: Option<String>,
    pub network: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowsocksOutbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub method: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hysteria2Outbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub sni: Option<String>,
    pub insecure: bool,
    pub obfs: Option<String>,
    pub obfs_password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TuicOutbound {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub password: String,
    pub sni: Option<String>,
    pub congestion_control: Option<String>,
    pub alpn: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "protocol")]
pub enum ParsedOutbound {
    Vmess(VmessOutbound),
    Vless(VlessOutbound),
    Trojan(TrojanOutbound),
    Shadowsocks(ShadowsocksOutbound),
    Hysteria2(Hysteria2Outbound),
    Tuic(TuicOutbound),
}

impl ParsedOutbound {
    pub fn name(&self) -> &str {
        match self {
            ParsedOutbound::Vmess(o) => &o.name,
            ParsedOutbound::Vless(o) => &o.name,
            ParsedOutbound::Trojan(o) => &o.name,
            ParsedOutbound::Shadowsocks(o) => &o.name,
            ParsedOutbound::Hysteria2(o) => &o.name,
            ParsedOutbound::Tuic(o) => &o.name,
        }
    }

    pub fn server(&self) -> &str {
        match self {
            ParsedOutbound::Vmess(o) => &o.server,
            ParsedOutbound::Vless(o) => &o.server,
            ParsedOutbound::Trojan(o) => &o.server,
            ParsedOutbound::Shadowsocks(o) => &o.server,
            ParsedOutbound::Hysteria2(o) => &o.server,
            ParsedOutbound::Tuic(o) => &o.server,
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            ParsedOutbound::Vmess(o) => o.port,
            ParsedOutbound::Vless(o) => o.port,
            ParsedOutbound::Trojan(o) => o.port,
            ParsedOutbound::Shadowsocks(o) => o.port,
            ParsedOutbound::Hysteria2(o) => o.port,
            ParsedOutbound::Tuic(o) => o.port,
        }
    }

    /// Whether a TCP handshake to `server:port` says anything about this
    /// outbound. Hysteria2 and TUIC ride QUIC, so their port only answers
    /// UDP: a TCP probe there times out on a perfectly healthy server.
    pub fn answers_tcp(&self) -> bool {
        !matches!(self, ParsedOutbound::Hysteria2(_) | ParsedOutbound::Tuic(_))
    }

    pub fn protocol_label(&self) -> &'static str {
        match self {
            ParsedOutbound::Vmess(_) => "VMess",
            ParsedOutbound::Vless(_) => "VLESS",
            ParsedOutbound::Trojan(_) => "Trojan",
            ParsedOutbound::Shadowsocks(_) => "Shadowsocks",
            ParsedOutbound::Hysteria2(_) => "Hysteria2",
            ParsedOutbound::Tuic(_) => "Tuic",
        }
    }
}
