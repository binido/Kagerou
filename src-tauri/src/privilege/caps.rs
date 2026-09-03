/// Bit index of `CAP_NET_ADMIN` per `/usr/include/linux/capability.h`.
const CAP_NET_ADMIN_BIT: u32 = 12;

/// Parses the `CapEff:` line of a Linux `/proc/<pid>/status` file to check
/// whether the effective capability set includes `CAP_NET_ADMIN` — the
/// capability that lets sing-box create a TUN device without running as
/// root or going through `pkexec` every launch (set once at install time
/// via `setcap cap_net_admin+ep <binary>`).
///
/// A missing/unparseable `CapEff` line is treated as "not present" rather
/// than an error: the caller should fall back to the polkit prompt, which
/// is always safe, just less convenient.
pub fn parse_cap_net_admin(proc_status: &str) -> bool {
    proc_status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:").map(str::trim))
        .and_then(|hex| u64::from_str_radix(hex, 16).ok())
        .map(|mask| (mask >> CAP_NET_ADMIN_BIT) & 1 == 1)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
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
}
