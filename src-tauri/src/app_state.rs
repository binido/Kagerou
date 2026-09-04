use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tokio::sync::watch;

use crate::clash_api::ClashApiClient;
use crate::singbox::{SidecarLauncher, Supervisor};
use crate::storage::Db;

/// Paths and network addresses resolved once at startup (sidecar binary
/// location, where the generated sing-box config is written, and the
/// loopback ports sing-box's inbounds/Clash API listen on).
pub struct RuntimePaths {
    pub sing_box_binary: PathBuf,
    pub config_path: PathBuf,
    pub clash_api_listen: String,
    pub mixed_listen_port: u16,
}

pub struct AppState {
    pub db: Db,
    pub supervisor: Mutex<Supervisor<SidecarLauncher>>,
    pub clash: Mutex<Option<ClashApiClient>>,
    pub traffic_stop: Mutex<Option<watch::Sender<bool>>>,
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
        let paths = RuntimePaths {
            sing_box_binary: sing_box_binary.clone(),
            config_path,
            clash_api_listen: "127.0.0.1:9090".to_string(),
            mixed_listen_port: 2080,
        };
        Self {
            db,
            supervisor: Mutex::new(Supervisor::new(SidecarLauncher {
                binary_path: sing_box_binary,
                run_dir,
            })),
            clash: Mutex::new(None),
            traffic_stop: Mutex::new(None),
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
}
