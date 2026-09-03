use rusqlite::{params, OptionalExtension};

use super::models::{NewProfile, Profile, Protocol, TestResult, Tone};
use super::{Db, StorageError};

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<Profile> {
    let protocol: String = row.get("protocol")?;
    let tcp_tone: String = row.get("tcp_tone")?;
    let url_tone: String = row.get("url_tone")?;
    Ok(Profile {
        id: row.get("id")?,
        name: row.get("name")?,
        region: row.get("region")?,
        protocol: protocol.parse().unwrap_or(Protocol::VLESS),
        origin: row.get("origin")?,
        group_id: row.get("group_id")?,
        source_id: row.get("source_id")?,
        selected: row.get::<_, i64>("selected")? != 0,
        tcp: TestResult {
            value: row.get("tcp_value")?,
            tone: tcp_tone.parse().unwrap_or(Tone::Muted),
        },
        url: TestResult {
            value: row.get("url_value")?,
            tone: url_tone.parse().unwrap_or(Tone::Muted),
        },
        key: row.get("key")?,
    })
}

pub fn list_all(db: &Db) -> Result<Vec<Profile>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, region, protocol, origin, group_id, source_id, selected, tcp_value, tcp_tone, url_value, url_tone, key
         FROM profiles ORDER BY group_id, position",
    )?;
    let rows = stmt.query_map([], row_to_profile)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn get(db: &Db, id: &str) -> Result<Profile, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT id, name, region, protocol, origin, group_id, source_id, selected, tcp_value, tcp_tone, url_value, url_tone, key
         FROM profiles WHERE id = ?1",
        params![id],
        row_to_profile,
    )
    .optional()?
    .ok_or(StorageError::NotFound)
}

/// Inserts a new profile, appended to the end of its group's ordering.
pub fn insert(db: &Db, profile: &NewProfile) -> Result<(), StorageError> {
    let conn = db.lock();
    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM profiles WHERE group_id = ?1",
        params![profile.group_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, tcp_value, tcp_tone, url_value, url_tone, key, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'Not tested', 'muted', 'Not tested', 'muted', ?8, ?9)",
        params![
            profile.id,
            profile.name,
            profile.region,
            profile.protocol.as_str(),
            profile.origin,
            profile.group_id,
            profile.source_id,
            profile.key,
            next_position,
        ],
    )?;
    Ok(())
}

