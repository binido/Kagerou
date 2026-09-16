use super::*;
use crate::app_state::RuntimePaths;
use crate::singbox::test_support::supervisor;
use crate::singbox::Status;
use crate::storage::models::NewProfile;
use crate::storage::models::Protocol;
use crate::storage::settings::SettingsPatch;

fn db_with_a_profile() -> Db {
    let db = Db::open_in_memory().unwrap();
    profiles::insert(
        &db,
        &NewProfile {
            id: "p1".into(),
            name: "Tokyo".into(),
            region: "jp".into(),
            protocol: Protocol::VLESS,
            origin: "local".into(),
            group_id: "default".into(),
            source_id: None,
            key: "vless://uuid@1.2.3.4:443?security=tls#Tokyo".into(),
        },
    )
    .unwrap();
    db
}

fn paths(dir: &std::path::Path) -> RuntimePaths {
    RuntimePaths {
        sing_box_binary: dir.join("sing-box"),
        config_path: dir.join("config.json"),
        clash_api_listen: "127.0.0.1:9090".into(),
        mixed_listen_port: 2080,
        test_config_path: dir.join("test-config.json"),
        test_clash_api_listen: "127.0.0.1:9091".into(),
        test_mixed_listen_port: 2081,
    }
}

fn stored(db: &Db) -> Settings {
    settings::get(db).unwrap()
}

#[test]
fn starting_writes_the_config_the_core_is_then_pointed_at() {
    let dir = std::env::temp_dir().join(format!("kagerou-core-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = db_with_a_profile();
    let (mut sup, _control) = supervisor();

    start(
        &db,
        &mut sup,
        &CoreSpec::connection(&paths(&dir), &stored(&db)),
    )
    .unwrap();

    assert!(matches!(sup.status(), Status::Running));
    let written = std::fs::read_to_string(dir.join("config.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(
        config["experimental"]["clash_api"]["external_controller"],
        "127.0.0.1:9090"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_test_core_never_gets_tun_or_the_system_proxy_however_the_user_set_them() {
    let db = db_with_a_profile();
    settings::update(
        &db,
        &SettingsPatch {
            tun_mode: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    let stored = stored(&db);
    let paths = paths(std::path::Path::new("/tmp"));

    let connection = CoreSpec::connection(&paths, &stored);
    assert!(connection.tun, "the user asked for TUN on their connection");

    let test = CoreSpec::test(&paths, &stored);
    assert!(
        !test.tun,
        "a delay test must not create a TUN device: it asks for a password and rewrites the machine's routing"
    );
    assert!(!test.system_proxy, "the OS proxy belongs to the connection");
}

#[test]
fn the_two_cores_never_share_a_config_file_or_a_port() {
    let db = db_with_a_profile();
    let stored = stored(&db);
    let paths = paths(std::path::Path::new("/tmp"));
    let connection = CoreSpec::connection(&paths, &stored);
    let test = CoreSpec::test(&paths, &stored);

    assert_ne!(connection.config_path, test.config_path);
    assert_ne!(connection.mixed_listen_port, test.mixed_listen_port);
    assert_ne!(connection.clash_api_listen, test.clash_api_listen);
}

#[test]
fn a_corrupt_profile_key_is_reported_before_anything_is_started() {
    let db = Db::open_in_memory().unwrap();
    profiles::insert(
        &db,
        &NewProfile {
            id: "p1".into(),
            name: "Broken".into(),
            region: "".into(),
            protocol: Protocol::VLESS,
            origin: "local".into(),
            group_id: "default".into(),
            source_id: None,
            key: "not-a-uri".into(),
        },
    )
    .unwrap();
    let (mut sup, control) = supervisor();

    let error = start(
        &db,
        &mut sup,
        &CoreSpec::connection(&paths(std::path::Path::new("/tmp")), &stored(&db)),
    )
    .unwrap_err();

    assert!(matches!(error, CoreError::Config(_)));
    assert!(matches!(sup.status(), Status::Stopped));
    assert_eq!(control.kill_count(), 0, "nothing was ever launched");
}
