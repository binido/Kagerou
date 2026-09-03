pub mod app_state;
pub mod clash_api;
pub mod commands;
pub mod privilege;
pub mod singbox;
pub mod storage;
pub mod subscription;

use tauri::Manager;

use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db = storage::Db::open(app_data_dir.join("kagerou.sqlite3"))?;
            let config_path = app_data_dir.join("sing-box-config.json");
            // "sing-box" is resolved via PATH at spawn time; once the real
            // binary is bundled as a Tauri sidecar this should become the
            // resolved sidecar path instead (see CLAUDE.md's notes on
            // stage 9's known limitations).
            let sing_box_binary = std::path::PathBuf::from("sing-box");

            app.manage(AppState::new(db, sing_box_binary, config_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
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
        .run(tauri::generate_context!())
        .expect("error while running kagerou");
}
