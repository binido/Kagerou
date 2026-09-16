
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
        log_level: "info",
        tun: false,
        system_proxy: false,
    }
}

fn sets_system_proxy(config: &Value) -> bool {
    config["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["type"] == "mixed")
        .unwrap()["set_system_proxy"]
        .as_bool()
        .unwrap()
}

#[test]
fn the_system_proxy_is_set_only_when_asked_and_never_under_tun() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let mut input = base_input(&profiles, &[]);
    assert!(!sets_system_proxy(&generate(&input).unwrap()));

    input.system_proxy = true;
    assert!(sets_system_proxy(&generate(&input).unwrap()));

    input.tun = true;
    assert!(
        !sets_system_proxy(&generate(&input).unwrap()),
        "an elevated TUN core must not touch the proxy, even if a stale row asks"
    );
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
    let tun = config["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["type"] == "tun")
        .unwrap();
    assert!(
        tun["interface_name"].is_null(),
        "a hardcoded interface name is not valid on every platform; \
             sing-box should choose one: {tun}"
    );

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
    // Past the sniff rule, which is not one of the user's.
    let rules_json = &config["route"]["rules"].as_array().unwrap()[1..];

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

/// Domain rules match on a name nobody has read out of the request until
/// something sniffs for it, so the sniff rule has to be there and has to be
/// first — rules run in order.
#[test]
fn sniffing_comes_before_the_rules_that_depend_on_it() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let rules = vec![rule("example", "example.com", "Proxy")];
    let config = generate(&base_input(&profiles, &rules)).unwrap();
    let rules_json = config["route"]["rules"].as_array().unwrap();

    assert_eq!(rules_json[0], json!({ "action": "sniff" }));
    assert_eq!(rules_json[1]["domain_suffix"], json!(["example.com"]));
    assert_eq!(
        rules_json.len(),
        2,
        "no rule beyond the sniff and the user's"
    );
}

/// Sniffing is not conditional on the user having written any rules: the
/// default outbound is a proxy either way, and a config without it hides
/// every domain from the log.
#[test]
fn sniffing_happens_even_with_no_rules_configured() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    let rules_json = config["route"]["rules"].as_array().unwrap();

    assert_eq!(rules_json, &[json!({ "action": "sniff" })]);
}

#[test]
fn queries_go_to_a_resolver_behind_the_proxy_by_default() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    let dns = &config["dns"];

    assert_eq!(dns["final"], "remote");
    assert_eq!(dns["strategy"], "ipv4_only");
    let remote = &dns["servers"][0];
    assert_eq!(remote["tag"], "remote");
    assert_eq!(remote["type"], "https");
    assert_eq!(
        remote["detour"], "proxy",
        "a resolver reached outside the tunnel is the leak this exists to close"
    );
    assert_eq!(dns["servers"][1]["type"], "local");
}

/// The proxy's own hostname cannot be resolved through the proxy, and
/// 1.14 refuses to start without somewhere to send that query.
#[test]
fn the_proxy_servers_own_names_resolve_locally() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    assert_eq!(config["route"]["default_domain_resolver"], "local");
}

/// A domain routed around the VPN must be resolved around it too, or the
/// answer describes the network near the proxy instead of the one the
/// traffic will actually take.
#[test]
fn direct_domains_are_resolved_locally_and_nothing_else_is() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let rules = vec![
        rule("direct-domain", "intranet.example", "Direct"),
        rule("direct-host", "localhost", "Direct"),
        rule("proxied", "example.com", "Proxy"),
        rule("blocked", "ads.example.net", "Block"),
        rule("direct-cidr", "192.168.0.0/16", "Direct"),
    ];
    let config = generate(&base_input(&profiles, &rules)).unwrap();
    let dns_rules = config["dns"]["rules"].as_array().unwrap();

    assert_eq!(
        dns_rules.len(),
        2,
        "only the two Direct rules carrying a name: {dns_rules:?}"
    );
    assert_eq!(dns_rules[0]["domain_suffix"], json!(["intranet.example"]));
    assert_eq!(dns_rules[0]["server"], "local");
    assert_eq!(dns_rules[1]["domain"], json!(["localhost"]));
    assert_eq!(dns_rules[1]["server"], "local");
}

