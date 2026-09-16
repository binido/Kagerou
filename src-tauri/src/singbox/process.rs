use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, Sender};

use thiserror::Error;

use super::system_proxy;
use crate::privilege::{self, TargetOs};

const MAX_BUFFERED_LOG_LINES: usize = 500;

/// How long [`ChildHandle::kill`] waits for the process to be gone before
/// giving up. Short, because the app's shutdown hook blocks on it, but longer
/// than [`GRACEFUL_STOP_TIMEOUT`] so the forced kill after it still counts.
const KILL_CONFIRMATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(4);

/// How long sing-box gets to shut down cleanly before it is killed outright.
const GRACEFUL_STOP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

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
    /// The only way to build one: `kill` is private, so a `Launcher` living
    /// outside this module (the fake one the tests use) has to come through
    /// here.
    pub fn new(
        events: Receiver<ProcessEvent>,
        kill: impl FnMut() -> Result<(), ProcessError> + Send + 'static,
    ) -> Self {
        Self {
            events,
            kill: Box::new(kill),
        }
    }

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
    /// The mixed inbound port this core listens on. A system proxy pointing
    /// at it is cleared once the process exits; see [`system_proxy`]. Each
    /// core passes its own, so the test core stopping never resets a proxy
    /// the main one set.
    pub system_proxy_port: u16,
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
        let proxy_port = self.system_proxy_port;
        std::thread::spawn(move || {
            let code = loop {
                if kill_rx.try_recv().is_ok() {
                    // An elevated child is only the osascript/pkexec wrapper,
                    // and the run file already told the real process to go.
                    if !tun {
                        terminate_gracefully(&mut child);
                    }
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                match child.try_wait() {
                    Ok(Some(status)) => break status.code(),
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(_) => break None,
                }
            };
            // Before `Exited`, which is what `stop` waits for: by the time it
            // returns, and before any restart, the proxy is back to direct.
            system_proxy::clear_if_ours(proxy_port);
            let _ = tx.send(ProcessEvent::Exited { code });
        });

        Ok(ChildHandle::new(rx, move || {
            // For an elevated launch this is the stop signal that
            // actually lands: killing our own child only reaches the
            // osascript/pkexec wrapper, while the root sing-box under
            // it is watching this file. Unprivileged it just retires
            // the record, so a later startup has nothing to reap.
            let _ = std::fs::remove_file(&run_file);
            kill_tx
                .send(())
                .map_err(|e| ProcessError::KillFailed(e.to_string()))
        }))
    }
}

/// Asks sing-box to exit before anything forces it to. It undoes the system
/// proxy only on a clean shutdown, and SIGKILL, which is all `Child::kill`
/// sends, would leave the OS pointed at a port nobody listens on any more.
/// Waiting on our own child is safe from PID reuse: until it is reaped, its
/// PID cannot be handed to another process.
#[cfg(unix)]
fn terminate_gracefully(child: &mut std::process::Child) {
    let _ = Command::new("kill").arg(child.id().to_string()).status();
    let deadline = std::time::Instant::now() + GRACEFUL_STOP_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if !matches!(child.try_wait(), Ok(None)) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Windows has no signal to send a windowless child, so the proxy is cleared
/// from outside once it is gone.
#[cfg(not(unix))]
fn terminate_gracefully(_child: &mut std::process::Child) {}

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
    /// How many lines the process has produced in total. The buffer is
    /// capped, so this keeps counting past what it still holds: a reader
    /// that tracked its position by buffer index instead went silent the
    /// moment the buffer filled up, because the index stopped moving.
    produced: usize,
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
            produced: 0,
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    /// The lines produced since `forwarded`, and the count to pass back next
    /// time. Lines that fell out of the buffer while the caller was away are
    /// gone, and the count moves past them so they are not asked for again.
    pub fn logs_since(&self, forwarded: usize) -> (impl Iterator<Item = &String>, usize) {
        let oldest_held = self.produced - self.logs.len();
        let skip = forwarded.saturating_sub(oldest_held);
        (self.logs.iter().skip(skip), self.produced)
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
                    self.produced += 1;
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
mod tests;
