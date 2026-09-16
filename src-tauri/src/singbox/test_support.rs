//! A `Launcher` that answers like a real sing-box child without spawning
//! one, so supervisor behaviour and everything built on top of it can be
//! tested without a process, a config the core would accept, or root.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use super::process::{ChildHandle, Launcher, ProcessError, ProcessEvent, Supervisor};

#[derive(Clone, Default)]
pub(crate) struct FakeControl {
    fail_next: Arc<Mutex<bool>>,
    kill_count: Arc<AtomicUsize>,
    last_sender: Arc<Mutex<Option<Sender<ProcessEvent>>>>,
    /// How long the fake child takes to die, mimicking a real one that
    /// only exits once the watcher thread gets round to killing it.
    exit_delay: Arc<Mutex<Option<std::time::Duration>>>,
    /// A child that ignores the kill entirely, so the timeout path is
    /// reachable.
    never_exits: Arc<Mutex<bool>>,
}

impl FakeControl {
    pub(crate) fn set_fail_next(&self) {
        *self.fail_next.lock().unwrap() = true;
    }

    pub(crate) fn kill_count(&self) -> usize {
        self.kill_count.load(Ordering::SeqCst)
    }

    pub(crate) fn set_exit_delay(&self, delay: std::time::Duration) {
        *self.exit_delay.lock().unwrap() = Some(delay);
    }

    pub(crate) fn set_never_exits(&self) {
        *self.never_exits.lock().unwrap() = true;
    }

    pub(crate) fn send_event(&self, event: ProcessEvent) {
        self.last_sender
            .lock()
            .unwrap()
            .as_ref()
            .expect("no launch has happened yet")
            .send(event)
            .unwrap();
    }
}

pub(crate) struct FakeLauncher(FakeControl);

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
        Ok(ChildHandle::new(rx, move || {
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
        }))
    }
}

pub(crate) fn supervisor() -> (Supervisor<FakeLauncher>, FakeControl) {
    let control = FakeControl::default();
    (Supervisor::new(FakeLauncher(control.clone())), control)
}
