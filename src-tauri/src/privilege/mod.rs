mod caps;
mod plan;

pub use caps::parse_cap_net_admin;
pub use plan::{plan_launch, to_command, LaunchPlan, TargetOs};

/// Reads this process's own effective capability set from `/proc/self/status`
/// to check for `CAP_NET_ADMIN`. Linux-only glue around `parse_cap_net_admin`;
/// not unit tested itself (there's nothing to assert beyond "reads a file
/// and calls the already-tested parser") — real capability prompts and
/// `/proc` contents aren't something a unit test can meaningfully fake.
#[cfg(target_os = "linux")]
pub fn current_process_has_cap_net_admin() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .map(|status| parse_cap_net_admin(&status))
        .unwrap_or(false)
}
