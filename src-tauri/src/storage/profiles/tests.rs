use super::*;
use crate::storage::groups;
use crate::storage::models::{NewProfileGroup, Tone};

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
    assert_eq!(profile.url.outcome, TestOutcome::NotTested);
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

fn add_subscription_group(db: &Db) {
    groups::insert(
        db,
        &NewProfileGroup {
            id: "sub".into(),
            label: "Sub".into(),
            kind: "subscription".into(),
            source_id: None,
        },
    )
    .unwrap();
}

#[test]
fn move_to_group_refuses_a_subscription_group_as_the_target() {
    let db = seeded_db();
    add_subscription_group(&db);
    insert(&db, &new_profile("p1", "default")).unwrap();

    let err = move_to_group(&db, "p1", "sub").unwrap_err();

    assert!(matches!(err, StorageError::InvalidInput(_)));
    assert_eq!(get(&db, "p1").unwrap().group_id, "default");
}

#[test]
fn move_to_group_refuses_to_take_a_profile_out_of_a_subscription() {
    let db = seeded_db();
    add_subscription_group(&db);
    insert(&db, &new_profile("p1", "sub")).unwrap();

    let err = move_to_group(&db, "p1", "custom").unwrap_err();

    assert!(matches!(err, StorageError::InvalidInput(_)));
    assert_eq!(get(&db, "p1").unwrap().group_id, "sub");
}

#[test]
fn move_to_group_on_an_unknown_profile_is_not_found() {
    let db = seeded_db();
    assert!(matches!(
        move_to_group(&db, "ghost", "custom").unwrap_err(),
        StorageError::NotFound
    ));
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
fn recently_selected_is_newest_first_and_skips_the_never_chosen() {
    let db = seeded_db();
    for id in ["p1", "p2", "p3"] {
        insert(&db, &new_profile(id, "default")).unwrap();
    }
    // unixepoch() has one-second resolution, so order is forced by hand
    // rather than by racing the clock.
    select_profile(&db, "p1").unwrap();
    stamp(&db, "p1", 100);
    select_profile(&db, "p2").unwrap();
    stamp(&db, "p2", 300);

    let recent = recently_selected(&db, 5).unwrap();

    let ids: Vec<_> = recent.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["p2", "p1"],
        "newest first, and p3 was never selected"
    );
}

#[test]
fn recently_selected_honours_its_limit() {
    let db = seeded_db();
    for (n, id) in ["p1", "p2", "p3"].iter().enumerate() {
        insert(&db, &new_profile(id, "default")).unwrap();
        select_profile(&db, id).unwrap();
        stamp(&db, id, 100 + n as i64);
    }
    assert_eq!(recently_selected(&db, 2).unwrap().len(), 2);
    assert!(recently_selected(&db, 0).unwrap().is_empty());
}

#[test]
fn selecting_again_moves_a_profile_back_to_the_front() {
    let db = seeded_db();
    for id in ["p1", "p2"] {
        insert(&db, &new_profile(id, "default")).unwrap();
        select_profile(&db, id).unwrap();
    }
    stamp(&db, "p1", 100);
    stamp(&db, "p2", 200);
    assert_eq!(recently_selected(&db, 2).unwrap()[0].id, "p2");

    select_profile(&db, "p1").unwrap();

    assert_eq!(
        recently_selected(&db, 2).unwrap()[0].id,
        "p1",
        "reselecting has to restamp, or the list freezes"
    );
}

fn stamp(db: &Db, id: &str, at: i64) {
    db.lock()
        .execute(
            "UPDATE profiles SET last_selected_at = ?1 WHERE id = ?2",
            params![at, id],
        )
        .unwrap();
}

#[test]
fn a_stored_outcome_comes_back_with_the_tone_it_implies() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    set_test_outcome(&db, "p1", TestOutcome::Latency { millis: 42 }).unwrap();
    let profile = get(&db, "p1").unwrap();
    assert_eq!(profile.url.outcome, TestOutcome::Latency { millis: 42 });
    assert_eq!(profile.url.tone, Tone::Good);
}

#[test]
fn a_slow_but_answering_server_is_stored_as_a_latency_not_as_a_failure() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    set_test_outcome(&db, "p1", TestOutcome::Latency { millis: 900 }).unwrap();
    let profile = get(&db, "p1").unwrap();
    assert_eq!(profile.url.outcome, TestOutcome::Latency { millis: 900 });
    assert_eq!(profile.url.tone, Tone::Bad, "it is still drawn as bad");
}

fn fail(db: &Db, id: &str) {
    set_test_outcome(db, id, TestOutcome::NoResponse).unwrap();
}

#[test]
fn clear_test_results_resets_only_the_target_group() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    insert(&db, &new_profile("p2", "default")).unwrap();
    insert(&db, &new_profile("p3", "custom")).unwrap();
    fail(&db, "p1");
    fail(&db, "p2");
    fail(&db, "p3");

    clear_test_results(&db, "default").unwrap();

    for id in ["p1", "p2"] {
        let profile = get(&db, id).unwrap();
        assert_eq!(profile.url.outcome, TestOutcome::NotTested);
        assert_eq!(profile.url.tone, Tone::Muted);
    }
    let untouched = get(&db, "p3").unwrap();
    assert_eq!(untouched.url.outcome, TestOutcome::NoResponse);
}

