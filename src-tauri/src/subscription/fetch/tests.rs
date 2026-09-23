use super::*;

#[test]
fn user_agent_names_the_bundled_core_version() {
    let script = include_str!("../../../../scripts/fetch-singbox.mjs");
    assert!(
        script.contains(&format!("const VERSION = \"{SINGBOX_VERSION}\";")),
        "SINGBOX_VERSION is out of step with scripts/fetch-singbox.mjs"
    );
    assert!(user_agent().contains("singbox/"));
}
