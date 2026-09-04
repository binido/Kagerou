use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, Sender};

use thiserror::Error;

use crate::privilege::{self, TargetOs};

const MAX_BUFFERED_LOG_LINES: usize = 500;

/// How long [`ChildHandle::kill`] waits for the process to be gone before
/// giving up. Short, because the app's shutdown hook blocks on it.
const KILL_CONFIRMATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

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
    /// Stops the process and waits until it is actually gone.
    ///
    /// The waiting is the point: the kill itself is carried out by the
    /// watcher thread, so returning as soon as the request was queued is
    /// what let sing-box outlive the app — at shutdown that thread dies
    /// with the process and the kill is never delivered.
    pub fn kill(&mut self) -> Result<(), ProcessError> {
        (self.kill)()?;
        let deadline = std::time::Instant::now() + KILL_CONFIRMATION_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match self.events.recv_timeout(remaining) {
                Ok(ProcessEvent::Exited { .. }) => return Ok(()),
                // Drain anything the process said on its way out.
                Ok(ProcessEvent::Log(_)) => continue,
                Err(_) => {
                    return Err(ProcessError::KillFailed(
                        "sing-box did not exit in time".to_string(),
                    ))
                }
            }
        }
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

/// Marks a live sing-box. Named per launch so a watchdog left over from a
/// crashed session never mistakes a new run's sentinel for its own.
const RUN_FILE_PREFIX: &str = "singbox-";
const RUN_FILE_SUFFIX: &str = ".run";

/// Reaps whatever a previous session left running and clears its sentinels.
/// Call it at startup: a crash, a force-quit or a SIGTERM can always strand
/// a sing-box, and until it goes it holds the proxy ports — or, elevated,
/// the TUN device and with it the machine's whole network.
///
/// The two kinds of leftover need opposite treatment. An elevated one is
/// not ours to signal, so deleting the file is the request and its own
/// watchdog does the killing. An unprivileged one has no watchdog, so its
/// PID is recorded in the file and killed here.
pub fn clear_run_files(run_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(run_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.starts_with(RUN_FILE_PREFIX) && name.ends_with(RUN_FILE_SUFFIX)) {
            continue;
        }
        if let Some(pid) = std::fs::read_to_string(entry.path())
            .ok()
            .and_then(|body| body.trim().parse::<u32>().ok())
        {
            reap(pid);
        }
        let _ = std::fs::remove_file(entry.path());
    }
}

/// Kills a leftover sing-box, but only once the PID is confirmed to still
/// belong to one. PIDs get recycled, and a stale file must never take an
/// unrelated process down with it.
fn reap(pid: u32) {
    if is_sing_box(pid) {
        kill_pid(pid);
    }
}

#[cfg(unix)]
fn is_sing_box(pid: u32) -> bool {
    Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("sing-box"))
        .unwrap_or(false)
}

#[cfg(unix)]
fn kill_pid(pid: u32) {
    let _ = Command::new("kill").arg(pid.to_string()).status();
}

// Same shape, built from the docs rather than from a live run: there is no
// Windows host here to check it against.
#[cfg(windows)]
fn is_sing_box(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("sing-box"))
        .unwrap_or(false)
}