/// The whole import path in miniature: a subscription line goes through
/// the parser and back out as a stored key, and the generated outbound
/// still has to carry REALITY. It used to come out as plaintext.
#[test]
fn an_imported_reality_subscription_still_generates_a_reality_outbound() {
    let line = "vless://11111111-2222-3333-4444-555555555555@example.com:443?encryption=none&security=reality&sni=cdn.example.com&type=tcp&flow=xtls-rprx-vision&pbk=abc123&sid=de#My%20Node";
    let imported = subscription::parse_subscription(line).unwrap();
    let key = subscription::to_uri(&imported[0]);

    let profiles = vec![profile("p1", &key)];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    let outbound = &config["outbounds"][0];

    assert_eq!(outbound["tls"]["enabled"], true);
    assert_eq!(outbound["tls"]["server_name"], "cdn.example.com");
    assert_eq!(outbound["tls"]["reality"]["enabled"], true);
    assert_eq!(outbound["tls"]["reality"]["public_key"], "abc123");
    assert_eq!(outbound["tls"]["reality"]["short_id"], "de");
    assert_eq!(outbound["flow"], "xtls-rprx-vision");
}

/// Observed live: with TUN on and this missing, sing-box dialled its own
/// proxy server through its own tunnel — 13k connections, 0 bytes moved.
#[test]
fn route_binds_outbounds_to_the_default_interface() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    assert_eq!(config["route"]["auto_detect_interface"], true);
}

#[test]
fn route_final_is_the_proxy_selector() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let config = generate(&base_input(&profiles, &[])).unwrap();
    assert_eq!(config["route"]["final"], "proxy");
}

#[test]
fn log_uses_the_configured_level() {
    let profiles = vec![profile("p1", "vless://uuid@a.example.com:443")];
    let mut input = base_input(&profiles, &[]);
    input.log_level = "debug";
    let config = generate(&input).unwrap();
    assert_eq!(config["log"]["level"], "debug");
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
            "vless://b831381d-6324-4d53-ad4f-8cda48b30811@a.example.com:443?encryption=none&security=reality&sni=cdn.example.com&type=tcp&flow=xtls-rprx-vision&pbk=jNXHt1yRo0vDuchQlIP6Z0ZvjT3KtzVI-T4E7RoLJS0&sid=de&fp=chrome#Node",
        )];
    let rules = vec![rule("r1", "example.com", "proxy")];
    let dir = tempfile::tempdir().unwrap();

    // Both branches: the TUN inbound is only built for `tun: true`, so
    // checking the other shape alone leaves it unvalidated entirely.
    //
    // What this does not check is the stack name: `sing-box check`
    // accepts any string there, including nonsense, because the value is
    // only read when the interface is actually created. Verified by
    // feeding it one. Field names and the rest of the shape are covered;
    // a typo in "gvisor" would only show up on a real connection.
    for tun in [false, true] {
        let mut input = base_input(&profiles, &rules);
        input.tun = tun;
        // On, so the non-TUN shape carries `set_system_proxy: true`.
        // `check` only parses it; the OS proxy is left untouched.
        input.system_proxy = true;
        let config = generate(&input).unwrap();

        let path = dir.path().join(format!("config-tun-{tun}.json"));
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

        let output = std::process::Command::new(&binary)
            .arg("check")
            .arg("-c")
            .arg(&path)
            .output()
            .unwrap_or_else(|e| panic!("could not run {}: {e}", binary.display()));
        assert!(
            output.status.success(),
            "sing-box rejected the generated config with tun={tun}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
