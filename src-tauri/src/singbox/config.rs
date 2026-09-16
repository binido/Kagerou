use serde_json::{json, Value};
use thiserror::Error;

use super::match_spec::{classify_match, Matcher};
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
    pub log_level: &'a str,
    pub tun: bool,
    /// Has sing-box set the OS proxy to the mixed inbound. Ignored under TUN.
    pub system_proxy: bool,
}

/// Builds a full sing-box JSON configuration from stored profiles and
/// routing rules. Each profile's raw URI (`profile.key`) is re-parsed here
/// rather than trusting cached protocol metadata, so a hand-edited or
/// otherwise corrupted key is caught with a specific error naming the
/// offending profile instead of producing a config sing-box would reject
/// opaquely at startup.
/// Resolvers, and which queries reach which one.
///
/// Without a `dns` block sing-box falls back to the system resolver, so every
/// name the user visits goes to their ISP in the clear while the traffic
/// itself is tunnelled. That is the leak this closes: queries go to a DoH
/// resolver reached *through* the proxy, so the ISP sees an encrypted
/// connection to the proxy and nothing else.
///
/// Two things deliberately stay local. The proxy servers' own hostnames,
/// which cannot be resolved through a tunnel that does not exist yet — that
/// is what `route.default_domain_resolver` is for, and 1.14 refuses to start
/// without it. And any domain the routing rules send Direct: resolving those
/// remotely would hand back a CDN address near the proxy rather than near the
/// user, sending traffic that was meant to skip the VPN across the world to
/// come back.
///
/// IPv4 only, to match the tunnel actually built: the TUN inbound is given a
/// v4 address and nothing else, so an AAAA answer routes a connection into a
/// tunnel that cannot carry it and it hangs.
fn dns_block(rules: &[RoutingRule]) -> Value {
    let mut dns_rules: Vec<Value> = Vec::new();
    for rule in rules {
        if outbound_tag_for(&rule.outbound) != "direct" {
            continue;
        }
        // An ip_cidr rule has no name to resolve, so there is nothing to
        // point at a resolver.
        let mut value = match classify_match(&rule.match_value) {
            Matcher::Domain(d) => json!({ "domain": [d] }),
            Matcher::DomainSuffix(d) => json!({ "domain_suffix": [d] }),
            Matcher::IpCidr(_) => continue,
        };
        value["server"] = json!("local");
        dns_rules.push(value);
    }

    json!({
        "servers": [
            // Addressed by IP on purpose: a resolver named by domain would
            // itself need resolving, and the only thing available to do that
            // is the system resolver we are trying not to leak to.
            { "tag": "remote", "type": "https", "server": "1.1.1.1", "detour": "proxy" },
            { "tag": "local", "type": "local" },
        ],
        "rules": dns_rules,
        "final": "remote",
        "strategy": "ipv4_only",
    })
}

/// The routing rules, preceded by the sniff rule they depend on.
///
/// Without sniffing, a connection reaches the rules as an address and a port,
/// so a `domain` or `domain_suffix` rule can never match anything a browser
/// sends: the name is inside the request, and nobody has looked. The rule sits
/// there looking configured and routes nothing.
///
/// It has to come first — rules are evaluated in order and sniffing is what
/// gives the later ones a domain to match on. Rule actions are how sing-box
/// has done this since 1.11; the inbound-level `sniff` fields were removed in
/// 1.13 and 1.14 refuses a config carrying them.
fn sniff_first(rules: &[RoutingRule]) -> Vec<Value> {
    let mut out = vec![json!({ "action": "sniff" })];
    out.extend(rules.iter().map(routing_rule_to_json));
    out
}

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
        // sing-box points the OS proxy here on start and undoes it on a clean
        // stop. Never with TUN: storage keeps the two exclusive, but a row
        // saved before it did can still hold both.
        "set_system_proxy": input.system_proxy && !input.tun,
    })];
    if input.tun {
        // ponytail: no `interface_name` — sing-box picks a valid one per
        // platform (macOS only accepts `utunN`). Set it explicitly only if
        // users ever need to pin the device name.
        inbounds.push(json!({
            // gvisor, not the default mixed: mixed keeps TCP on the system
            // stack, whose kernel TCP path hung every TCP connection through
            // the tunnel on CachyOS (7.2.2) while UDP and DNS kept working.
            "type": "tun", "tag": "tun-in",
            "address": ["172.19.0.1/30"], "auto_route": true, "strict_route": true, "stack": "gvisor",
        }));
    }

    Ok(json!({
        "log": { "level": input.log_level, "timestamp": true },
        "dns": dns_block(input.routing_rules),
        "inbounds": inbounds,
        "outbounds": outbounds,
        "route": {
            "rules": sniff_first(input.routing_rules),
            "final": "proxy",
            // Without this, TUN's auto_route captures sing-box's own
            // connections to the proxy server and feeds them back into the
            // tunnel, which then dials the server again: a loop that opens
            // thousands of connections and moves zero bytes. Binding
            // outbounds to the default NIC breaks it.
            "auto_detect_interface": true,
            // Mandatory since 1.14 once anything resolves a name, and the
            // proxy servers' own hostnames do. Local, necessarily: their
            // addresses cannot come through a tunnel that is not up yet.
            "default_domain_resolver": "local",
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

#[cfg(test)]
mod tests;
