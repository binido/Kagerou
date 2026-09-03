use std::net::IpAddr;

use serde_json::{json, Value};
use thiserror::Error;

use super::outbound_json::to_singbox_outbound;
use crate::storage::models::{Profile, RoutingRule};
use crate::subscription::{self, SubscriptionError};

#[derive(Debug, Error, PartialEq)]
pub enum ConfigError {
    #[error("no profiles to configure")]
    NoProfiles,

    #[error("profile {id} ({name}) has a corrupted key and could not be parsed: {source}")]
    CorruptProfileKey {
        id: String,
        name: String,
        source: SubscriptionError,
    },
}

pub struct ConfigInput<'a> {
    pub profiles: &'a [Profile],
    /// Tag of the profile the `proxy` selector should default to. Falls
    /// back to the first profile if empty or not found among `profiles`.
    pub active_profile_id: &'a str,
    pub routing_rules: &'a [RoutingRule],
    pub mixed_listen_port: u16,
    pub clash_api_listen: &'a str,
    pub tun: bool,
}

/// Builds a full sing-box JSON configuration from stored profiles and
/// routing rules. Each profile's raw URI (`profile.key`) is re-parsed here
/// rather than trusting cached protocol metadata, so a hand-edited or
/// otherwise corrupted key is caught with a specific error naming the
/// offending profile instead of producing a config sing-box would reject
/// opaquely at startup.
pub fn generate(input: &ConfigInput) -> Result<Value, ConfigError> {
    if input.profiles.is_empty() {
        return Err(ConfigError::NoProfiles);
    }

    let mut outbounds = Vec::with_capacity(input.profiles.len() + 3);
    let mut tags = Vec::with_capacity(input.profiles.len());
    for profile in input.profiles {
        let parsed = subscription::parse_uri(&profile.key).map_err(|source| {
            ConfigError::CorruptProfileKey {
                id: profile.id.clone(),
                name: profile.name.clone(),
                source,
            }
        })?;
        outbounds.push(to_singbox_outbound(&parsed, &profile.id));
        tags.push(profile.id.clone());
    }

    let default_tag = if tags.iter().any(|t| t == input.active_profile_id) {
        input.active_profile_id.to_string()
    } else {
        tags[0].clone()
    };

    outbounds.push(json!({ "type": "direct", "tag": "direct" }));
    outbounds.push(json!({ "type": "block", "tag": "block" }));
    outbounds.push(
        json!({ "type": "selector", "tag": "proxy", "outbounds": tags, "default": default_tag }),
    );

    let mut inbounds = vec![json!({
        "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": input.mixed_listen_port,
    })];
    if input.tun {
        inbounds.push(json!({
            "type": "tun", "tag": "tun-in", "interface_name": "kagerou0",
            "address": ["172.19.0.1/30"], "auto_route": true, "strict_route": true, "stack": "system",
        }));
    }

    Ok(json!({
        "log": { "level": "info", "timestamp": true },
        "inbounds": inbounds,
        "outbounds": outbounds,
        "route": {
            "rules": input.routing_rules.iter().map(routing_rule_to_json).collect::<Vec<_>>(),
            "final": "proxy",
        },
        "experimental": {
            "clash_api": { "external_controller": input.clash_api_listen },
        },
    }))
}

fn outbound_tag_for(outbound: &str) -> &'static str {
    match outbound {
        "Direct" => "direct",
        "Block" => "block",
        _ => "proxy",
    }
}

fn routing_rule_to_json(rule: &RoutingRule) -> Value {
    let outbound = outbound_tag_for(&rule.outbound);
    let matcher = classify_match(&rule.match_value);
    let mut value = json!({ "outbound": outbound });
    match matcher {
        Matcher::Domain(d) => value["domain"] = json!([d]),
        Matcher::DomainSuffix(d) => value["domain_suffix"] = json!([d]),
        Matcher::IpCidr(c) => value["ip_cidr"] = json!([c]),
    }
    value
}

enum Matcher {
    Domain(String),
    DomainSuffix(String),
    IpCidr(String),
}

