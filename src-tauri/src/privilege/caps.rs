/// Bit index of `CAP_NET_ADMIN` per `/usr/include/linux/capability.h`.
const CAP_NET_ADMIN_BIT: u32 = 12;

/// Parses the `CapEff:` line of a Linux `/proc/<pid>/status` file to check
/// whether the effective capability set includes `CAP_NET_ADMIN` - the
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
mod tests;
