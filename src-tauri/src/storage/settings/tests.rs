use super::*;

#[test]
fn get_returns_the_seeded_defaults() {
    let db = Db::open_in_memory().unwrap();
    let settings = get(&db).unwrap();
    assert_eq!(settings.theme, "catppuccin-mocha");
    assert_eq!(settings.language, "en");
    assert!(
        !settings.startup,
        "a fresh install must not add itself to login items unasked"
    );
    assert_eq!(settings.subscription_update_interval, "30");
    assert!(
        !settings.tun_mode && !settings.system_proxy,
        "connection modes default to off on a fresh install"
    );
    assert!(
        !settings.auto_connect,
        "auto-connect defaults to off on a fresh install"
    );
    assert_eq!(settings.log_level, "info");
    assert_eq!(settings.test_url, "http://www.gstatic.com/generate_204");
}

#[test]
fn update_rejects_an_unknown_log_level() {
    let db = Db::open_in_memory().unwrap();
    let err = update(
        &db,
        &SettingsPatch {
            log_level: Some("verbose"),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::Sqlite(_)));
    assert_eq!(
        get(&db).unwrap().log_level,
        "info",
        "the rejected update must not partially apply"
    );
}

#[test]
fn log_level_round_trip() {
    let db = Db::open_in_memory().unwrap();
    update(
        &db,
        &SettingsPatch {
            log_level: Some("trace"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(get(&db).unwrap().log_level, "trace");

    update(
        &db,
        &SettingsPatch {
            log_level: Some("error"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(get(&db).unwrap().log_level, "error");
}

#[test]
fn test_url_round_trip() {
    let db = Db::open_in_memory().unwrap();
    update(
        &db,
        &SettingsPatch {
            test_url: Some("http://example.com/health"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(get(&db).unwrap().test_url, "http://example.com/health");

    // A blank value is rejected and the stored URL survives.
    let err = update(
        &db,
        &SettingsPatch {
            test_url: Some("   "),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
    assert_eq!(
        get(&db).unwrap().test_url,
        "http://example.com/health",
        "the rejected update must not partially apply"
    );
}

#[test]
fn connection_modes_round_trip() {
    let db = Db::open_in_memory().unwrap();
    update(
        &db,
        &SettingsPatch {
            tun_mode: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    let settings = get(&db).unwrap();
    assert!(settings.tun_mode);
    assert!(!settings.system_proxy);

    update(
        &db,
        &SettingsPatch {
            tun_mode: Some(false),
            system_proxy: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    let settings = get(&db).unwrap();
    assert!(!settings.tun_mode);
    assert!(settings.system_proxy);
}

#[test]
fn turning_one_connection_mode_on_turns_the_other_off() {
    let db = Db::open_in_memory().unwrap();
    let set = |patch: SettingsPatch| update(&db, &patch).unwrap();

    set(SettingsPatch {
        system_proxy: Some(true),
        ..Default::default()
    });
    set(SettingsPatch {
        tun_mode: Some(true),
        ..Default::default()
    });
    let settings = get(&db).unwrap();
    assert!(settings.tun_mode && !settings.system_proxy);

    set(SettingsPatch {
        system_proxy: Some(true),
        ..Default::default()
    });
    let settings = get(&db).unwrap();
    assert!(!settings.tun_mode && settings.system_proxy);

    // Turning one off leaves the other alone.
    set(SettingsPatch {
        tun_mode: Some(false),
        ..Default::default()
    });
    assert!(get(&db).unwrap().system_proxy);
}

#[test]
fn both_connection_modes_at_once_are_rejected_without_a_partial_write() {
    let db = Db::open_in_memory().unwrap();
    let err = update(
        &db,
        &SettingsPatch {
            tun_mode: Some(true),
            system_proxy: Some(true),
            language: Some("ru"),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
    let settings = get(&db).unwrap();
    assert!(!settings.tun_mode && !settings.system_proxy);
    assert_eq!(settings.language, "en");
}

#[test]
fn auto_connect_round_trip() {
    let db = Db::open_in_memory().unwrap();
    assert!(
        !get(&db).unwrap().auto_connect,
        "a fresh install must not connect on its own"
    );

    update(
        &db,
        &SettingsPatch {
            auto_connect: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(get(&db).unwrap().auto_connect);

    update(
        &db,
        &SettingsPatch {
            auto_connect: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!get(&db).unwrap().auto_connect);
}

#[test]
fn geo_lookup_round_trip() {
    let db = Db::open_in_memory().unwrap();
    assert!(
        get(&db).unwrap().geo_lookup,
        "the location on the dashboard is the point, so the lookup starts on"
    );

    update(
        &db,
        &SettingsPatch {
            geo_lookup: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        !get(&db).unwrap().geo_lookup,
        "turning it off must actually stop the request to the third party"
    );

    update(
        &db,
        &SettingsPatch {
            geo_lookup: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(get(&db).unwrap().geo_lookup);
}

#[test]
fn update_only_touches_provided_fields() {
    let db = Db::open_in_memory().unwrap();
    update(
        &db,
        &SettingsPatch {
            theme: Some("kanagawa-wave"),
            ..Default::default()
        },
    )
    .unwrap();
    let settings = get(&db).unwrap();
    assert_eq!(settings.theme, "kanagawa-wave");
    assert_eq!(
        settings.language, "en",
        "untouched fields must survive a partial patch"
    );
}

#[test]
fn update_rejects_an_invalid_language_via_the_check_constraint() {
    let db = Db::open_in_memory().unwrap();
    let err = update(
        &db,
        &SettingsPatch {
            language: Some("fr"),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::Sqlite(_)));
    assert_eq!(
        get(&db).unwrap().language,
        "en",
        "the rejected update must not partially apply"
    );
}

#[test]
fn active_profile_id_round_trips_and_defaults_to_none() {
    let db = Db::open_in_memory().unwrap();
    crate::storage::groups::insert(
        &db,
        &crate::storage::models::NewProfileGroup {
            id: "g".into(),
            label: "G".into(),
            kind: "default".into(),
            source_id: None,
        },
    )
    .unwrap();
    crate::storage::profiles::insert(
        &db,
        &crate::storage::models::NewProfile {
            id: "p1".into(),
            name: "P1".into(),
            region: "r".into(),
            protocol: crate::storage::models::Protocol::VLESS,
            origin: "local".into(),
            group_id: "g".into(),
            source_id: None,
            key: "vless://p1".into(),
        },
    )
    .unwrap();

    assert_eq!(get_active_profile_id(&db).unwrap(), None);
    set_active_profile_id(&db, Some("p1")).unwrap();
    assert_eq!(get_active_profile_id(&db).unwrap(), Some("p1".to_string()));
    set_active_profile_id(&db, None).unwrap();
    assert_eq!(get_active_profile_id(&db).unwrap(), None);
}

#[test]
fn set_active_profile_id_rejects_an_unknown_profile() {
    let db = Db::open_in_memory().unwrap();
    let err = set_active_profile_id(&db, Some("ghost")).unwrap_err();
    assert!(matches!(err, StorageError::Sqlite(_)));
    assert_eq!(get_active_profile_id(&db).unwrap(), None);
}

#[test]
fn active_profile_id_is_cleared_when_the_profile_is_deleted() {
    let db = Db::open_in_memory().unwrap();
    crate::storage::groups::insert(
        &db,
        &crate::storage::models::NewProfileGroup {
            id: "g".into(),
            label: "G".into(),
            kind: "default".into(),
            source_id: None,
        },
    )
    .unwrap();
    crate::storage::profiles::insert(
        &db,
        &crate::storage::models::NewProfile {
            id: "p1".into(),
            name: "P1".into(),
            region: "r".into(),
            protocol: crate::storage::models::Protocol::VLESS,
            origin: "local".into(),
            group_id: "g".into(),
            source_id: None,
            key: "vless://p1".into(),
        },
    )
    .unwrap();
    set_active_profile_id(&db, Some("p1")).unwrap();

    crate::storage::profiles::delete(&db, "p1").unwrap();

    assert_eq!(get_active_profile_id(&db).unwrap(), None);
}
