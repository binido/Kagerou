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

/// A POSIX `sh` program that runs sing-box and then outlives it only as
/// long as `run_file` exists.
///
/// The elevated process is not ours to signal: `osascript`/`pkexec` hand
/// privileges to a process that is reparented away from us, so an
/// unprivileged `kill` from the app can never reach it (this is exactly how
/// a root sing-box used to survive the app and keep the machine offline).
/// Instead the privileged side watches a file the app owns: deleting it —
/// on disconnect, on shutdown, or on the next startup after a crash — is
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
        // plain kill reaches it — no sentinel needed.
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
mod tests {
    use super::*;

    fn program() -> PathBuf {
        PathBuf::from("/opt/kagerou/sing-box")
    }

    fn run_file() -> &'static Path {
        Path::new("/tmp/kagerou/singbox-1.run")
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
        let command = to_command(&plan, run_file());
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
        let command = to_command(&plan, run_file());
        assert_eq!(command.get_program(), "pkexec");
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[0], "/bin/sh");
        assert_eq!(args[1], "-c");
        assert!(
            args[2].starts_with("'/opt/kagerou/sing-box' 'run' '-c' 'config.json' &"),
            "the program still runs, just under a watchdog: {}",
            args[2]
        );
    }

    #[test]
    fn windows_run_as_command_invokes_powershell_start_process_with_verb_runas() {
        let plan = LaunchPlan::WindowsRunAs {
            program: PathBuf::from(r"C:\Program Files\Kagerou\sing-box.exe"),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan, run_file());
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
            // Doubled quotes: the watchdog is itself a PowerShell string that
            // gets re-parsed by the elevated shell.
            ps_command.contains("''run'',''-c'',''config.json''"),
            "args must be passed through: {ps_command}"
        );
    }

    #[test]
    fn windows_run_as_command_escapes_an_embedded_single_quote_in_a_path() {
        let plan = LaunchPlan::WindowsRunAs {
            program: PathBuf::from(r"C:\Users\O'Brien\sing-box.exe"),
            args: vec![],
        };
        let command = to_command(&plan, run_file());
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let ps_command = args.last().unwrap();
        assert!(
            ps_command.contains("O''''Brien"),
            "an embedded ' must be doubled per PowerShell quoting rules: {ps_command}"
        );
    }

    #[test]
    fn macos_admin_prompt_command_wraps_osascript_with_administrator_privileges() {
        let plan = LaunchPlan::MacOsAdminPrompt {
            program: program(),
            args: vec!["run".into(), "-c".into(), "config.json".into()],
        };
        let command = to_command(&plan, run_file());
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
        let command = to_command(&plan, run_file());
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
        let command = to_command(&plan, run_file());
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

    /// The whole point of the sentinel: an unprivileged app cannot signal a
    /// root process, so the privileged side has to stop itself when the file
    /// the app owns disappears.
    #[test]
    fn an_elevated_launch_stops_itself_when_the_run_file_is_deleted() {
        for plan in [
            LaunchPlan::MacOsAdminPrompt {
                program: program(),
                args: vec!["run".into()],
            },
            LaunchPlan::LinuxPolkit {
                program: program(),
                args: vec!["run".into()],
            },
        ] {
            let command = to_command(&plan, run_file());
            let script = command
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(
                script.contains("/tmp/kagerou/singbox-1.run"),
                "{plan:?} must watch the run file: {script}"
            );
            assert!(
                script.contains("kill $child"),
                "{plan:?} must kill sing-box once the file is gone: {script}"
            );
        }
    }

    #[test]
    fn the_windows_watchdog_is_itself_elevated_so_it_can_stop_the_process() {
        let plan = LaunchPlan::WindowsRunAs {
            program: PathBuf::from(r"C:\Kagerou\sing-box.exe"),
            args: vec!["run".into()],
        };
        let command = to_command(&plan, Path::new(r"C:\Users\b\singbox-1.run"));
        let script = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        // -Verb RunAs is on the powershell that runs the watchdog, not on
        // sing-box itself: an unprivileged parent cannot Stop-Process an
        // elevated child.
        let runas = script.find("-Verb RunAs").expect("must trigger UAC");
        let stop = script.find("Stop-Process").expect("must stop the process");
        assert!(
            stop < runas,
            "Stop-Process belongs inside the elevated command: {script}"
        );
        assert!(
            script.contains("Test-Path"),
            "must watch the run file: {script}"
        );
    }

    /// macOS runs the watchdog through AppleScript, so anything the shell
    /// snippet adds still has to survive the AppleScript string escape.
    #[test]
    fn the_macos_watchdog_stays_inside_one_applescript_string() {
        let plan = LaunchPlan::MacOsAdminPrompt {
            program: PathBuf::from("/Applications/Kagerou.app/sing-box"),
            args: vec!["run".into(), "-c".into(), "/tmp/a b/config.json".into()],
        };
        let command = to_command(&plan, run_file());
        let script = command
            .get_args()
            .last()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(script.starts_with("do shell script \""));
        assert!(script.ends_with("\" with administrator privileges"));
        assert_eq!(
            script.matches('"').count(),
            2,
            "an unescaped quote would end the AppleScript string early: {script}"
        );
        assert!(script.contains("'/tmp/a b/config.json'"), "{script}");
    }
}
