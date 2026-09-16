mod error;
pub mod groups;
pub mod models;
pub mod profiles;
pub mod routing;
pub mod settings;
pub mod sources;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

pub use error::StorageError;

/// Schema migrations, applied in order starting from `PRAGMA user_version`.
/// Each entry is run inside its own transaction; on failure the transaction
/// is rolled back and `user_version` is left at the last successful step.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_init.sql"),
    include_str!("migrations/0002_connection_modes.sql"),
    include_str!("migrations/0003_settings_log_level.sql"),
    include_str!("migrations/0004_settings_test_url.sql"),
    include_str!("migrations/0005_startup_defaults_off.sql"),
    include_str!("migrations/0006_drop_tcp_test.sql"),
    include_str!("migrations/0007_profile_last_selected.sql"),
    include_str!("migrations/0008_auto_connect.sql"),
    include_str!("migrations/0009_geo_lookup.sql"),
    include_str!("migrations/0010_unify_sources.sql"),
    include_str!("migrations/0011_subscription_refresh_time.sql"),
    include_str!("migrations/0012_test_outcome.sql"),
];

/// A handle to the application's SQLite database.
///
/// Wrapped in a single `Mutex<Connection>` rather than a connection pool:
/// SQLite serializes writers anyway, and this keeps every mutation
/// (profile selection, group edits, ...) trivially atomic and race-free
/// across threads without pulling in r2d2 for a single-file embedded DB.
pub struct Db(Mutex<Connection>);

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, StorageError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self, StorageError> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        Ok(Self(Mutex::new(conn)))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn migrate(conn: &Connection) -> Result<(), StorageError> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current < 0 || current as usize > MIGRATIONS.len() {
        return Err(StorageError::UnsupportedSchemaVersion {
            found: current,
            supported: MIGRATIONS.len() as i64,
        });
    }

    for (index, migration) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        conn.execute_batch(migration)
            .map_err(|source| StorageError::Migration {
                step: index + 1,
                source,
            })?;
        conn.pragma_update(None, "user_version", (index as i64) + 1)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
