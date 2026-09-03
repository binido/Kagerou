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
    /// set on the binary) — just run it.
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
/// it) — irrelevant on the other two platforms, which always need a
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

/// Builds the actual `Command` to spawn for a given plan.
pub fn to_command(plan: &LaunchPlan) -> Command {
    match plan {
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
            let ps_command = format!(
                "Start-Process -FilePath {} -ArgumentList {} -Verb RunAs -WindowStyle Hidden",
                powershell_single_quote(&program.to_string_lossy()),
                arg_list
            );
            let mut command = Command::new("powershell");
            command.args(["-NoProfile", "-NonInteractive", "-Command", &ps_command]);
            command
        }
        LaunchPlan::MacOsAdminPrompt { program, args } => {
            let shell_command = std::iter::once(program.to_string_lossy().into_owned())
                .chain(args.iter().cloned())
                .map(|part| posix_shell_quote(&part))
                .collect::<Vec<_>>()
                .join(" ");
            let mut command = Command::new("osascript");
            command.arg("-e");
            command.arg(format!(
                "do shell script \"{}\" with administrator privileges",
                applescript_double_quote_escape(&shell_command)
            ));
            command
        }
        LaunchPlan::LinuxPolkit { program, args } => {
            let mut command = Command::new("pkexec");
            command.arg(program);
            command.args(args);
            command
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program() -> PathBuf {
        PathBuf::from("/opt/kagerou/sing-box")
    }

    #[test]
    fn windows_always_needs_run_as_regardless_of_cap_net_admin() {
        let plan = plan_launch(
            TargetOs::Windows,
            &program(),
            &["run".into(), "-c".into(), "config.json".into()],
            true,
        );
        assert!(matches!(plan, LaunchPlan::WindowsRunAs { .. }));
    }

    #[test]
    fn macos_always_needs_an_admin_prompt() {
        let plan = plan_launch(TargetOs::MacOs, &program(), &[], true);
        assert!(matches!(plan, LaunchPlan::MacOsAdminPrompt { .. }));
    }

    #[test]
    fn linux_with_cap_net_admin_runs_directly() {
        let plan = plan_launch(TargetOs::Linux, &program(), &[], true);
        assert!(matches!(plan, LaunchPlan::Direct { .. }));
    }

    #[test]
    fn linux_without_cap_net_admin_falls_back_to_polkit() {
        let plan = plan_launch(TargetOs::Linux, &program(), &[], false);
        assert!(matches!(plan, LaunchPlan::LinuxPolkit { .. }));
    }

    #[test]
    fn direct_command_passes_args_through_unmodified() {
        let plan = LaunchPlan::Direct {
            program: program(),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan);
        assert_eq!(command.get_program(), program().as_os_str());
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args, vec!["run", "-c", "config.json"]);
    }

    #[test]
    fn linux_polkit_command_runs_pkexec_with_the_program_as_the_first_argument() {
        let plan = LaunchPlan::LinuxPolkit {
            program: program(),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan);
        assert_eq!(command.get_program(), "pkexec");
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(
            args,
            vec!["/opt/kagerou/sing-box", "run", "-c", "config.json"]
        );
    }

    #[test]
    fn windows_run_as_command_invokes_powershell_start_process_with_verb_runas() {
        let plan = LaunchPlan::WindowsRunAs {
            program: PathBuf::from(r"C:\Program Files\Kagerou\sing-box.exe"),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan);
        assert_eq!(command.get_program(), "powershell");
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let ps_command = args.last().unwrap();
        assert!(
            ps_command.contains("-Verb RunAs"),
            "must trigger the UAC prompt: {ps_command}"
        );
        assert!(
            ps_command.contains(r"C:\Program Files\Kagerou\sing-box.exe"),
            "must reference the real binary path: {ps_command}"
        );
        assert!(
            ps_command.contains("'run','-c','config.json'"),
            "args must be passed through: {ps_command}"
        );
    }

    #[test]
    fn windows_run_as_command_escapes_an_embedded_single_quote_in_a_path() {
        let plan = LaunchPlan::WindowsRunAs {
            program: PathBuf::from(r"C:\Users\O'Brien\sing-box.exe"),
            args: vec![],
        };
        let command = to_command(&plan);
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let ps_command = args.last().unwrap();
        assert!(
            ps_command.contains("O''Brien"),
            "an embedded ' must be doubled per PowerShell quoting rules: {ps_command}"
        );
    }

    #[test]
    fn macos_admin_prompt_command_wraps_osascript_with_administrator_privileges() {
        let plan = LaunchPlan::MacOsAdminPrompt {
            program: program(),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan);
        assert_eq!(command.get_program(), "osascript");
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[0], "-e");
        assert!(args[1].contains("with administrator privileges"));
        assert!(args[1].contains("/opt/kagerou/sing-box"));
    }

    #[test]
    fn macos_admin_prompt_escapes_a_path_containing_a_double_quote_and_backslash() {
        let program = PathBuf::from("/tmp/weird\"path\\name/sing-box");
        let plan = LaunchPlan::MacOsAdminPrompt {
            program,
            args: vec![],
        };
        let command = to_command(&plan);
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let script = &args[1];
        // The AppleScript string literal itself must remain well-formed:
        // no unescaped `"` may appear inside it.
        let inner = script
            .strip_prefix("do shell script \"")
            .unwrap()
            .strip_suffix("\" with administrator privileges")
            .unwrap();
        assert!(
            inner.contains("\\\"path"),
            "the embedded double quote must be backslash-escaped: {inner}"
        );

        // No bare (unescaped) double-quote should remain in the inner text.
        let mut previous_was_backslash = false;
        for c in inner.chars() {
            if c == '"' && !previous_was_backslash {
                panic!("found an unescaped double quote in the AppleScript literal: {script}");
            }
            previous_was_backslash = c == '\\' && !previous_was_backslash;
        }
    }

    #[test]
    fn macos_shell_command_quotes_arguments_containing_spaces() {
        let plan = LaunchPlan::MacOsAdminPrompt {
            program: PathBuf::from("/opt/kagerou/sing-box"),
            args: vec!["-c".into(), "/path with spaces/config.json".into()],
        };
        let command = to_command(&plan);
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            args[1].contains("'/path with spaces/config.json'"),
            "the argument must stay a single shell token: {}",
            args[1]
        );
    }
}
