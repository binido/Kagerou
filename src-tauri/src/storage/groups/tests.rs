
use super::*;

fn group(id: &str, label: &str, kind: &str) -> NewProfileGroup {
    NewProfileGroup {
        id: id.into(),
        label: label.into(),
        kind: kind.into(),
        source_id: None,
    }
}

#[test]
fn insert_then_list_preserves_insertion_order() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &group("a", "A", "custom")).unwrap();
    insert(&db, &group("b", "B", "custom")).unwrap();
    let labels: Vec<_> = list_all(&db)
        .unwrap()
        .into_iter()
        .map(|g| g.label)
        .collect();
    // The base migration seeds "Default" first, so it leads.
    assert_eq!(labels, vec!["Default", "A", "B"]);
}

#[test]
fn insert_rejects_blank_label() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        insert(&db, &group("a", "   ", "custom")).unwrap_err(),
        StorageError::InvalidInput(_)
    ));
}

#[test]
fn insert_rejects_duplicate_ids() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &group("a", "A", "custom")).unwrap();
    assert!(matches!(
        insert(&db, &group("a", "A2", "custom")).unwrap_err(),
        StorageError::Sqlite(_)
    ));
}

#[test]
fn rename_refuses_the_default_group() {
    // The base migration always seeds the "default" group already.
    let db = Db::open_in_memory().unwrap();
    let err = rename(&db, "default", "Renamed").unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
    assert_eq!(get(&db, "default").unwrap().label, "Default");
}

#[test]
fn rename_unknown_group_is_not_found() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        rename(&db, "ghost", "New").unwrap_err(),
        StorageError::NotFound
    ));
}

#[test]
fn get_reports_profile_ids_in_position_order() {
    let db = Db::open_in_memory().unwrap();
    insert(&db, &group("g", "G", "custom")).unwrap();
    {
        let conn = db.lock();
        conn.execute_batch(
                "INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, url_value, url_tone, key, position) VALUES
                   ('p2','P2','r','VLESS','local','g',NULL,0,'','muted','k2',1),
                   ('p1','P1','r','VLESS','local','g',NULL,0,'','muted','k1',0);",
            )
            .unwrap();
    }
    let g = get(&db, "g").unwrap();
    assert_eq!(g.profile_ids, vec!["p1", "p2"]);
}
