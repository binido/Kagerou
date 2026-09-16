
use super::*;

#[test]
fn the_base_migration_seeds_the_default_presets() {
    let db = Db::open_in_memory().unwrap();
    let presets = list_presets(&db).unwrap();
    let ids: Vec<_> = presets.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids, vec!["bypass-lan", "block-ads"]);
    assert!(presets.iter().all(|p| p.enabled), "defaults ship enabled");
}

#[test]
fn set_preset_toggles_enabled_flag() {
    let db = Db::open_in_memory().unwrap();
    set_preset(&db, "bypass-lan", false).unwrap();
    let presets = list_presets(&db).unwrap();
    assert!(!presets[0].enabled);
}

#[test]
fn set_unknown_preset_is_not_found() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        set_preset(&db, "ghost", true).unwrap_err(),
        StorageError::NotFound
    ));
}

#[test]
fn insert_rule_rejects_blank_match_pattern() {
    let db = Db::open_in_memory().unwrap();
    let err = insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "  ".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
}

#[test]
fn select_rule_deselects_all_others() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r2".into(),
            match_value: "b".into(),
            outbound: "Proxy".into(),
        },
    )
    .unwrap();
    select_rule(&db, "r1").unwrap();
    select_rule(&db, "r2").unwrap();
    let rules = list_rules(&db).unwrap();
    let selected: Vec<_> = rules
        .iter()
        .filter(|r| r.selected)
        .map(|r| r.id.clone())
        .collect();
    assert_eq!(selected, vec!["r2"]);
}

#[test]
fn delete_rule_removes_it_and_keeps_the_order_of_the_rest() {
    let db = Db::open_in_memory().unwrap();
    for id in ["r1", "r2", "r3"] {
        insert_rule(
            &db,
            &NewRoutingRule {
                id: id.into(),
                match_value: id.into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
    }
    delete_rule(&db, "r2").unwrap();
    let ids: Vec<_> = list_rules(&db).unwrap().into_iter().map(|r| r.id).collect();
    assert_eq!(ids, vec!["r1", "r3"]);
}

#[test]
fn deleting_the_selected_rule_leaves_nothing_selected() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    select_rule(&db, "r1").unwrap();
    delete_rule(&db, "r1").unwrap();
    assert!(list_rules(&db).unwrap().is_empty());
}

#[test]
fn deleting_an_unknown_rule_is_not_found() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        delete_rule(&db, "ghost").unwrap_err(),
        StorageError::NotFound
    ));
}

#[test]
fn select_unknown_rule_leaves_selection_unchanged() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    select_rule(&db, "r1").unwrap();
    assert!(matches!(
        select_rule(&db, "ghost").unwrap_err(),
        StorageError::NotFound
    ));
    assert!(list_rules(&db).unwrap()[0].selected);
}

#[test]
fn update_rule_outbound_rejects_unknown_rule() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        update_rule_outbound(&db, "ghost", "Block").unwrap_err(),
        StorageError::NotFound
    ));
}

#[test]
fn update_rule_outbound_rejects_a_value_outside_the_allowed_set() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    let err = update_rule_outbound(&db, "r1", "Teleport").unwrap_err();
    assert!(
        matches!(err, StorageError::Sqlite(_)),
        "the CHECK constraint should reject an unknown outbound"
    );
}

#[test]
fn update_rule_patches_only_the_provided_fields() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    update_rule(
        &db,
        "r1",
        &RulePatch {
            match_value: Some("b.example.com"),
            outbound: None,
        },
    )
    .unwrap();
    let rule = list_rules(&db)
        .unwrap()
        .into_iter()
        .find(|r| r.id == "r1")
        .unwrap();
    assert_eq!(rule.match_value, "b.example.com");
    assert_eq!(
        rule.outbound, "Direct",
        "untouched field must survive a partial patch"
    );
}

#[test]
fn update_rule_rejects_a_blank_match_pattern() {
    let db = Db::open_in_memory().unwrap();
    insert_rule(
        &db,
        &NewRoutingRule {
            id: "r1".into(),
            match_value: "a".into(),
            outbound: "Direct".into(),
        },
    )
    .unwrap();
    let err = update_rule(
        &db,
        "r1",
        &RulePatch {
            match_value: Some("   "),
            outbound: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::InvalidInput(_)));
}

#[test]
fn update_rule_on_an_unknown_rule_is_not_found() {
    let db = Db::open_in_memory().unwrap();
    let err = update_rule(
        &db,
        "ghost",
        &RulePatch {
            match_value: None,
            outbound: Some("Block"),
        },
    )
    .unwrap_err();
    assert!(matches!(err, StorageError::NotFound));
}
