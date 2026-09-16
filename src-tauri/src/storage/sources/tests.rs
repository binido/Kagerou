use super::*;

fn source(id: &str, value: &str) -> NewSource {
    NewSource {
        id: id.into(),
        name: format!("Source {id}"),
        kind: "url".into(),
        value: value.into(),
        status: "up-to-date".into(),
        last_refresh: "1757352766522".into(),
        origin_label: "Remote URL".into(),
    }
}

#[test]
fn refreshed_now_is_a_parseable_unix_millisecond_stamp() {
    let stamp = refreshed_now();
    let millis: i64 = stamp.parse().expect("the column holds a number, not prose");
    // 2020-01-01; a clock this far back means the stamp is not epoch millis.
    assert!(millis > 1_577_836_800_000, "got {stamp}");
}

#[test]
fn insert_then_get_round_trips() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &source("s1", "https://example.com/sub")).unwrap();
    let s = get(&db, "s1").unwrap();
    assert_eq!(s.value, "https://example.com/sub");
    assert_eq!(s.kind, "url");
}

#[test]
fn insert_rejects_blank_value() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        insert(&db, &source("s1", "   ")).unwrap_err(),
        StorageError::InvalidInput(_)
    ));
}

#[test]
fn update_is_a_partial_patch() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &source("s1", "https://example.com/sub")).unwrap();
    update(
        &db,
        "s1",
        &SourcePatch {
            name: None,
            value: None,
            status: Some("refresh-due"),
            last_refresh: None,
        },
    )
    .unwrap();
    let s = get(&db, "s1").unwrap();
    assert_eq!(s.status, "refresh-due");
    assert_eq!(
        s.name, "Source s1",
        "untouched fields must survive a partial patch"
    );
}

#[test]
fn update_rejects_blank_name() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &source("s1", "https://example.com/sub")).unwrap();
    let err = update(
        &db,
        "s1",
        &SourcePatch {
            name: Some("  "),
            value: None,
            status: None,
            last_refresh: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
}

#[test]
fn update_unknown_source_is_not_found() {
    let db = Db::open_in_memory().unwrap();
    let err = update(
        &db,
        "ghost",
        &SourcePatch {
            name: None,
            value: None,
            status: Some("ready"),
            last_refresh: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::NotFound));
}

#[test]
fn deleting_a_source_clears_but_does_not_delete_its_profiles() {
    let db = Db::open_in_memory().unwrap();
    crate::storage::groups::insert(
        &db,
        &crate::storage::models::NewProfileGroup {
            id: "g".into(),
            label: "G".into(),
            kind: "custom".into(),
            source_id: None,
        },
    )
    .unwrap();
    insert(&db, &source("s1", "https://example.com/sub")).unwrap();
    crate::storage::profiles::insert(
        &db,
        &crate::storage::models::NewProfile {
            id: "p1".into(),
            name: "P1".into(),
            region: "r".into(),
            protocol: crate::storage::models::Protocol::VLESS,
            origin: "imported".into(),
            group_id: "g".into(),
            source_id: Some("s1".into()),
            key: "vless://p1".into(),
        },
    )
    .unwrap();

    delete(&db, "s1").unwrap();

    let profile = crate::storage::profiles::get(&db, "p1").unwrap();
    assert_eq!(profile.source_id, None);
}
