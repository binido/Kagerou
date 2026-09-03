use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tone {
    Good,
    Warn,
    Bad,
    Muted,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Tone::Good => "good",
            Tone::Warn => "warn",
            Tone::Bad => "bad",
            Tone::Muted => "muted",
        }
    }
}

impl std::str::FromStr for Tone {
    type Err = super::StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "good" => Ok(Tone::Good),
            "warn" => Ok(Tone::Warn),
            "bad" => Ok(Tone::Bad),
            "muted" => Ok(Tone::Muted),
            other => Err(super::StorageError::InvalidInput(format!(
                "unknown tone: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
    pub value: String,
    pub tone: Tone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    VLESS,
    VMess,
    Trojan,
    Shadowsocks,
    Hysteria2,
    Tuic,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::VLESS => "VLESS",
            Protocol::VMess => "VMess",
            Protocol::Trojan => "Trojan",
            Protocol::Shadowsocks => "Shadowsocks",
            Protocol::Hysteria2 => "Hysteria2",
            Protocol::Tuic => "Tuic",
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = super::StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VLESS" => Ok(Protocol::VLESS),
            "VMess" => Ok(Protocol::VMess),
            "Trojan" => Ok(Protocol::Trojan),
            "Shadowsocks" => Ok(Protocol::Shadowsocks),
            "Hysteria2" => Ok(Protocol::Hysteria2),
            "Tuic" => Ok(Protocol::Tuic),
            other => Err(super::StorageError::InvalidInput(format!(
                "unknown protocol: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub region: String,
    pub protocol: Protocol,
    pub origin: String, // "local" | "imported"
    pub group_id: String,
    pub source_id: Option<String>,
    pub selected: bool,
    pub tcp: TestResult,
    pub url: TestResult,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProfile {
    pub id: String,
    pub name: String,
    pub region: String,
    pub protocol: Protocol,
    pub origin: String,
    pub group_id: String,
    pub source_id: Option<String>,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileGroup {
    pub id: String,
    pub label: String,
    pub kind: String, // "default" | "custom" | "subscription"
    pub source_id: Option<String>,
    pub open: bool,
    pub profile_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProfileGroup {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String, // "url" | "key"
    pub value: String,
    pub status: String,
    pub last_refresh: String,
    pub origin_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSource {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    pub status: String,
    pub last_refresh: String,
    pub origin_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingPreset {
    pub id: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    pub id: String,
    #[serde(rename = "match")]
    pub match_value: String,
    pub outbound: String, // "Direct" | "Proxy" | "Block"
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoutingRule {
    pub id: String,
    pub match_value: String,
    pub outbound: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub startup: bool,
    pub tun_interface: String,
    pub auto_update_subscriptions: bool,
    pub subscription_update_interval: String,
    pub custom_subscription_update_minutes: i64,
    pub group_sort: String,
}
