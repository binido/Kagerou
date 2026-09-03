use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    #[serde(default)]
    pub premium: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProxyInfo {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub all: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProxiesResponse {
    pub proxies: HashMap<String, ProxyInfo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConnectionMetadata {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub network: String,
    #[serde(rename = "destinationPort", default)]
    pub destination_port: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub metadata: ConnectionMetadata,
    #[serde(default)]
    pub upload: u64,
    #[serde(default)]
    pub download: u64,
    #[serde(default)]
    pub chains: Vec<String>,
    #[serde(default)]
    pub rule: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConnectionsResponse {
    #[serde(rename = "downloadTotal", default)]
    pub download_total: u64,
    #[serde(rename = "uploadTotal", default)]
    pub upload_total: u64,
    #[serde(default)]
    pub connections: Vec<ConnectionInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct TrafficSample {
    pub up: u64,
    pub down: u64,
}

#[derive(Serialize)]
pub(crate) struct SelectOutboundBody<'a> {
    pub name: &'a str,
}