#[test]
fn clear_test_results_is_idempotent_on_untested_profiles() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    clear_test_results(&db, "default").unwrap();
    clear_test_results(&db, "no-such-group").unwrap();
    assert_eq!(get(&db, "p1").unwrap().url.outcome, TestOutcome::NotTested);
}

#[test]
fn delete_unavailable_deletes_only_failed_rows_of_the_target_group() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    insert(&db, &new_profile("p2", "default")).unwrap();
    insert(&db, &new_profile("p3", "default")).unwrap();
    insert(&db, &new_profile("p4", "custom")).unwrap();
    fail(&db, "p1");
    fail(&db, "p4");

    let deleted = delete_unavailable(&db, "default", None).unwrap();

    assert_eq!(deleted, 1);
    assert!(matches!(get(&db, "p1"), Err(StorageError::NotFound)));
    assert!(
        get(&db, "p3").is_ok(),
        "never-tested profile survives by construction"
    );
    assert!(get(&db, "p4").is_ok(), "other groups are untouched");
}

/// "Remove unavailable" used to filter on the display colour, which a
/// latency over 400ms also carries, so a working-but-slow server was deleted
/// along with the ones that never answered.
#[test]
fn delete_unavailable_keeps_a_server_that_answered_slowly() {
    let db = seeded_db();
    insert(&db, &new_profile("slow", "default")).unwrap();
    insert(&db, &new_profile("dead", "default")).unwrap();
    set_test_outcome(&db, "slow", TestOutcome::Latency { millis: 1200 }).unwrap();
    set_test_outcome(&db, "dead", TestOutcome::Timeout).unwrap();

    let deleted = delete_unavailable(&db, "default", None).unwrap();

    assert_eq!(deleted, 1);
    assert!(get(&db, "slow").is_ok(), "1200ms is slow, not unavailable");
    assert!(matches!(get(&db, "dead"), Err(StorageError::NotFound)));
}

#[test]
fn delete_unavailable_never_deletes_the_skipped_profile() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    insert(&db, &new_profile("p2", "default")).unwrap();
    fail(&db, "p1");
    fail(&db, "p2");

    let deleted = delete_unavailable(&db, "default", Some("p1")).unwrap();

    assert_eq!(deleted, 1);
    assert!(get(&db, "p1").is_ok());
    assert!(matches!(get(&db, "p2"), Err(StorageError::NotFound)));
}

#[test]
fn delete_unavailable_on_an_unknown_group_deletes_nothing() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    fail(&db, "p1");

    assert_eq!(delete_unavailable(&db, "ghost", None).unwrap(), 0);
    assert!(get(&db, "p1").is_ok());
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

fn ordered_ids(db: &Db, group_id: &str) -> Vec<String> {
    groups::get(db, group_id).unwrap().profile_ids
}

#[test]
fn moving_up_swaps_a_profile_with_the_one_above_it() {
    let db = seeded_db();
    for id in ["p1", "p2", "p3"] {
        insert(&db, &new_profile(id, "default")).unwrap();
    }

    move_within_group(&db, "p3", Direction::Up).unwrap();

    assert_eq!(ordered_ids(&db, "default"), ["p1", "p3", "p2"]);
}

/// Pressing "move up" on the first row is what any list does: nothing. It
/// used to be an error, which the interface then had to swallow.
#[test]
fn moving_past_either_end_changes_nothing_and_is_not_an_error() {
    let db = seeded_db();
    for id in ["p1", "p2"] {
        insert(&db, &new_profile(id, "default")).unwrap();
    }

    move_within_group(&db, "p1", Direction::Up).unwrap();
    move_within_group(&db, "p2", Direction::Down).unwrap();

    assert_eq!(ordered_ids(&db, "default"), ["p1", "p2"]);
}

#[test]
fn a_profile_dropped_onto_another_takes_its_place_and_shifts_the_rest() {
    let db = seeded_db();
    for id in ["p1", "p2", "p3", "p4"] {
        insert(&db, &new_profile(id, "default")).unwrap();
    }

    move_before(&db, "p4", "p2").unwrap();

    assert_eq!(ordered_ids(&db, "default"), ["p1", "p4", "p2", "p3"]);
}

#[test]
fn a_drop_onto_a_profile_in_another_group_is_refused() {
    let db = seeded_db();
    insert(&db, &new_profile("p1", "default")).unwrap();
    insert(&db, &new_profile("elsewhere", "custom")).unwrap();

    assert!(matches!(
        move_before(&db, "p1", "elsewhere"),
        Err(StorageError::NotFound)
    ));
}

#[test]
fn only_up_and_down_are_directions() {
    assert_eq!("up".parse::<Direction>().unwrap(), Direction::Up);
    assert_eq!("down".parse::<Direction>().unwrap(), Direction::Down);
    assert!(matches!(
        "sideways".parse::<Direction>(),
        Err(StorageError::InvalidInput(_))
    ));
}
