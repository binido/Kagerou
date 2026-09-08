//! The tray icon: the app's other face, for when its window is closed.
//!
//! A VPN client spends most of its life not being looked at, so closing the
//! window hides it here rather than dropping the connection. What the menu
//! offers is what someone would open the window for anyway — connect, switch
//! to one of the servers they actually use, get the window back.

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::app_state::AppState;
use crate::storage::profiles;

/// How many recently used profiles the menu offers. Enough for the handful
/// someone rotates between, short enough to stay a menu.
const RECENT_PROFILES: usize = 6;

const ID_TOGGLE: &str = "tray-toggle-connection";
const ID_SHOW: &str = "tray-show-window";
const ID_QUIT: &str = "tray-quit";
const PROFILE_PREFIX: &str = "tray-profile:";

/// macOS renders a template image in the menu bar's own colour, inverting it
/// for a dark bar; Windows and Linux draw the icon as given, so they get the
/// coloured mark instead.
fn icon_for(connected: bool) -> Image<'static> {
    let bytes: &[u8] = if cfg!(target_os = "macos") {
        if connected {
            include_bytes!("../icons/tray/mac-connected.png")
        } else {
            include_bytes!("../icons/tray/mac-disconnected.png")
        }
    } else if connected {
        include_bytes!("../icons/tray/connected.png")
    } else {
        include_bytes!("../icons/tray/disconnected.png")
    };
    Image::from_bytes(bytes).expect("tray icons are compiled in and known-good PNGs")
}

fn build_menu<R: Runtime>(app: &AppHandle<R>, connected: bool) -> tauri::Result<Menu<R>> {
    let toggle = MenuItem::with_id(
        app,
        ID_TOGGLE,
        if connected { "Disconnect" } else { "Connect" },
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&toggle, &PredefinedMenuItem::separator(app)?])?;

    // Recently used, not all of them: a subscription runs to hundreds and a
    // menu that long is unusable on every platform.
    let recent = app
        .try_state::<AppState>()
        .and_then(|state| profiles::recently_selected(&state.db, RECENT_PROFILES).ok())
        .unwrap_or_default();
    if !recent.is_empty() {
        let items: Vec<MenuItem<R>> = recent
            .iter()
            .map(|profile| {
                MenuItem::with_id(
                    app,
                    format!("{PROFILE_PREFIX}{}", profile.id),
                    &profile.name,
                    // The active one stays visible but unclickable: switching
                    // to the profile already in use does nothing worth a
                    // reconnect.
                    !profile.selected,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<_>>()?;
        let refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = items
            .iter()
            .map(|i| i as &dyn tauri::menu::IsMenuItem<R>)
            .collect();
        let submenu = Submenu::with_items(app, "Recent servers", true, &refs)?;
        menu.append(&submenu)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    menu.append(&MenuItem::with_id(
        app,
        ID_SHOW,
        "Show Kagerou",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_QUIT,
        "Quit",
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

/// Creates the tray. Called once at startup.
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let menu = build_menu(app, false)?;
    let tray = TrayIconBuilder::with_id("main")
        .icon(icon_for(false))
        .icon_as_template(true)
        .tooltip("Kagerou")
        .menu(&menu)
        // Left-clicking a tray icon opens the window on Windows and Linux;
        // on macOS the same click opens the menu, which is that platform's
        // convention, so this is left off there.
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left && !cfg!(target_os = "macos") {
                    show_window(tray.app_handle());
                }
            }
        })
        .build(app)?;
    Ok(tray)
}

/// Redraws the tray for a new connection state: the icon says whether the
/// tunnel is up, and the first menu item has to offer the opposite of what
/// is currently true.
pub fn refresh<R: Runtime>(app: &AppHandle<R>, connected: bool) {
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };
    // Not `set_icon`: on macOS that one hands the image to the status bar
    // with the template flag hardcoded off, so the black silhouette would
    // vanish into a dark menu bar. This sets both together, which also
    // spares the redraw the two separate calls would cost.
    let _ = tray.set_icon_with_as_template(Some(icon_for(connected)), true);
    if let Ok(menu) = build_menu(app, connected) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn show_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref().to_string();
    match id.as_str() {
        ID_SHOW => show_window(app),
        // Quitting is the one way out, since closing the window only hides
        // it. Nothing is saved on the way: every mutation is already written
        // when it happens.
        ID_QUIT => app.exit(0),
        ID_TOGGLE => emit_intent(app, "kagerou://tray-toggle-connection", ()),
        _ => {
            if let Some(profile_id) = id.strip_prefix(PROFILE_PREFIX) {
                emit_intent(app, "kagerou://tray-select-profile", profile_id.to_string());
            }
        }
    }
}

/// The tray asks the frontend to act rather than acting itself. Connecting and
/// switching profiles are already implemented there, on top of the same
/// commands, and a second path through the backend would be a second set of
/// bugs.
fn emit_intent<R: Runtime, T: serde::Serialize + Clone>(
    app: &AppHandle<R>,
    event: &str,
    payload: T,
) {
    use tauri::Emitter;
    let _ = app.emit(event, payload);
}
