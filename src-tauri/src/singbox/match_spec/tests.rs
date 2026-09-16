use super::*;

fn kind(raw: &str) -> MatchKind {
    analyze(raw).kind
}

fn warning(raw: &str) -> Option<MatchWarning> {
    analyze(raw).warning
}

#[test]
fn a_leading_dot_is_dropped_before_anything_else_looks_at_it() {
    let analysis = analyze("  .example.com  ");
    assert_eq!(analysis.normalized, "example.com");
    assert_eq!(analysis.kind, MatchKind::DomainSuffix);
    assert_eq!(analysis.warning, None);
}

#[test]
fn plain_names_and_addresses_classify_without_complaint() {
    assert_eq!(kind("example.com"), MatchKind::DomainSuffix);
    assert_eq!(kind("localhost"), MatchKind::Domain);
    assert_eq!(kind("10.0.0.5"), MatchKind::IpCidr);
    assert_eq!(kind("192.168.0.0/16"), MatchKind::IpCidr);
    assert_eq!(kind("fd00::/8"), MatchKind::IpCidr);
    for raw in ["example.com", "localhost", "10.0.0.5", "192.168.0.0/16"] {
        assert_eq!(warning(raw), None, "{raw}");
    }
}

#[test]
fn a_prefix_out_of_range_is_caught_rather_than_left_to_the_core() {
    assert_eq!(warning("192.168.1.0/99"), Some(MatchWarning::InvalidPrefix));
    assert_eq!(
        warning("192.168.1.0/abc"),
        Some(MatchWarning::InvalidPrefix)
    );
    assert_eq!(warning("fd00::/200"), Some(MatchWarning::InvalidPrefix));
    // 128 is fine for v6 and would be out of range if the width were
    // read off the wrong family.
    assert_eq!(warning("fd00::/128"), None);
}

#[test]
fn the_two_mistakes_worth_naming_get_their_own_codes() {
    assert_eq!(warning("*.example.com"), Some(MatchWarning::Wildcard));
    assert_eq!(warning("https://example.com/foo"), Some(MatchWarning::Url));
    assert_eq!(warning("example.com/foo"), Some(MatchWarning::Url));
}

#[test]
fn unicode_is_flagged_rather_than_converted() {
    assert_eq!(warning("пример.рф"), Some(MatchWarning::NonAscii));
}

#[test]
fn nonsense_falls_back_to_the_generic_code() {
    for raw in [
        "example",
        "exa mple.com",
        "-example.com",
        "example..com",
        "",
    ] {
        assert_eq!(warning(raw), Some(MatchWarning::InvalidDomain), "{raw}");
    }
}

#[test]
fn the_wire_format_is_codes_and_a_normalized_value() {
    let value = serde_json::to_value(analyze(".Example.com")).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "normalized": "Example.com",
            "kind": "domain-suffix",
            "warning": null,
        })
    );
    let value = serde_json::to_value(analyze("*.example.com")).unwrap();
    assert_eq!(value["warning"], "wildcard");
}
