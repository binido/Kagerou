//! Measuring profiles: the short-lived core that answers latency tests, and
//! the run that walks a group through it.
//!
//! Never the connection's own core, even when one is running. Aiming a test
//! at a particular server means pointing the selector at it, and doing that
//! to a live tunnel would silently reroute the user's traffic through
//! whatever is being measured.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::watch;

use super::core::{self, CoreSpec};
use super::events::{AppEvent, Events, TestFinished, TestProgress};
use crate::app_state::RuntimePaths;
use crate::clash_api::ClashApiClient;
use crate::probe;
use crate::singbox::{SidecarLauncher, Supervisor};
use crate::storage::models::{TestResult, Tone};
use crate::storage::{profiles, settings, Db};

/// How long a test-only core lingers after the last test before shutting
/// itself down. Long enough that testing profiles one at a time does not
/// restart it each time, short enough that nothing is left running.
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const READY_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const REAPER_TICK: Duration = Duration::from_secs(5);

/// Everything the test core needs to exist, kept together rather than as six
/// loose fields on the application state. It is not the connection: no TUN,
/// nothing announced to the UI, and it shuts itself down once the tests stop
/// coming.
pub struct TestCore {
    supervisor: Mutex<Supervisor<SidecarLauncher>>,
    clash: Mutex<Option<ClashApiClient>>,
    last_request_at: Mutex<Option<Instant>>,
    /// Serialises starting the core, so a group test firing dozens of
    /// concurrent requests starts exactly one.
    start_gate: tokio::sync::Mutex<()>,
    /// Set while a group run is walking its profiles; sending on it asks the
    /// run to stop, and its presence is the "already running" guard.
    run_cancel: Mutex<Option<watch::Sender<bool>>>,
}

impl TestCore {
    pub fn new(launcher: SidecarLauncher) -> Self {
        Self {
            supervisor: Mutex::new(Supervisor::new(launcher)),
            clash: Mutex::new(None),
            last_request_at: Mutex::new(None),
            start_gate: tokio::sync::Mutex::new(()),
            run_cancel: Mutex::new(None),
        }
    }

    /// The client for a core that is already up, if there is one. Cloned out
    /// rather than borrowed so a caller can make its network call without
    /// holding the lock for the duration.
    pub fn client(&self) -> Option<ClashApiClient> {
        self.clash.lock().unwrap().clone()
    }

    /// Takes the core down. A no-op when there isn't one.
    pub fn stop(&self) {
        if self.clash.lock().unwrap().take().is_none() {
            return;
        }
        let _ = self.supervisor.lock().unwrap().stop();
        *self.last_request_at.lock().unwrap() = None;
    }

    /// Claims the single run slot, handing back the receiver that says when
    /// to stop. `None` when a run is already under way.
    fn begin_run(&self) -> Option<watch::Receiver<bool>> {
        let mut slot = self.run_cancel.lock().unwrap();
        if slot.is_some() {
            return None;
        }
        let (tx, rx) = watch::channel(false);
        *slot = Some(tx);
        Some(rx)
    }

    fn end_run(&self) {
        *self.run_cancel.lock().unwrap() = None;
    }

    /// Asks a running group test to stop after the profile it is on. A no-op
    /// when nothing is running.
    pub fn cancel_run(&self) {
        if let Some(cancel) = self.run_cancel.lock().unwrap().as_ref() {
            let _ = cancel.send(true);
        }
    }
}

/// Whether a test core has gone long enough without a request to be worth
/// shutting down. No recorded request at all counts as idle: it means the
/// core outlived whatever started it.
fn is_idle(last_request_at: Option<Instant>, now: Instant, timeout: Duration) -> bool {
    match last_request_at {
        None => true,
        Some(at) => now.duration_since(at) >= timeout,
    }
}

fn latency_tone(delay_ms: u32) -> Tone {
    if delay_ms < 150 {
        Tone::Good
    } else if delay_ms < 400 {
        Tone::Warn
    } else {
        Tone::Bad
    }
}

/// What asking the test core about one profile produced. A core that would
/// not come up is not the profile's fault, and is kept apart from a profile
/// that genuinely did not answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Measured {
    Result(TestResult),
    CoreUnavailable,
}

impl Measured {
    fn into_result(self) -> TestResult {
        match self {
            Self::Result(result) => result,
            // Report the profile as untested rather than blaming it for the
            // run's problem.
            Self::CoreUnavailable => TestResult {
                value: "Not tested".to_string(),
                tone: Tone::Muted,
            },
        }
    }
}