#[cfg(windows)]
fn kill_pid(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status();
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
        let run_file = self.new_run_file()?;
        // Only an elevated launch gets the watchdog: unprivileged, the
        // process is our own child and a plain kill reaches it.
        let mut child = self
            .command_for(config_path, tun.then_some(run_file.as_path()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .inspect_err(|_| {
                let _ = std::fs::remove_file(&run_file);
            })
            .map_err(|e| ProcessError::SpawnFailed(e.to_string()))?;

        if !tun {
            // Recorded so a startup after a crash can find and kill it. An
            // elevated child's PID is not ours to signal, so that file stays
            // empty and its watchdog reacts to the deletion instead.
            let _ = std::fs::write(&run_file, child.id().to_string());
        }

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
                // it is watching this file. Unprivileged it just retires
                // the record, so a later startup has nothing to reap.
                let _ = std::fs::remove_file(&run_file);
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
        /// How long the fake child takes to die, mimicking a real one that
        /// only exits once the watcher thread gets round to killing it.
        exit_delay: Arc<StdMutex<Option<std::time::Duration>>>,
        /// A child that ignores the kill entirely, so the timeout path is
        /// reachable.
        never_exits: Arc<StdMutex<bool>>,
    }

    impl FakeControl {
        fn set_fail_next(&self) {
            *self.fail_next.lock().unwrap() = true;
        }

        fn kill_count(&self) -> usize {
            self.kill_count.load(Ordering::SeqCst)
        }

        fn set_exit_delay(&self, delay: std::time::Duration) {
            *self.exit_delay.lock().unwrap() = Some(delay);
        }

        fn set_never_exits(&self) {
            *self.never_exits.lock().unwrap() = true;
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
            *self.0.last_sender.lock().unwrap() = Some(tx.clone());
            let kill_count = Arc::clone(&self.0.kill_count);
            let exit_delay = Arc::clone(&self.0.exit_delay);
            let never_exits = Arc::clone(&self.0.never_exits);
            Ok(ChildHandle {
                events: rx,
                kill: Box::new(move || {
                    kill_count.fetch_add(1, Ordering::SeqCst);
                    if *never_exits.lock().unwrap() {
                        return Ok(());
                    }
                    let delay = *exit_delay.lock().unwrap();
                    let tx = tx.clone();
                    // Like the real watcher thread: the process goes away
                    // some time after the request, not during it.
                    std::thread::spawn(move || {
                        if let Some(delay) = delay {
                            std::thread::sleep(delay);
                        }
                        let _ = tx.send(ProcessEvent::Exited { code: None });
                    });
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

    /// The reaper must be able to tell a recycled PID from a real leftover,
    /// or a stale file becomes a licence to kill an unrelated process.
    #[test]
    #[cfg(unix)]
    fn a_recorded_pid_that_is_no_longer_sing_box_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let mut bystander = Command::new("/bin/sleep").arg("30").spawn().unwrap();
        std::fs::write(
            dir.path().join("singbox-abc.run"),
            bystander.id().to_string(),
        )
        .unwrap();

        clear_run_files(dir.path());

        assert!(
            bystander.try_wait().unwrap().is_none(),
            "a PID that is not sing-box must survive the reaper"
        );
        let _ = bystander.kill();
        let _ = bystander.wait();
    }

    /// And it must actually reap a real one. `ps` reports the executable, so
    /// a copy under sing-box's name is indistinguishable to the check —
    /// which is the point: this exercises the guard, not just the kill.
    #[test]
    #[cfg(unix)]
    fn a_leftover_sing_box_is_killed_at_startup() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("sing-box");
        std::fs::copy("/bin/sleep", &fake).unwrap();
        let mut leftover = Command::new(&fake).arg("30").spawn().unwrap();
        std::fs::write(
            dir.path().join("singbox-abc.run"),
            leftover.id().to_string(),
        )
        .unwrap();

        clear_run_files(dir.path());

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if leftover.try_wait().unwrap().is_some() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "a leftover sing-box must not survive startup"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
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

    /// The regression this whole change exists for: `stop` used to return
    /// as soon as the kill was queued, so the app could finish exiting
    /// before its watcher thread ever delivered it — leaving sing-box
    /// orphaned and still holding the tunnel.
    #[test]
    fn stop_waits_until_the_process_is_actually_gone() {
        let (mut sup, control) = supervisor();
        control.set_exit_delay(std::time::Duration::from_millis(300));
        sup.start(Path::new("/tmp/config.json"), false).unwrap();

        let before = std::time::Instant::now();
        sup.stop().unwrap();
        assert!(
            before.elapsed() >= std::time::Duration::from_millis(250),
            "stop returned before the process had exited"
        );
    }

    #[test]
    fn stop_gives_up_rather_than_hanging_on_a_process_that_will_not_die() {
        let (mut sup, control) = supervisor();
        control.set_never_exits();
        sup.start(Path::new("/tmp/config.json"), false).unwrap();
        assert!(matches!(sup.stop(), Err(ProcessError::KillFailed(_))));
        assert_eq!(
            *sup.status(),
            Status::Stopped,
            "the supervisor still has to let go of a child it cannot kill"
        );
    }
}
