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
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;
    use std::io::Write;

    #[test]
    fn opens_and_migrates_a_fresh_in_memory_db() {
        let db = Db::open_in_memory().expect("fresh db should open");
        let conn = db.lock();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as i64);
    }

    #[test]
    fn opens_a_db_file_on_disk_and_reopens_it_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kagerou.sqlite3");

        {
            let db = Db::open(&path).expect("first open should create + migrate");
            db.lock()
                .execute(
                    "INSERT INTO profile_groups (id, label, kind, source_id, is_open, position) VALUES ('custom-1', 'Custom Group', 'custom', NULL, 1, 1)",
                    [],
                )
                .unwrap();
        }

        let db = Db::open(&path).expect("second open should not re-run migrations");
        let label: String = db
            .lock()
            .query_row(
                "SELECT label FROM profile_groups WHERE id = 'custom-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(label, "Custom Group");
    }

    #[test]
    fn rejects_a_corrupted_database_file_instead_of_panicking() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.sqlite3");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"this is not a sqlite database, just garbage bytes\0\x01\x02")
            .unwrap();
        drop(file);

        let result = Db::open(&path);
        assert!(matches!(result, Err(StorageError::Sqlite(_))));
    }

    #[test]
    fn rejects_a_database_from_a_newer_unknown_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("future.sqlite3");

        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", (MIGRATIONS.len() as i64) + 5)
                .unwrap();
        }

        let result = Db::open(&path);
        assert!(matches!(
            result,
            Err(StorageError::UnsupportedSchemaVersion { .. })
        ));
    }

    #[test]
    fn a_migration_failure_does_not_advance_the_schema_version() {
        // Simulate a broken migration set: valid step 1, then a step that
        // fails. The failure must be surfaced, and user_version must stay
        // at 1 rather than silently advancing past the broken step.
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        let broken_migrations: &[&str] = &[MIGRATIONS[0], "THIS IS NOT VALID SQL AND MUST FAIL;"];

        let mut result = Ok(());
        for (index, migration) in broken_migrations.iter().enumerate() {
            match conn.execute_batch(migration) {
                Ok(()) => {
                    conn.pragma_update(None, "user_version", (index as i64) + 1)
                        .unwrap();
                }
                Err(source) => {
                    result = Err(StorageError::Migration {
                        step: index + 1,
                        source,
                    });
                    break;
                }
            }
        }

        assert!(result.is_err());
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1, "version must not advance past the failed step");
    }

    #[test]
    fn concurrent_profile_selection_never_leaves_more_than_one_profile_selected() {
        use std::sync::Arc;
        use std::thread;

        let db = Arc::new(Db::open_in_memory().unwrap());
        {
            let conn = db.lock();
            conn.execute_batch(
                "INSERT INTO profile_groups (id, label, kind, source_id, is_open, position) VALUES ('g', 'Default', 'default', NULL, 1, 0);
                 INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, tcp_value, tcp_tone, url_value, url_tone, key, position) VALUES
                   ('p1','P1','r','VLESS','local','g',NULL,1,'','muted','','muted','k1',0),
                   ('p2','P2','r','VLESS','local','g',NULL,0,'','muted','','muted','k2',1),
                   ('p3','P3','r','VLESS','local','g',NULL,0,'','muted','','muted','k3',2);",
            )
            .unwrap();
        }

        let mut handles = Vec::new();
        for id in ["p1", "p2", "p3", "p1", "p2"] {
            let db = Arc::clone(&db);
            handles.push(thread::spawn(move || {
                crate::storage::profiles::select_profile(&db, id).unwrap();
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }

        let conn = db.lock();
        let selected_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM profiles WHERE selected = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            selected_count, 1,
            "exactly one profile must end up selected, never zero or many"
        );

        let orphan_selected: Option<String> = conn
            .query_row(
                "SELECT id FROM profiles WHERE selected = 1 AND id NOT IN ('p1','p2','p3')",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        assert!(orphan_selected.is_none());
    }
}