fn classify_match(raw: &str) -> Matcher {
    if raw.eq_ignore_ascii_case("localhost") {
        return Matcher::Domain(raw.to_string());
    }
    if let Some((addr, _prefix)) = raw.split_once('/') {
        if addr.parse::<IpAddr>().is_ok() {
            return Matcher::IpCidr(raw.to_string());
        }
    }
    if raw.parse::<IpAddr>().is_ok() {
        let cidr = match raw.parse::<IpAddr>().unwrap() {
            IpAddr::V4(_) => format!("{raw}/32"),
            IpAddr::V6(_) => format!("{raw}/128"),
        };
        return Matcher::IpCidr(cidr);
    }
    Matcher::DomainSuffix(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::{Protocol, TestResult, Tone};

    fn profile(id: &str, key: &str) -> Profile {
        Profile {
            id: id.into(),
            name: id.into(),
            region: "r".into(),
            protocol: Protocol::VLESS,
            origin: "local".into(),
            group_id: "g".into(),
            source_id: None,
            selected: false,
            tcp: TestResult {
                value: "".into(),
                tone: Tone::Muted,
            },
            url: TestResult {
                value: "".into(),
                tone: Tone::Muted,
            },
            key: key.into(),
        }
    }

    fn rule(id: &str, m: &str, outbound: &str) -> RoutingRule {
        RoutingRule {
            id: id.into(),
            match_value: m.into(),
            outbound: outbound.into(),
            selected: false,
        }
    }

    fn base_input<'a>(profiles: &'a [Profile], rules: &'a [RoutingRule]) -> ConfigInput<'a> {
        ConfigInput {
            profiles,
            active_profile_id: "p1",
            routing_rules: rules,
            mixed_listen_port: 2080,
            clash_api_listen: "127.0.0.1:9090",
            tun: false,
        }
    }

    #[test]
    fn empty_profiles_is_an_error() {
        let err = generate(&base_input(&[], &[])).unwrap_err();
        assert_eq!(err, ConfigError::NoProfiles);
    }

    #[test]
    fn a_corrupted_profile_key_is_reported_by_id_and_name() {
        let profiles = vec![profile("p1", "not-a-valid-uri-at-all")];
        let err = generate(&base_input(&profiles, &[])).unwrap_err();
        match err {
            ConfigError::CorruptProfileKey { id, name, .. } => {
                assert_eq!(id, "p1");
                assert_eq!(name, "p1");
            }
            other => panic!("expected CorruptProfileKey, got {other:?}"),
        }
    }

    #[test]
    fn selector_defaults_to_the_active_profile() {
        let profiles = vec![
            profile("p1", "vless://uuid@a.example.com:443"),
            profile("p2", "vless://uuid@b.example.com:443"),
        ];
        let mut input = base_input(&profiles, &[]);
        input.active_profile_id = "p2";
        let config = generate(&input).unwrap();
        let selector = config["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["tag"] == "proxy")
            .unwrap();
        assert_eq!(selector["default"], "p2");
        assert_eq!(selector["outbounds"], json!(["p1", "p2"]));
    }

    #[test]
    fn selector_falls_back_to_the_first_profile_when_active_id_is_unknown() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let mut input = base_input(&profiles, &[]);
        input.active_profile_id = "does-not-exist";
        let config = generate(&input).unwrap();
        let selector = config["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["tag"] == "proxy")
            .unwrap();
        assert_eq!(selector["default"], "p1");
    }

    #[test]
    fn includes_direct_and_block_outbounds() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let config = generate(&base_input(&profiles, &[])).unwrap();
        let tags: Vec<_> = config["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["tag"].as_str().unwrap().to_string())
            .collect();
        assert!(tags.contains(&"direct".to_string()));
        assert!(tags.contains(&"block".to_string()));
    }

    #[test]
    fn tun_flag_adds_a_tun_inbound() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let mut input = base_input(&profiles, &[]);
        input.tun = true;
        let config = generate(&input).unwrap();
        let inbound_types: Vec<_> = config["inbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["type"].as_str().unwrap().to_string())
            .collect();
        assert!(inbound_types.contains(&"tun".to_string()));

        input.tun = false;
        let config = generate(&input).unwrap();
        let inbound_types: Vec<_> = config["inbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["type"].as_str().unwrap().to_string())
            .collect();
        assert!(!inbound_types.contains(&"tun".to_string()));
    }

    #[test]
    fn classifies_cidr_localhost_and_domain_rules_correctly() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let rules = vec![
            rule("lan", "192.168.0.0/16", "Direct"),
            rule("localhost", "localhost", "Direct"),
            rule("example", "example.com", "Proxy"),
            rule("ads", "ads.example.net", "Block"),
            rule("single-ip", "10.0.0.5", "Direct"),
        ];
        let config = generate(&base_input(&profiles, &rules)).unwrap();
        let rules_json = config["route"]["rules"].as_array().unwrap();

        assert_eq!(rules_json[0]["ip_cidr"], json!(["192.168.0.0/16"]));
        assert_eq!(rules_json[0]["outbound"], "direct");

        assert_eq!(rules_json[1]["domain"], json!(["localhost"]));
        assert_eq!(rules_json[1]["outbound"], "direct");

        assert_eq!(rules_json[2]["domain_suffix"], json!(["example.com"]));
        assert_eq!(rules_json[2]["outbound"], "proxy");

        assert_eq!(rules_json[3]["domain_suffix"], json!(["ads.example.net"]));
        assert_eq!(rules_json[3]["outbound"], "block");

        assert_eq!(rules_json[4]["ip_cidr"], json!(["10.0.0.5/32"]));
    }

    #[test]
    fn route_final_is_the_proxy_selector() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let config = generate(&base_input(&profiles, &[])).unwrap();
        assert_eq!(config["route"]["final"], "proxy");
    }

    #[test]
    fn clash_api_controller_uses_the_given_listen_address() {
        let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
        let config = generate(&base_input(&profiles, &[])).unwrap();
        assert_eq!(
            config["experimental"]["clash_api"]["external_controller"],
            "127.0.0.1:9090"
        );
    }

    /// Opt-in smoke test against the real bundled binary (fetched by
    /// `scripts/fetch-singbox.mjs`, copied into the target dir by
    /// tauri-build): `cargo test -- --ignored`. Catches config-schema drift
    /// when the pinned sing-box version moves — something no amount of
    /// JSON-shape assertions above can notice.
    #[test]
    #[ignore = "needs the bundled sing-box binary; run with --ignored"]
    fn the_bundled_sing_box_accepts_a_generated_config() {
        let exe = std::env::current_exe().unwrap();
        // target/<profile>/deps/<test binary> -> target/<profile>/sing-box
        let binary = exe.parent().unwrap().parent().unwrap().join("sing-box");

        let profiles = vec![profile(
            "p1",
            "vless://b831381d-6324-4d53-ad4f-8cda48b30811@a.example.com:443",
        )];
        let rules = vec![rule("r1", "example.com", "proxy")];
        let config = generate(&base_input(&profiles, &rules)).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

        let output = std::process::Command::new(&binary)
            .arg("check")
            .arg("-c")
            .arg(&path)
            .output()
            .unwrap_or_else(|e| panic!("could not run {}: {e}", binary.display()));
        assert!(
            output.status.success(),
            "sing-box rejected the generated config:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
