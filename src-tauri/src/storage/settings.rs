use rusqlite::params;

use super::models::Settings;
use super::{Db, StorageError};

pub fn get(db: &Db) -> Result<Settings, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT theme, language, startup, tun_mode, system_proxy, auto_connect, geo_lookup, tun_interface, auto_update_subscriptions, subscription_update_interval, custom_subscription_update_minutes, group_sort, log_level, test_url
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(Settings {
                theme: row.get("theme")?,
                language: row.get("language")?,
                startup: row.get::<_, i64>("startup")? != 0,
                tun_mode: row.get::<_, i64>("tun_mode")? != 0,
                system_proxy: row.get::<_, i64>("system_proxy")? != 0,
                auto_connect: row.get::<_, i64>("auto_connect")? != 0,
                geo_lookup: row.get::<_, i64>("geo_lookup")? != 0,
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
    pub auto_connect: Option<bool>,
    pub geo_lookup: Option<bool>,
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
    // The connection modes are exclusive, so turning one on turns the other
    // off. TUN already captures everything the proxy would, and its core runs
    // elevated: on Linux that is pkexec's root, whose proxy is not the user's.
    if patch.tun_mode == Some(true) && patch.system_proxy == Some(true) {
        return Err(StorageError::InvalidInput(
            "TUN mode and the system proxy cannot both be on".into(),
        ));
    }
    let tun_mode = patch
        .tun_mode
        .or((patch.system_proxy == Some(true)).then_some(false));
    let system_proxy = patch
        .system_proxy
        .or((patch.tun_mode == Some(true)).then_some(false));
    let conn = db.lock();
    conn.execute(
        "UPDATE settings SET
            theme = COALESCE(?1, theme),
            language = COALESCE(?2, language),
            startup = COALESCE(?3, startup),
            tun_mode = COALESCE(?4, tun_mode),
            system_proxy = COALESCE(?5, system_proxy),
            auto_connect = COALESCE(?6, auto_connect),
            geo_lookup = COALESCE(?7, geo_lookup),
            tun_interface = COALESCE(?8, tun_interface),
            auto_update_subscriptions = COALESCE(?9, auto_update_subscriptions),
            subscription_update_interval = COALESCE(?10, subscription_update_interval),
            custom_subscription_update_minutes = COALESCE(?11, custom_subscription_update_minutes),
            group_sort = COALESCE(?12, group_sort),
            log_level = COALESCE(?13, log_level),
            test_url = COALESCE(?14, test_url)
         WHERE id = 1",
        params![
            patch.theme,
            patch.language,
            patch.startup.map(|v| v as i64),
            tun_mode.map(|v| v as i64),
            system_proxy.map(|v| v as i64),
            patch.auto_connect.map(|v| v as i64),
            patch.geo_lookup.map(|v| v as i64),
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
mod tests;
