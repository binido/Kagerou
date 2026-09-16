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
fn migration_11_clears_the_prose_that_used_to_stand_for_a_refresh_time() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    for migration in &MIGRATIONS[..10] {
        conn.execute_batch(migration).unwrap();
    }
    conn.execute_batch(
            "INSERT INTO sources (id, name, type, value, status, last_refresh, origin_label) VALUES
               ('a', 'A', 'url', 'https://a.example', 'up-to-date', 'Updated just now', 'Remote URL'),
               ('b', 'B', 'url', 'https://b.example', 'refresh-due', 'Updated 3 days ago', 'Remote URL');",
        )
        .unwrap();

    conn.execute_batch(MIGRATIONS[10]).unwrap();

    // Prose cannot be turned back into a time, so both read as never
    // refreshed rather than keeping a sentence no one can translate.
    let mut stmt = conn
        .prepare("SELECT last_refresh FROM sources ORDER BY id")
        .unwrap();
    let stamps: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(stamps, vec![String::new(), String::new()]);
}

#[test]
fn migration_10_drops_key_sources_and_frees_orphaned_subscription_groups() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    for migration in &MIGRATIONS[..9] {
        conn.execute_batch(migration).unwrap();
    }
    conn.execute_batch(
            "INSERT INTO sources (id, name, type, value, status, last_refresh, origin_label) VALUES
               ('key', 'Key', 'key', 'vless://a', 'ready', 'Added just now', 'Local key'),
               ('url', 'Url', 'url', 'https://sub.example', 'up-to-date', 'Updated just now', 'Remote URL');
             INSERT INTO profile_groups (id, label, kind, source_id, is_open, position) VALUES
               ('live', 'Live', 'subscription', 'url', 1, 5),
               ('orphan', 'Orphan', 'subscription', NULL, 1, 6);
             INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, url_value, url_tone, key, position) VALUES
               ('from-key', 'K', '', 'VLESS', 'local', 'default', 'key', 0, '', 'muted', 'vless://a', 0),
               ('from-live', 'L', '', 'VLESS', 'imported', 'live', 'url', 0, '', 'muted', 'vless://b', 0),
               ('from-orphan', 'O', '', 'VLESS', 'imported', 'orphan', NULL, 0, '', 'muted', 'vless://c', 0);",
        )
        .unwrap();

    conn.execute_batch(MIGRATIONS[9]).unwrap();

    let source_ids: Vec<String> = conn
        .prepare("SELECT id FROM sources")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(source_ids, vec!["url"]);
    let profile = |id: &str| -> (String, Option<String>) {
        conn.query_row(
            "SELECT origin, source_id FROM profiles WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    };
    assert_eq!(
        profile("from-key"),
        ("local".into(), None),
        "the VPN outlives its key row"
    );
    assert_eq!(
        profile("from-live"),
        ("imported".into(), Some("url".into()))
    );
    assert_eq!(profile("from-orphan"), ("local".into(), None));
    let kind = |id: &str| -> String {
        conn.query_row(
            "SELECT kind FROM profile_groups WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap()
    };
    assert_eq!(kind("live"), "subscription");
    assert_eq!(kind("orphan"), "custom");
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
                 INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, url_kind, url_millis, key, position) VALUES
                   ('p1','P1','r','VLESS','local','g',NULL,1,'notTested',NULL,'k1',0),
                   ('p2','P2','r','VLESS','local','g',NULL,0,'notTested',NULL,'k2',1),
                   ('p3','P3','r','VLESS','local','g',NULL,0,'notTested',NULL,'k3',2);",
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

/// Sentences the interface used to show become a kind and a number, so
/// sorting has something to compare and translating has something to switch
/// on. Anything unrecognised is untested rather than lost.
#[test]
fn migration_12_turns_result_prose_into_a_kind_and_a_number() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    for migration in &MIGRATIONS[..11] {
        conn.execute_batch(migration).unwrap();
    }
    conn.execute_batch(
        "INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, url_value, url_tone, key, position) VALUES
           ('fast', 'F', '', 'VLESS', 'local', 'default', NULL, 0, '42 ms', 'good', 'vless://a', 0),
           ('slow', 'S', '', 'VLESS', 'local', 'default', NULL, 0, '1200 ms', 'bad', 'vless://b', 1),
           ('out', 'O', '', 'VLESS', 'local', 'default', NULL, 0, 'Timeout', 'bad', 'vless://c', 2),
           ('quiet', 'Q', '', 'VLESS', 'local', 'default', NULL, 0, 'No response', 'bad', 'vless://d', 3),
           ('gone', 'G', '', 'VLESS', 'local', 'default', NULL, 0, 'Unavailable', 'bad', 'vless://e', 4),
           ('new', 'N', '', 'VLESS', 'local', 'default', NULL, 0, 'Not tested', 'muted', 'vless://f', 5),
           ('odd', 'D', '', 'VLESS', 'local', 'default', NULL, 0, '200 OK', 'good', 'vless://g', 6);",
    )
    .unwrap();

    conn.execute_batch(MIGRATIONS[11]).unwrap();

    let mut stmt = conn
        .prepare("SELECT id, url_kind, url_millis FROM profiles ORDER BY position")
        .unwrap();
    let rows: Vec<(String, String, Option<i64>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();

    assert_eq!(
        rows,
        vec![
            ("fast".into(), "latency".into(), Some(42)),
            ("slow".into(), "latency".into(), Some(1200)),
            ("out".into(), "timeout".into(), None),
            ("quiet".into(), "noResponse".into(), None),
            ("gone".into(), "unavailable".into(), None),
            ("new".into(), "notTested".into(), None),
            ("odd".into(), "notTested".into(), None),
        ]
    );
}