/// Walks `profile_ids`, reporting each result as it lands and stopping when
/// asked.
///
/// The run lives in the backend rather than the frontend firing one command
/// per profile: aiming a test means pointing the test core's selector at one
/// profile, so they cannot overlap, and a sequence of hundreds needs
/// somewhere to report progress from and something to stop it with.
///
/// `measure` is a parameter so the walk itself — cancellation, counting,
/// which results are worth storing — can be tested without a core.
pub async fn run_group<E, F, Fut>(
    db: &Db,
    core: &TestCore,
    events: &E,
    mut cancel: watch::Receiver<bool>,
    profile_ids: Vec<String>,
    measure: F,
) where
    E: Events,
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Measured>,
{
    let total = profile_ids.len();
    let mut done = 0usize;
    let mut cancelled = false;

    for profile_id in profile_ids {
        if *cancel.borrow_and_update() {
            cancelled = true;
            break;
        }
        let result = measure(profile_id.clone()).await.into_result();
        // A core that never came up says nothing about the profile, so the
        // stored result stays whatever it was.
        if result.tone != Tone::Muted {
            let _ = profiles::set_test_result(db, &profile_id, &result);
        }
        done += 1;
        events.emit(AppEvent::TestProgress(TestProgress {
            profile_id,
            result,
            done,
            total,
        }));
    }

    core.end_run();
    events.emit(AppEvent::TestFinished(TestFinished {
        done,
        total,
        cancelled,
    }));
}

/// Claims the run slot for `profile_ids`, or reports that one is already
/// under way.
pub fn begin_run(core: &TestCore) -> Result<watch::Receiver<bool>, String> {
    core.begin_run()
        .ok_or_else(|| "a test run is already in progress".to_string())
}

/// Brings the core up if it is not already, and hands back a client for it.
pub async fn ensure_running(
    db: &Db,
    paths: &RuntimePaths,
    core: &Arc<TestCore>,
) -> Result<ClashApiClient, String> {
    *core.last_request_at.lock().unwrap() = Some(Instant::now());

    // One starter at a time: a group test fires dozens of these at once.
    let _gate = core.start_gate.lock().await;
    if let Some(clash) = core.client() {
        return Ok(clash);
    }

    let stored = settings::get(db).map_err(|e| e.to_string())?;
    core::start(
        db,
        &mut core.supervisor.lock().unwrap(),
        &CoreSpec::test(paths, &stored),
    )
    .map_err(|e| e.to_string())?;

    let clash = ClashApiClient::new(format!("http://{}", paths.test_clash_api_listen));
    // The API is not up the instant the process is: poll until it answers,
    // or the first delay request fails for a reason that has nothing to do
    // with the profile being tested.
    let ready = tokio::time::timeout(READY_TIMEOUT, async {
        while clash.get_version().await.is_err() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;
    if ready.is_err() {
        let _ = core.supervisor.lock().unwrap().stop();
        return Err("the proxy core did not come up for testing".to_string());
    }

    *core.clash.lock().unwrap() = Some(clash.clone());
    spawn_reaper(Arc::clone(core));
    Ok(clash)
}

/// Round trip through one profile, measured by sending a real request over
/// the test core's own inbound. Aiming at one profile means pointing that
/// core's selector at it, which is why tests run one at a time.
pub async fn measure_profile(
    db: &Db,
    socks_port: u16,
    clash: &ClashApiClient,
    profile_id: &str,
) -> Result<TestResult, String> {
    let test_url = settings::get(db).map_err(|e| e.to_string())?.test_url;
    if clash.select_outbound("proxy", profile_id).await.is_err() {
        return Ok(TestResult {
            value: "Unavailable".to_string(),
            tone: Tone::Bad,
        });
    }

    let socks = format!("127.0.0.1:{socks_port}");
    Ok(
        match probe::rtt_through_socks(&socks, &test_url, REQUEST_TIMEOUT).await {
            Ok(delay_ms) => TestResult {
                value: format!("{delay_ms} ms"),
                tone: latency_tone(delay_ms),
            },
            Err(probe::ProbeError::Timeout) => TestResult {
                value: "Timeout".to_string(),
                tone: Tone::Bad,
            },
            Err(probe::ProbeError::Unreachable) => TestResult {
                value: "No response".to_string(),
                tone: Tone::Bad,
            },
        },
    )
}

/// Shuts the test-only core down once the tests stop arriving.
fn spawn_reaper(core: Arc<TestCore>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(REAPER_TICK).await;
            if core.client().is_none() {
                return;
            }
            let last = *core.last_request_at.lock().unwrap();
            if is_idle(last, Instant::now(), IDLE_TIMEOUT) {
                core.stop();
                return;
            }
        }
    });
}

#[cfg(test)]
mod tests;
