
use super::*;

#[test]
fn detects_cap_net_admin_when_its_bit_is_set() {
    // Bit 12 set, nothing else: 0x1000
    let status = "Name:\tsing-box\nState:\tR (running)\nCapEff:\t0000000000001000\n";
    assert!(parse_cap_net_admin(status));
}

#[test]
fn reports_false_when_the_bit_is_not_set() {
    // CAP_CHOWN (bit 0) and CAP_KILL (bit 5) set, but not bit 12.
    let status = "CapEff:\t0000000000000021\n";
    assert!(!parse_cap_net_admin(status));
}

#[test]
fn reports_false_for_a_fully_empty_capability_set() {
    let status = "CapEff:\t0000000000000000\n";
    assert!(!parse_cap_net_admin(status));
}

#[test]
fn detects_cap_net_admin_among_a_realistic_full_root_capability_mask() {
    // The typical "root, all capabilities" mask.
    let status = "CapEff:\t0000003fffffffff\n";
    assert!(parse_cap_net_admin(status));
}

#[test]
fn a_missing_capeff_line_is_treated_as_not_present_rather_than_panicking() {
    let status = "Name:\tsing-box\nState:\tR (running)\n";
    assert!(!parse_cap_net_admin(status));
}

#[test]
fn a_malformed_hex_value_is_treated_as_not_present_rather_than_panicking() {
    let status = "CapEff:\tnot-hex-at-all\n";
    assert!(!parse_cap_net_admin(status));
}

#[test]
fn an_empty_file_is_treated_as_not_present() {
    assert!(!parse_cap_net_admin(""));
}
