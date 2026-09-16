pub mod config;
pub mod match_spec;
mod outbound_json;
pub mod process;
pub mod system_proxy;
#[cfg(test)]
pub(crate) mod test_support;

pub use config::{generate, ConfigError, ConfigInput};
pub use process::{
    clear_run_files, sidecar_path, ChildHandle, Launcher, ProcessError, ProcessEvent,
    SidecarLauncher, Status, Supervisor,
};
