use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, Sender};

use thiserror::Error;

use crate::privilege::{self, TargetOs};

const MAX_BUFFERED_LOG_LINES: usize = 500;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("sing-box is already running")]
    AlreadyRunning,

    #[error("sing-box is not running")]
    NotRunning,

    #[error("failed to spawn sing-box: {0}")]
    SpawnFailed(String),

    #[error("failed to stop sing-box: {0}")]
    KillFailed(String),

    #[error("could not locate the bundled sing-box binary: {0}")]
    SidecarNotFound(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessEvent {
    Log(String),
    Exited { code: Option<i32> },
}

/// A spawned sing-box process: a stream of log/exit events, and a way to
/// kill it. Abstracted behind `Launcher` so the supervisor's lifecycle
/// logic can be unit tested without ever spawning a real binary.
pub struct ChildHandle {
    pub events: Receiver<ProcessEvent>,
    kill: Box<dyn FnMut() -> Result<(), ProcessError> + Send>,
}

impl ChildHandle {
    pub fn kill(&mut self) -> Result<(), ProcessError> {
        (self.kill)()
    }
}

pub trait Launcher: Send + Sync {
    /// `tun` says the process needs to create a TUN device, which needs
    /// privileges the app itself doesn't have — see `crate::privilege`.
    fn launch(&self, config_path: &Path, tun: bool) -> Result<ChildHandle, ProcessError>;
}

/// Resolves a `bundle.externalBin` sidecar: Tauri drops it next to the app's
/// own executable, with the target triple stripped from the name.
/// `tauri_plugin_shell` resolves it the same way but only hands back a
/// ready-to-spawn `Command`; we need the path itself, because TUN mode has to
/// wrap the binary in an elevation command of its own (see `crate::privilege`).
pub fn sidecar_path(name: &str) -> Result<PathBuf, ProcessError> {
    let exe = tauri::utils::platform::current_exe()
        .map_err(|e| ProcessError::SidecarNotFound(e.to_string()))?;
    let dir = exe
        .parent()
        .ok_or_else(|| ProcessError::SidecarNotFound(format!("{} has no parent", exe.display())))?;
    Ok(sidecar_in(dir, name))
}

fn sidecar_in(exe_dir: &Path, name: &str) -> PathBuf {
    let mut path = exe_dir.join(name);
    if cfg!(windows) {
        // Not `set_extension`: that would eat any dot already in the name.
        path.as_mut_os_string().push(".exe");
    }
    path
}

/// Spawns the real sing-box binary via `std::process::Command`, streaming
/// its combined stdout+stderr as `ProcessEvent::Log` lines from a
/// background thread, followed by a single `ProcessEvent::Exited` once the
/// process terminates (however it terminates: clean exit, crash, or being
/// killed out from under the wait thread).
pub struct SidecarLauncher {
    pub binary_path: PathBuf,
    /// Where to keep the run-file sentinel for an elevated launch. Its own
    /// directory rather than a fixed path so a leftover from a previous run
    /// can be spotted and cleared at startup — see [`clear_run_files`].
    pub run_dir: PathBuf,
}

/// Marks a live elevated sing-box. Named per launch so a watchdog left over
/// from a crashed session never mistakes a new run's sentinel for its own.
const RUN_FILE_PREFIX: &str = "singbox-";
const RUN_FILE_SUFFIX: &str = ".run";

/// Deletes every run-file sentinel in `run_dir`, which asks any elevated
/// sing-box left over from a previous session to exit. Call it at startup:
/// a crash or a force-quit can always strand one, and it holds the TUN
/// device (and with it the machine's whole network) until it goes.
pub fn clear_run_files(run_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(run_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(RUN_FILE_PREFIX) && name.ends_with(RUN_FILE_SUFFIX) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

impl SidecarLauncher {
    fn new_run_file(&self) -> Result<PathBuf, ProcessError> {
        let path = self.run_dir.join(format!(
            "{RUN_FILE_PREFIX}{}{RUN_FILE_SUFFIX}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&self.run_dir)
            .and_then(|()| std::fs::write(&path, b""))
            .map_err(|e| ProcessError::SpawnFailed(e.to_string()))?;
        Ok(path)
    }

    /// Without TUN the binary runs as-is and stays our own child. With it,
    /// the launch goes through `privilege::plan_launch`, which wraps it in
    /// the platform's privilege-escalation command (UAC / osascript /
    /// pkexec) — and `run_file` is how we ask that unreachable process to
    /// stop, since we can no longer signal it.
    fn command_for(&self, config_path: &Path, run_file: Option<&Path>) -> Command {
        let args = vec![
            "run".to_string(),
            "-c".to_string(),
            config_path.to_string_lossy().into_owned(),
        ];
        let plain = || {
            let mut command = Command::new(&self.binary_path);
            command.args(&args);
            command
        };
        let Some(run_file) = run_file else {
            return plain();
        };
        #[cfg(target_os = "linux")]
        let has_cap = privilege::current_process_has_cap_net_admin();
        #[cfg(not(target_os = "linux"))]
        let has_cap = false;
        match TargetOs::current() {
            Some(os) => privilege::to_command(
                &privilege::plan_launch(os, &self.binary_path, &args, has_cap),
                run_file,
            ),
            // ponytail: unknown OS — no escalation strategy to pick, so run
            // it plainly and let sing-box report the permission failure.
            None => plain(),
        }
    }
}

impl Launcher for SidecarLauncher {
    fn launch(&self, config_path: &Path, tun: bool) -> Result<ChildHandle, ProcessError> {
        let run_file = if tun {
            Some(self.new_run_file()?)
        } else {
            None
        };
        let mut child = self
            .command_for(config_path, run_file.as_deref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .inspect_err(|_| {
                if let Some(run_file) = &run_file {
                    let _ = std::fs::remove_file(run_file);
                }
            })
            .map_err(|e| ProcessError::SpawnFailed(e.to_string()))?;

        let (tx, rx) = std::sync::mpsc::channel();
        spawn_line_forwarder(child.stdout.take(), tx.clone());
        spawn_line_forwarder(child.stderr.take(), tx.clone());

        // A single thread owns `child` for its whole life. It polls
        // non-blockingly rather than calling the blocking `Child::wait()`,
        // so a `kill` request delivered via `kill_rx` is never stuck
        // behind a held lock on a process that never exits on its own.
        let (kill_tx, kill_rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || loop {
            if kill_rx.try_recv().is_ok() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = tx.send(ProcessEvent::Exited { code: None });
                break;
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    let _ = tx.send(ProcessEvent::Exited {
                        code: status.code(),
                    });
                    break;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(_) => {
                    let _ = tx.send(ProcessEvent::Exited { code: None });
                    break;
                }
            }
        });

        Ok(ChildHandle {
            events: rx,
            kill: Box::new(move || {
                // For an elevated launch this is the stop signal that
                // actually lands: killing our own child only reaches the
                // osascript/pkexec wrapper, while the root sing-box under
                // it is watching this file.
                if let Some(run_file) = &run_file {
                    let _ = std::fs::remove_file(run_file);
                }
                kill_tx
                    .send(())
                    .map_err(|e| ProcessError::KillFailed(e.to_string()))
            }),
        })
    }
}

fn spawn_line_forwarder(
    stream: Option<impl std::io::Read + Send + 'static>,
    tx: Sender<ProcessEvent>,
) {
    if let Some(stream) = stream {
        std::thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(ProcessEvent::Log(line)).is_err() {
                    break;
                }
            }
        });
    }
}

/// Owns the lifecycle of a single sing-box process: starting, stopping,
/// and reacting to an unexpected exit. Not internally thread-safe — like
/// `storage::Db`, callers share one instance behind a `Mutex` so every
/// transition (start/stop/poll) is serialized rather than racing.
pub struct Supervisor<L: Launcher> {
    launcher: L,
    child: Option<ChildHandle>,
    status: Status,
    logs: VecDeque<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Stopped,
    Running,
    Crashed { exit_code: Option<i32> },
}

impl<L: Launcher> Supervisor<L> {
    pub fn new(launcher: L) -> Self {
        Self {
            launcher,
            child: None,
            status: Status::Stopped,
            logs: VecDeque::new(),
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn recent_logs(&self) -> impl Iterator<Item = &String> {
        self.logs.iter()
    }

    pub fn start(&mut self, config_path: &Path, tun: bool) -> Result<(), ProcessError> {
        if matches!(self.status, Status::Running) {
            return Err(ProcessError::AlreadyRunning);
        }
        let child = self.launcher.launch(config_path, tun)?;
        self.child = Some(child);
        self.status = Status::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ProcessError> {
        match self.child.take() {
            None => Err(ProcessError::NotRunning),
            Some(mut child) => {
                let result = child.kill();
                self.status = Status::Stopped;
                result
            }
        }
    }

    /// Drains any pending events from the running child without blocking.
    /// Call this periodically (e.g. from a Tauri background task) to
    /// notice a crash and pick up log output.
    pub fn poll_events(&mut self) {
        let Some(child) = &self.child else { return };
        loop {
            match child.events.try_recv() {
                Ok(ProcessEvent::Log(line)) => {
                    self.logs.push_back(line);
                    if self.logs.len() > MAX_BUFFERED_LOG_LINES {
                        self.logs.pop_front();
                    }
                }
                Ok(ProcessEvent::Exited { code }) => {
                    self.status = Status::Crashed { exit_code: code };
                    self.child = None;
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};

    #[derive(Clone, Default)]
    struct FakeControl {
        fail_next: Arc<StdMutex<bool>>,
        kill_count: Arc<AtomicUsize>,
        last_sender: Arc<StdMutex<Option<Sender<ProcessEvent>>>>,
    }

    impl FakeControl {
        fn set_fail_next(&self) {
            *self.fail_next.lock().unwrap() = true;
        }

        fn kill_count(&self) -> usize {
            self.kill_count.load(Ordering::SeqCst)
        }

        fn send_event(&self, event: ProcessEvent) {
            self.last_sender
                .lock()
                .unwrap()
                .as_ref()
                .expect("no launch has happened yet")
                .send(event)
                .unwrap();
        }
    }

    struct FakeLauncher(FakeControl);

    impl Launcher for FakeLauncher {
        fn launch(&self, _config_path: &Path, _tun: bool) -> Result<ChildHandle, ProcessError> {
            if std::mem::replace(&mut *self.0.fail_next.lock().unwrap(), false) {
                return Err(ProcessError::SpawnFailed("fake failure".into()));
            }
            let (tx, rx) = std::sync::mpsc::channel();
            *self.0.last_sender.lock().unwrap() = Some(tx);
            let kill_count = Arc::clone(&self.0.kill_count);
            Ok(ChildHandle {
                events: rx,
                kill: Box::new(move || {
                    kill_count.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            })
        }
    }

    fn supervisor() -> (Supervisor<FakeLauncher>, FakeControl) {
        let control = FakeControl::default();
        (Supervisor::new(FakeLauncher(control.clone())), control)
    }

    #[test]
    fn start_sets_status_to_running() {
        let (mut sup, _control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        assert_eq!(*sup.status(), Status::Running);
    }

    #[test]
    fn starting_twice_is_an_error_and_does_not_disturb_the_first_child() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        let err = sup.start(Path::new("/tmp/config.json"), false).unwrap_err();
        assert!(matches!(err, ProcessError::AlreadyRunning));
        assert_eq!(*sup.status(), Status::Running);
        assert_eq!(control.kill_count(), 0);
    }

    #[test]
    fn stop_kills_the_child_and_sets_status_to_stopped() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        sup.stop().unwrap();
        assert_eq!(*sup.status(), Status::Stopped);
        assert_eq!(control.kill_count(), 1);
    }

    #[test]
    fn stopping_when_not_running_is_an_error() {
        let (mut sup, _control) = supervisor();
        assert!(matches!(sup.stop().unwrap_err(), ProcessError::NotRunning));
    }

    #[test]
    fn a_spawn_failure_leaves_status_stopped_and_is_reported() {
        let (mut sup, control) = supervisor();
        control.set_fail_next();
        let err = sup.start(Path::new("/tmp/config.json"), false).unwrap_err();
        assert!(matches!(err, ProcessError::SpawnFailed(_)));
        assert_eq!(*sup.status(), Status::Stopped);
    }

    #[test]
    fn poll_events_buffers_log_lines_in_order() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        control.send_event(ProcessEvent::Log("line 1".into()));
        control.send_event(ProcessEvent::Log("line 2".into()));
        sup.poll_events();
        let logs: Vec<_> = sup.recent_logs().cloned().collect();
        assert_eq!(logs, vec!["line 1".to_string(), "line 2".to_string()]);
    }

    #[test]
    fn poll_events_detects_a_crash_and_records_the_exit_code() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        control.send_event(ProcessEvent::Exited { code: Some(1) });
        sup.poll_events();
        assert_eq!(*sup.status(), Status::Crashed { exit_code: Some(1) });
    }

    #[test]
    fn the_log_buffer_is_capped_so_a_noisy_process_cannot_grow_it_unbounded() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        for i in 0..(MAX_BUFFERED_LOG_LINES + 50) {
            control.send_event(ProcessEvent::Log(format!("line {i}")));
        }
        sup.poll_events();
        assert_eq!(sup.recent_logs().count(), MAX_BUFFERED_LOG_LINES);
        assert_eq!(
            sup.recent_logs().next().unwrap(),
            "line 50",
            "oldest lines should be dropped first"
        );
    }

    #[test]
    fn can_restart_after_a_crash() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        control.send_event(ProcessEvent::Exited { code: Some(137) });
        sup.poll_events();
        assert_eq!(
            *sup.status(),
            Status::Crashed {
                exit_code: Some(137)
            }
        );

        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        assert_eq!(*sup.status(), Status::Running);
    }

    #[test]
    fn stopping_after_a_crash_is_an_error_since_there_is_no_live_child() {
        let (mut sup, control) = supervisor();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        control.send_event(ProcessEvent::Exited { code: None });
        sup.poll_events();
        assert!(matches!(sup.stop().unwrap_err(), ProcessError::NotRunning));
    }

    #[test]
    fn a_sidecar_sits_next_to_the_executable_under_its_platform_name() {
        let resolved = sidecar_in(Path::new("/apps/Kagerou.app/Contents/MacOS"), "sing-box");
        let expected = if cfg!(windows) {
            "/apps/Kagerou.app/Contents/MacOS/sing-box.exe"
        } else {
            "/apps/Kagerou.app/Contents/MacOS/sing-box"
        };
        assert_eq!(resolved, Path::new(expected));
    }

    fn sidecar_launcher(binary: &str, run_dir: &Path) -> SidecarLauncher {
        SidecarLauncher {
            binary_path: PathBuf::from(binary),
            run_dir: run_dir.to_path_buf(),
        }
    }

    /// On Linux with CAP_NET_ADMIN already set the plan is `Direct`, so
    /// the "not the bare binary" half of this only holds elsewhere.
    #[test]
    #[cfg(not(target_os = "linux"))]
    fn tun_mode_wraps_the_binary_in_an_elevation_command() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = sidecar_launcher("/opt/kagerou/sing-box", dir.path());
        let config = Path::new("/tmp/config.json");
        assert_eq!(
            launcher.command_for(config, None).get_program(),
            launcher.binary_path.as_os_str()
        );
        assert_ne!(
            launcher
                .command_for(config, Some(Path::new("/tmp/x.run")))
                .get_program(),
            launcher.binary_path.as_os_str(),
            "TUN needs root, so the binary must be wrapped in an elevation command"
        );
    }

    #[test]
    fn the_real_launcher_reports_a_spawn_failure_for_a_nonexistent_binary_without_touching_a_real_process(
    ) {
        let dir = tempfile::tempdir().unwrap();
        let launcher = sidecar_launcher("/definitely/not/a/real/sing-box/binary", dir.path());
        match launcher.launch(Path::new("/tmp/config.json"), false) {
            Err(ProcessError::SpawnFailed(_)) => {}
            other => panic!("expected SpawnFailed, got {}", other.is_ok()),
        }
    }

    /// Each launch gets its own sentinel, so a watchdog stranded by a
    /// crashed session can never mistake a later run's file for its own and
    /// keep the old process alive.
    #[test]
    fn every_launch_gets_its_own_run_file() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = sidecar_launcher("/opt/kagerou/sing-box", dir.path());
        let first = launcher.new_run_file().unwrap();
        let second = launcher.new_run_file().unwrap();
        assert_ne!(first, second);
        assert!(first.exists() && second.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    /// Deliberately not exercised by letting a real TUN launch fail:
    /// spawning the elevation command would raise an actual password prompt
    /// on the developer's machine. This covers the half reachable without
    /// one — the launch gives up before spawning anything.
    #[test]
    fn a_tun_launch_that_cannot_create_its_run_file_fails_before_spawning() {
        let dir = tempfile::tempdir().unwrap();
        let blocked = dir.path().join("not-a-dir");
        std::fs::write(&blocked, b"").unwrap();
        let launcher = sidecar_launcher("/opt/kagerou/sing-box", &blocked);
        assert!(matches!(
            launcher.launch(Path::new("/tmp/config.json"), true),
            Err(ProcessError::SpawnFailed(_))
        ));
    }

    #[test]
    fn a_non_tun_launch_creates_no_run_file() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = sidecar_launcher("/definitely/not/a/real/sing-box/binary", dir.path());
        let _ = launcher.launch(Path::new("/tmp/config.json"), false);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn clear_run_files_removes_sentinels_and_leaves_everything_else_alone() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("singbox-abc.run"), b"").unwrap();
        std::fs::write(dir.path().join("singbox-def.run"), b"").unwrap();
        std::fs::write(dir.path().join("kagerou.sqlite3"), b"db").unwrap();
        std::fs::write(dir.path().join("sing-box-config.json"), b"{}").unwrap();

        clear_run_files(dir.path());

        let mut left: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, vec!["kagerou.sqlite3", "sing-box-config.json"]);
    }

    #[test]
    fn clearing_a_directory_that_does_not_exist_is_not_an_error() {
        clear_run_files(Path::new("/definitely/not/a/real/directory"));
    }
}