/// Atomically makes `id` the only selected profile. Errors if `id` does not
/// exist, leaving the previous selection untouched. Safe to call
/// concurrently from multiple threads: the whole read-modify-write runs
/// inside a single transaction serialized by `Db`'s connection mutex, so
/// competing selections never race each other into an inconsistent state.
pub fn select_profile(db: &Db, id: &str) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let exists: bool = tx
        .query_row("SELECT 1 FROM profiles WHERE id = ?1", params![id], |_| {
            Ok(())
        })
        .optional()?
        .is_some();
    if !exists {
        return Err(StorageError::NotFound);
    }

    tx.execute("UPDATE profiles SET selected = 0 WHERE selected = 1", [])?;
    tx.execute(
        "UPDATE profiles SET selected = 1 WHERE id = ?1",
        params![id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn rename(db: &Db, id: &str, name: &str) -> Result<(), StorageError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidInput(
            "profile name cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE profiles SET name = ?1 WHERE id = ?2",
        params![trimmed, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn delete(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM profiles WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub enum TestMethod {
    Tcp,
    Url,
}

pub fn set_test_result(
    db: &Db,
    id: &str,
    method: TestMethod,
    result: &TestResult,
) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = match method {
        TestMethod::Tcp => conn.execute(
            "UPDATE profiles SET tcp_value = ?1, tcp_tone = ?2 WHERE id = ?3",
            params![result.value, result.tone.as_str(), id],
        )?,
        TestMethod::Url => conn.execute(
            "UPDATE profiles SET url_value = ?1, url_tone = ?2 WHERE id = ?3",
            params![result.value, result.tone.as_str(), id],
        )?,
    };
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

/// Moves a profile into `target_group_id`, appending it at the end of that
/// group's ordering.
pub fn move_to_group(db: &Db, id: &str, target_group_id: &str) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let group_exists: bool = tx
        .query_row(
            "SELECT 1 FROM profile_groups WHERE id = ?1",
            params![target_group_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !group_exists {
        return Err(StorageError::InvalidInput(format!(
            "group {target_group_id} does not exist"
        )));
    }

    let next_position: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM profiles WHERE group_id = ?1",
        params![target_group_id],
        |row| row.get(0),
    )?;
    let affected = tx.execute(
        "UPDATE profiles SET group_id = ?1, position = ?2 WHERE id = ?3",
        params![target_group_id, next_position, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    tx.commit()?;
    Ok(())
}

/// Rewrites the `position` of every profile in `group_id` to match the
/// order of `ordered_ids`. Used to implement both "move up/down" (compute
/// the swapped order, then reorder) and drag-to-reorder in one primitive
/// rather than two position-swap-specific queries. Rejects the whole
/// operation — no partial reorder — if `ordered_ids` doesn't contain
/// exactly the profiles currently in that group.
pub fn reorder(db: &Db, group_id: &str, ordered_ids: &[String]) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let mut current: Vec<String> = {
        let mut stmt =
            tx.prepare("SELECT id FROM profiles WHERE group_id = ?1 ORDER BY position")?;
        let rows = stmt.query_map(params![group_id], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    current.sort();
    let mut requested = ordered_ids.to_vec();
    requested.sort();
    if current != requested {
        return Err(StorageError::InvalidInput(
            "ordered_ids must be exactly the profiles currently in the group".into(),
        ));
    }

    for (position, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE profiles SET position = ?1 WHERE id = ?2",
            params![position as i64, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::groups;
    use crate::storage::models::NewProfileGroup;

    /// The base migration already seeds a `default` group; this only adds
    /// the extra `custom` one these tests need.
    fn seeded_db() -> Db {
        let db = Db::open_in_memory().unwrap();
        groups::insert(
            &db,
            &NewProfileGroup {
                id: "custom".into(),
                label: "Custom".into(),
                kind: "custom".into(),
                source_id: None,
            },
        )
        .unwrap();
        db
    }

    fn new_profile(id: &str, group_id: &str) -> NewProfile {
        NewProfile {
            id: id.into(),
            name: format!("Profile {id}"),
            region: "local".into(),
            protocol: Protocol::VLESS,
            origin: "local".into(),
            group_id: group_id.into(),
            source_id: None,
            key: format!("vless://{id}"),
        }
    }

    #[test]
    fn insert_then_get_round_trips_all_fields() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        let profile = get(&db, "p1").unwrap();
        assert_eq!(profile.name, "Profile p1");
        assert_eq!(profile.group_id, "default");
        assert_eq!(profile.protocol, Protocol::VLESS);
        assert!(!profile.selected);
        assert_eq!(profile.tcp.value, "Not tested");
    }

    #[test]
    fn get_unknown_profile_returns_not_found() {
        let db = seeded_db();
        assert!(matches!(get(&db, "missing"), Err(StorageError::NotFound)));
    }

    #[test]
    fn insert_rejects_duplicate_ids() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        let err = insert(&db, &new_profile("p1", "default")).unwrap_err();
        assert!(matches!(err, StorageError::Sqlite(_)));
    }

    #[test]
    fn insert_rejects_profile_referencing_an_unknown_group() {
        let db = seeded_db();
        let err = insert(&db, &new_profile("p1", "no-such-group")).unwrap_err();
        assert!(matches!(err, StorageError::Sqlite(_)));
    }

    #[test]
    fn inserted_profiles_are_appended_in_order_within_a_group() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        insert(&db, &new_profile("p2", "default")).unwrap();
        insert(&db, &new_profile("p3", "default")).unwrap();
        let ids: Vec<_> = list_all(&db).unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["p1", "p2", "p3"]);
    }

    #[test]
    fn select_profile_makes_it_the_only_selected_one() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        insert(&db, &new_profile("p2", "default")).unwrap();
        select_profile(&db, "p1").unwrap();
        select_profile(&db, "p2").unwrap();

        let profiles = list_all(&db).unwrap();
        let selected: Vec<_> = profiles
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.id.clone())
            .collect();
        assert_eq!(selected, vec!["p2"]);
    }

    #[test]
    fn select_profile_on_unknown_id_leaves_selection_unchanged() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        select_profile(&db, "p1").unwrap();

        let err = select_profile(&db, "does-not-exist").unwrap_err();
        assert!(matches!(err, StorageError::NotFound));

        let profile = get(&db, "p1").unwrap();
        assert!(
            profile.selected,
            "original selection must survive a failed re-selection"
        );
    }

    #[test]
    fn rename_rejects_blank_and_whitespace_only_names() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        assert!(matches!(
            rename(&db, "p1", "   ").unwrap_err(),
            StorageError::InvalidInput(_)
        ));
        assert_eq!(get(&db, "p1").unwrap().name, "Profile p1");
    }

    #[test]
    fn rename_unknown_profile_is_not_found() {
        let db = seeded_db();
        assert!(matches!(
            rename(&db, "missing", "New name").unwrap_err(),
            StorageError::NotFound
        ));
    }

    #[test]
    fn deleting_a_group_cascades_to_its_profiles() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "custom")).unwrap();
        groups::delete(&db, "custom").unwrap();
        assert!(matches!(get(&db, "p1"), Err(StorageError::NotFound)));
    }

    #[test]
    fn move_to_group_rejects_an_unknown_target_group() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        let err = move_to_group(&db, "p1", "ghost-group").unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));
        assert_eq!(get(&db, "p1").unwrap().group_id, "default");
    }

    #[test]
    fn move_to_group_appends_at_the_end_of_the_target_group() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "custom")).unwrap();
        insert(&db, &new_profile("p2", "default")).unwrap();
        move_to_group(&db, "p2", "custom").unwrap();
        let ids: Vec<_> = list_all(&db)
            .unwrap()
            .into_iter()
            .filter(|p| p.group_id == "custom")
            .map(|p| p.id)
            .collect();
        assert_eq!(ids, vec!["p1", "p2"]);
    }

    #[test]
    fn set_test_result_updates_only_the_targeted_method() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        set_test_result(
            &db,
            "p1",
            TestMethod::Tcp,
            &TestResult {
                value: "42 ms".into(),
                tone: Tone::Good,
            },
        )
        .unwrap();
        let profile = get(&db, "p1").unwrap();
        assert_eq!(profile.tcp.value, "42 ms");
        assert_eq!(profile.tcp.tone, Tone::Good);
        assert_eq!(profile.url.value, "Not tested");
    }

    #[test]
    fn concurrent_inserts_with_the_same_id_leave_exactly_one_row() {
        use std::sync::Arc;
        use std::thread;

        let db = Arc::new(seeded_db());
        let mut handles = Vec::new();
        for _ in 0..8 {
            let db = Arc::clone(&db);
            handles.push(thread::spawn(move || {
                insert(&db, &new_profile("race", "default"))
            }));
        }
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let ok_count = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(
            ok_count, 1,
            "exactly one concurrent insert of the same id should succeed"
        );

        let count: i64 = db
            .lock()
            .query_row(
                "SELECT COUNT(*) FROM profiles WHERE id = 'race'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn reorder_applies_the_given_order() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        insert(&db, &new_profile("p2", "default")).unwrap();
        insert(&db, &new_profile("p3", "default")).unwrap();

        reorder(&db, "default", &["p3".into(), "p1".into(), "p2".into()]).unwrap();

        let ids: Vec<_> = list_all(&db).unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["p3", "p1", "p2"]);
    }

    #[test]
    fn reorder_rejects_a_set_that_omits_a_profile_in_the_group() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        insert(&db, &new_profile("p2", "default")).unwrap();

        let err = reorder(&db, "default", &["p1".into()]).unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));

        // Nothing should have changed.
        let ids: Vec<_> = list_all(&db).unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["p1", "p2"]);
    }

    #[test]
    fn reorder_rejects_an_id_from_a_different_group() {
        let db = seeded_db();
        insert(&db, &new_profile("p1", "default")).unwrap();
        insert(&db, &new_profile("p2", "custom")).unwrap();

        let err = reorder(&db, "default", &["p1".into(), "p2".into()]).unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));
    }
}
