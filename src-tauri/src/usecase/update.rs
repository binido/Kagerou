use serde::Serialize;
use tauri::utils::config::BundleType;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};

use super::connection;
use super::error::{AppError, ErrorCode};
use super::events::{AppEvent, Events, UpdateProgress};
use crate::app_state::AppState;
use crate::net::updates::{self, UpdateInfo};

/// A newer release, and whether this copy can install it by itself.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    #[serde(flatten)]
    pub release: UpdateInfo,
    pub installable: bool,
}

/// A downloaded, signature-checked release waiting for the restart.
pub struct PendingUpdate {
    update: Update,
    bytes: Vec<u8>,
}

/// Whether the updater has an installer to replace this copy with.
///
/// The bundler stamps the bundle type into the binary it packs and restores
/// the original afterwards, so the portable Windows exe and a dev build carry
/// none. Otherwise the updater would run the NSIS installer over a portable
/// copy.
fn installable(is_dev: bool, bundle: Option<BundleType>) -> bool {
    !is_dev && bundle.is_some()
}

/// Reports a newer release, if there is one. Silent on failure, like
/// `net::updates::check`.
pub async fn check(app: &AppHandle) -> Option<AvailableUpdate> {
    let release = updates::check(&app.package_info().version).await?;
    Some(AvailableUpdate {
        release,
        installable: installable(tauri::is_dev(), tauri::utils::platform::bundle_type()),
    })
}

/// Downloads the latest release and keeps it until `install` is called.
///
/// The VPN stays up here: the download can go through it, and nothing is
/// replaced until the user asks for the restart.
pub async fn download(app: &AppHandle, state: &AppState) -> Result<(), AppError> {
    let update =
        app.updater()?.check().await?.ok_or_else(|| {
            AppError::new(ErrorCode::UpdateFailed, "no newer release to download")
        })?;

    let mut downloaded = 0u64;
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                Events::emit(
                    app,
                    AppEvent::UpdateProgress(UpdateProgress { downloaded, total }),
                );
            },
            || {},
        )
        .await?;

    *state.pending_update.lock().unwrap() = Some(PendingUpdate { update, bytes });
    Ok(())
}

/// Installs the downloaded release and restarts into it.
pub async fn install(app: &AppHandle, state: &AppState) -> Result<(), AppError> {
    let pending =
        state.pending_update.lock().unwrap().take().ok_or_else(|| {
            AppError::new(ErrorCode::UpdateFailed, "nothing downloaded to install")
        })?;

    // The Windows installer ends the process with `exit(0)`, past the exit
    // hook that stops sing-box, and cannot overwrite a sing-box.exe that is
    // still running. Under TUN the orphan would keep holding the network.
    if state.is_connected() {
        connection::disconnect(app, state)?;
    }
    state.test_core.stop();

    if let Err(error) = pending.update.install(&pending.bytes) {
        *state.pending_update.lock().unwrap() = Some(pending);
        return Err(error.into());
    }
    app.restart()
}

#[cfg(test)]
mod tests;
