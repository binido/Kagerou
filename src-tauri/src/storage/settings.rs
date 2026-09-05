use rusqlite::params;

use super::models::Settings;
use super::{Db, StorageError};

pub fn get(db: &Db) -> Result<Settings, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT theme, language, startup, tun_mode, system_proxy, tun_interface, auto_update_subscriptions, subscription_update_interval, custom_subscription_update_minutes, group_sort, log_level, test_url
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(Settings {
                theme: row.get("theme")?,
                language: row.get("language")?,
                startup: row.get::<_, i64>("startup")? != 0,
                tun_mode: row.get::<_, i64>("tun_mode")? != 0,
                system_proxy: row.get::<_, i64>("system_proxy")? != 0,
                tun_interface: row.get("tun_interface")?,
                auto_update_subscriptions: row.get::<_, i64>("auto_update_subscriptions")? != 0,
                subscription_update_interval: row.get("subscription_update_interval")?,
                custom_subscription_update_minutes: row.get("custom_subscription_update_minutes")?,
                group_sort: row.get("group_sort")?,
                log_level: row.get("log_level")?,
                test_url: row.get("test_url")?,
            })
        },
    )
    .map_err(StorageError::from)
}

#[derive(Default)]
pub struct SettingsPatch<'a> {
    pub theme: Option<&'a str>,
    pub language: Option<&'a str>,
    pub startup: Option<bool>,
    pub tun_mode: Option<bool>,
    pub system_proxy: Option<bool>,
    pub tun_interface: Option<&'a str>,
    pub auto_update_subscriptions: Option<bool>,
    pub subscription_update_interval: Option<&'a str>,
    pub custom_subscription_update_minutes: Option<i64>,
    pub group_sort: Option<&'a str>,
    pub log_level: Option<&'a str>,
    pub test_url: Option<&'a str>,
}

pub fn update(db: &Db, patch: &SettingsPatch) -> Result<(), StorageError> {
    if let Some(url) = patch.test_url {
        if url.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "test url cannot be empty".into(),
            ));
        }
    }
    let conn = db.lock();
    conn.execute(
        "UPDATE settings SET
            theme = COALESCE(?1, theme),
            language = COALESCE(?2, language),
            startup = COALESCE(?3, startup),
            tun_mode = COALESCE(?4, tun_mode),
            system_proxy = COALESCE(?5, system_proxy),
            tun_interface = COALESCE(?6, tun_interface),
            auto_update_subscriptions = COALESCE(?7, auto_update_subscriptions),
            subscription_update_interval = COALESCE(?8, subscription_update_interval),
            custom_subscription_update_minutes = COALESCE(?9, custom_subscription_update_minutes),
            group_sort = COALESCE(?10, group_sort),
            log_level = COALESCE(?11, log_level),
            test_url = COALESCE(?12, test_url)
         WHERE id = 1",
        params![
            patch.theme,
            patch.language,
            patch.startup.map(|v| v as i64),
            patch.tun_mode.map(|v| v as i64),
            patch.system_proxy.map(|v| v as i64),
            patch.tun_interface,
            patch.auto_update_subscriptions.map(|v| v as i64),
            patch.subscription_update_interval,
            patch.custom_subscription_update_minutes,
            patch.group_sort,
            patch.log_level,
            patch.test_url.map(str::trim),
        ],
    )?;
    Ok(())
}

pub fn get_active_profile_id(db: &Db) -> Result<Option<String>, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT active_profile_id FROM app_state WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .map_err(StorageError::from)
}

/// Sets the persisted "last active profile" pointer. The foreign key to
/// `profiles` means `id` must name an existing profile (or be `None`);
/// the pointer is cleared automatically by `ON DELETE SET NULL` if that
/// profile is later removed.
pub fn set_active_profile_id(db: &Db, id: Option<&str>) -> Result<(), StorageError> {
    let conn = db.lock();
    conn.execute(
        "UPDATE app_state SET active_profile_id = ?1 WHERE id = 1",
        params![id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
        assert!(!settings.system_proxy, "the two modes patch independently");

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
}
