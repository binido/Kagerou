
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
        system_proxy_port: 2080,
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

/// And it must actually reap a real one. `ps` reports the name the
/// process was executed under, so something launched via a path ending
/// in `sing-box` is indistinguishable to the check — which is the point:
/// this exercises the guard, not just the kill.
///
/// A symlink, not a copy: copying opens the destination for writing, and
/// a sibling test forking in another thread inherits that descriptor for
/// the moment between its fork and its exec. The file is then still open
/// for writing when this test execs it, and Linux answers ETXTBSY.
#[test]
#[cfg(unix)]
fn a_leftover_sing_box_is_killed_at_startup() {
    let dir = tempfile::tempdir().unwrap();
    let fake = dir.path().join("sing-box");
    std::os::unix::fs::symlink("/bin/sleep", &fake).unwrap();
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

/// SIGKILL cannot be trapped, so a fake that records a trapped TERM
/// proves the process was given the chance to undo the system proxy.
#[test]
#[cfg(unix)]
fn an_unprivileged_stop_lets_the_process_shut_down_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("terminated");
    let script = dir.path().join("fake-sing-box");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\ntrap 'touch {}; exit 0' TERM\nwhile :; do sleep 0.05; done\n",
            marker.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let mut sup = Supervisor::new(sidecar_launcher(
        script.to_str().unwrap(),
        &dir.path().join("run"),
    ));
    sup.start(Path::new("/tmp/config.json"), false).unwrap();
    // Let the shell install its trap before it is signalled.
    std::thread::sleep(std::time::Duration::from_millis(300));

    sup.stop().unwrap();

    assert!(marker.exists(), "sing-box must get SIGTERM before SIGKILL");
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
