use std::path::{Path, PathBuf};
use std::process::Command;

/// The desktop OS to plan an elevated launch for. A real value comes from
/// `std::env::consts::OS`; kept as its own enum (rather than branching on
/// the string directly) so `plan_launch` and `to_command` are pure,
/// deterministic functions testable on any host regardless of which OS
/// they're compiled/run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    MacOs,
    Linux,
}

impl TargetOs {
    pub fn current() -> Option<Self> {
        match std::env::consts::OS {
            "windows" => Some(TargetOs::Windows),
            "macos" => Some(TargetOs::MacOs),
            "linux" => Some(TargetOs::Linux),
            _ => None,
        }
    }
}

/// How to launch sing-box so it has the privileges TUN mode needs.
#[derive(Debug, Clone, PartialEq)]
pub enum LaunchPlan {
    /// Already sufficiently privileged (Linux with `CAP_NET_ADMIN` already
    /// set on the binary) - just run it.
    Direct { program: PathBuf, args: Vec<String> },
    /// Windows: relaunch through PowerShell's `Start-Process -Verb RunAs`,
    /// which triggers the UAC consent prompt.
    WindowsRunAs { program: PathBuf, args: Vec<String> },
    /// macOS: `osascript ... with administrator privileges`, which
    /// triggers the native admin-password prompt.
    MacOsAdminPrompt { program: PathBuf, args: Vec<String> },
    /// Linux without `CAP_NET_ADMIN` on the binary: `pkexec`, which
    /// triggers a polkit authentication prompt.
    LinuxPolkit { program: PathBuf, args: Vec<String> },
}

/// Decides how sing-box should be launched for TUN mode. `linux_has_cap_net_admin`
/// is the caller's answer to "does the sing-box binary already have
/// CAP_NET_ADMIN set" (see `caps::parse_cap_net_admin` for how to compute
/// it) - irrelevant on the other two platforms, which always need a
/// per-run privilege prompt.
pub fn plan_launch(
    os: TargetOs,
    binary: &Path,
    args: &[String],
    linux_has_cap_net_admin: bool,
) -> LaunchPlan {
    let program = binary.to_path_buf();
    let args = args.to_vec();
    match os {
        TargetOs::Windows => LaunchPlan::WindowsRunAs { program, args },
        TargetOs::MacOs => LaunchPlan::MacOsAdminPrompt { program, args },
        TargetOs::Linux => {
            if linux_has_cap_net_admin {
                LaunchPlan::Direct { program, args }
            } else {
                LaunchPlan::LinuxPolkit { program, args }
            }
        }
    }
}

fn powershell_single_quote(s: &str) -> String {
    // PowerShell single-quoted strings escape an embedded ' by doubling it.
    format!("'{}'", s.replace('\'', "''"))
}

fn applescript_double_quote_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// A single shell-safe (POSIX `sh`) token: wraps in single quotes and
/// escapes any embedded single quote as `'\''`.
fn posix_shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// A POSIX `sh` program that runs sing-box and then outlives it only as
/// long as `run_file` exists.
///
/// The elevated process is not ours to signal: `osascript`/`pkexec` hand
/// privileges to a process that is reparented away from us, so an
/// unprivileged `kill` from the app can never reach it (this is exactly how
/// a root sing-box used to survive the app and keep the machine offline).
/// Instead the privileged side watches a file the app owns: deleting it -
/// on disconnect, on shutdown, or on the next startup after a crash - is
/// the stop signal, and needs no second password prompt.
fn posix_watchdog(program: &Path, args: &[String], run_file: &Path) -> String {
    let command = std::iter::once(program.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|part| posix_shell_quote(&part))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{command} & child=$!; \
         while [ -e {run_file} ] && kill -0 $child 2>/dev/null; do sleep 1; done; \
         kill $child 2>/dev/null; wait $child 2>/dev/null",
        run_file = posix_shell_quote(&run_file.to_string_lossy()),
    )
}

/// Builds the actual `Command` to spawn for a given plan. `run_file` is the
/// liveness sentinel described on [`posix_watchdog`]: the caller creates it
/// before spawning and deletes it to ask the process to stop.
pub fn to_command(plan: &LaunchPlan, run_file: &Path) -> Command {
    match plan {
        // Already privileged, so this one really is our own child and a
        // plain kill reaches it - no sentinel needed.
        LaunchPlan::Direct { program, args } => {
            let mut command = Command::new(program);
            command.args(args);
            command
        }
        LaunchPlan::WindowsRunAs { program, args } => {
            let arg_list = args
                .iter()
                .map(|a| powershell_single_quote(a))
                .collect::<Vec<_>>()
                .join(",");
            // The watchdog has to run elevated too: an unprivileged parent
            // cannot Stop-Process a child it launched with -Verb RunAs.
            let inner = format!(
                "$p = Start-Process -FilePath {} -ArgumentList {} -WindowStyle Hidden -PassThru; \
                 while ((Test-Path {}) -and !$p.HasExited) {{ Start-Sleep -Seconds 1 }}; \
                 if (!$p.HasExited) {{ Stop-Process -Id $p.Id -Force }}",
                powershell_single_quote(&program.to_string_lossy()),
                arg_list,
                powershell_single_quote(&run_file.to_string_lossy()),
            );
            let ps_command = format!(
                "Start-Process -FilePath 'powershell' -ArgumentList '-NoProfile','-WindowStyle','Hidden','-Command',{} -Verb RunAs -WindowStyle Hidden",
                powershell_single_quote(&inner),
            );
            let mut command = Command::new("powershell");
            command.args(["-NoProfile", "-NonInteractive", "-Command", &ps_command]);
            command
        }
        LaunchPlan::MacOsAdminPrompt { program, args } => {
            let mut command = Command::new("osascript");
            command.arg("-e");
            command.arg(format!(
                "do shell script \"{}\" with administrator privileges",
                applescript_double_quote_escape(&posix_watchdog(program, args, run_file))
            ));
            command
        }
        LaunchPlan::LinuxPolkit { program, args } => {
            let mut command = Command::new("pkexec");
            command.args(["/bin/sh", "-c", &posix_watchdog(program, args, run_file)]);
            command
        }
    }
}

#[cfg(test)]
mod tests;
