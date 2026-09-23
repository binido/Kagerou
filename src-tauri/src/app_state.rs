use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tokio::sync::watch;

use crate::clash_api::ClashApiClient;
use crate::singbox::{SidecarLauncher, Supervisor};
use crate::storage::Db;
use crate::usecase::testing::TestCore;
use crate::usecase::update::PendingUpdate;

/// Paths and test-core addresses resolved once at startup. The connection's
/// own ports are settings, read on every connect.
pub struct RuntimePaths {
    pub sing_box_binary: PathBuf,
    pub config_path: PathBuf,
    /// The test core runs alongside a live connection, so it cannot share the
    /// connection's ports or its config file.
    pub test_config_path: PathBuf,
    pub test_clash_api_listen: String,
    pub test_mixed_listen_port: u16,
}

pub struct AppState {
    pub db: Db,
    pub supervisor: Mutex<Supervisor<SidecarLauncher>>,
    pub clash: Mutex<Option<ClashApiClient>>,
    pub traffic_stop: Mutex<Option<watch::Sender<bool>>>,
    /// When the running connection came up, so the dashboard's uptime is
    /// measured by whoever kept counting rather than by the WebView, which
    /// forgets everything on a reload.
    pub connected_since: Mutex<Option<SystemTime>>,
    /// The core that answers latency tests. Its own process, its own ports,
    /// and not the connection: see `usecase::testing`.
    pub test_core: Arc<TestCore>,
    pub pending_update: Mutex<Option<PendingUpdate>>,
    pub paths: RuntimePaths,
}

impl AppState {
    pub fn new(db: Db, sing_box_binary: PathBuf, config_path: PathBuf) -> Self {
        // Alongside the generated config: an app-owned directory that
        // survives for the life of the install, which is what the run-file
        // sentinel needs (see singbox::process::clear_run_files).
        let run_dir = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let test_config_path = config_path.with_file_name("sing-box-test-config.json");
        let paths = RuntimePaths {
            sing_box_binary: sing_box_binary.clone(),
            config_path,
            test_config_path,
            // Reserved by migration 0014, so a user port never collides.
            test_clash_api_listen: "127.0.0.1:9091".to_string(),
            test_mixed_listen_port: 2081,
        };
        Self {
            db,
            supervisor: Mutex::new(Supervisor::new(SidecarLauncher {
                binary_path: sing_box_binary.clone(),
                run_dir: run_dir.clone(),
                // Replaced from settings on every connect.
                system_proxy_port: 0,
            })),
            clash: Mutex::new(None),
            traffic_stop: Mutex::new(None),
            connected_since: Mutex::new(None),
            test_core: Arc::new(TestCore::new(SidecarLauncher {
                binary_path: sing_box_binary,
                run_dir,
                system_proxy_port: paths.test_mixed_listen_port,
            })),
            pending_update: Mutex::new(None),
            paths,
        }
    }

    /// Clones out the current Clash API client handle (cheap: it's just a
    /// base URL, a timeout, and a `reqwest::Client`, which is itself a
    /// cheaply-cloneable handle to a shared connection pool), so a command
    /// can make its network call without holding the state-wide lock for
    /// the duration of that call.
    pub fn clash_client(&self) -> Option<ClashApiClient> {
        self.clash.lock().unwrap().clone()
    }

    /// Whether the user's own core is up. Asked in several commands, and
    /// worth one name: the test core being up is not the same thing.
    pub fn is_connected(&self) -> bool {
        matches!(
            *self.supervisor.lock().unwrap().status(),
            crate::singbox::Status::Running
        )
    }
}
