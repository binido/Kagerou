use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::app_state::RuntimePaths;
use crate::singbox::{self, ConfigError, Launcher, ProcessError, Supervisor};
use crate::storage::models::Settings;
use crate::storage::{profiles, routing, settings, Db, StorageError};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error("could not write the sing-box config to {path}: {source}")]
    WriteConfig {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    Process(#[from] ProcessError),
}

/// How one sing-box process should run. Two of them exist at once: the
/// user's connection, and the short-lived core that answers latency tests.
pub struct CoreSpec<'a> {
    pub config_path: &'a Path,
    pub mixed_listen_port: u16,
    pub clash_api_listen: &'a str,
    pub log_level: &'a str,
    pub tun: bool,
    pub system_proxy: bool,
}

impl<'a> CoreSpec<'a> {
    /// The core behind the user's connection: the only one allowed to
    /// create a TUN device or repoint the OS proxy.
    pub fn connection(paths: &'a RuntimePaths, stored: &'a Settings) -> Self {
        Self {
            config_path: &paths.config_path,
            mixed_listen_port: paths.mixed_listen_port,
            clash_api_listen: &paths.clash_api_listen,
            log_level: &stored.log_level,
            tun: stored.tun_mode,
            system_proxy: stored.system_proxy,
        }
    }

    /// The core that exists only to measure profiles. Its own ports and its
    /// own config file, because it runs alongside a live connection.
    ///
    /// `tun` and `system_proxy` are hardcoded off and must stay that way:
    /// a TUN device asks for a password and rewrites the machine's routing,
    /// and the OS proxy belongs to the connection. Neither is something a
    /// delay test may do.
    pub fn test(paths: &'a RuntimePaths, stored: &'a Settings) -> Self {
        Self {
            config_path: &paths.test_config_path,
            mixed_listen_port: paths.test_mixed_listen_port,
            clash_api_listen: &paths.test_clash_api_listen,
            log_level: &stored.log_level,
            tun: false,
            system_proxy: false,
        }
    }
}

/// Generates the config this spec describes, writes it, and starts the core
/// on it.
///
/// Both cores come through here. They used to have a copy each, differing
/// only in ports and flags, which is the kind of split that does not fail
/// loudly: a fix applied to one copy leaves the other generating a config
/// that still starts and still routes, just not the way it was meant to.
pub fn start<L: Launcher>(
    db: &Db,
    supervisor: &mut Supervisor<L>,
    spec: &CoreSpec,
) -> Result<(), CoreError> {
    let all_profiles = profiles::list_all(db)?;
    let routing_rules = routing::list_rules(db)?;
    let active_profile_id = settings::get_active_profile_id(db)?.unwrap_or_default();

    let config = singbox::generate(&singbox::ConfigInput {
        profiles: &all_profiles,
        active_profile_id: &active_profile_id,
        routing_rules: &routing_rules,
        mixed_listen_port: spec.mixed_listen_port,
        clash_api_listen: spec.clash_api_listen,
        log_level: spec.log_level,
        tun: spec.tun,
        system_proxy: spec.system_proxy,
    })?;

    let bytes = serde_json::to_vec_pretty(&config).map_err(|e| CoreError::WriteConfig {
        path: spec.config_path.to_path_buf(),
        source: e.into(),
    })?;
    std::fs::write(spec.config_path, bytes).map_err(|e| CoreError::WriteConfig {
        path: spec.config_path.to_path_buf(),
        source: e,
    })?;

    supervisor.start(spec.config_path, spec.tun)?;
    Ok(())
}

#[cfg(test)]
mod tests;
