pub mod app_state;
pub mod clash_api;
pub mod commands;
pub mod privilege;
pub mod singbox;
pub mod storage;
pub mod subscription;
pub mod updates;

use tauri::Manager;

use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            // A crash or a force-quit can strand an elevated sing-box from a
            // previous session, and it holds the TUN device — and with it
            // the machine's whole network — until it goes. Dropping its run
            // file is how we ask it to exit; see singbox::process.
            singbox::clear_run_files(&app_data_dir);

            let db = storage::Db::open(app_data_dir.join("kagerou.sqlite3"))?;
            let config_path = app_data_dir.join("sing-box-config.json");
            let sing_box_binary = singbox::sidecar_path("sing-box")?;

            app.manage(AppState::new(db, sing_box_binary, config_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::check_for_update,
            commands::connect,
            commands::disconnect,
            commands::select_profile,
            commands::add_local_profile,
            commands::rename_profile,
            commands::delete_profile,
            commands::move_profile_to_group,
            commands::move_profile,
            commands::reorder_profiles,
            commands::run_profile_test,
            commands::clear_test_results,
            commands::delete_unavailable_profiles,
            commands::set_profile_group_open,
            commands::add_profile_group,
            commands::rename_profile_group,
            commands::validate_source,
            commands::add_source,
            commands::update_source,
            commands::refresh_source,
            commands::remove_source,
            commands::set_preset,
            commands::select_rule,
            commands::update_rule,
            commands::add_routing_rule,
            commands::set_theme,
            commands::update_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building kagerou")
        .run(|app, event| {
            // Without this, sing-box outlives the window: unprivileged it
            // just lingers, and under TUN it runs as root and keeps the
            // tunnel up with no way left to reach it from the UI.
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<AppState>() {
                    let _ = state.supervisor.lock().unwrap().stop();
                }
            }
        });
}
