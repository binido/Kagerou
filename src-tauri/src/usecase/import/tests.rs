use super::*;

const TOKYO: &str = "vless://uuid@tokyo.example:443?security=tls&sni=tokyo.example#Tokyo";
const RELAY: &str = "trojan://pass@relay.example:443#Relay";

fn outbounds(text: &str) -> Vec<ParsedOutbound> {
    match classify(text).unwrap() {
        Pasted::Outbounds(parsed) => parsed.outbounds,
        other => panic!("expected keys, got {other:?}"),
    }
}

#[test]
fn an_http_url_is_a_subscription() {
    assert_eq!(
        classify("  https://sub.example/list?token=abc\n").unwrap(),
        Pasted::SubscriptionUrl("https://sub.example/list?token=abc".into())
    );
    assert!(matches!(
        classify("http://sub.example").unwrap(),
        Pasted::SubscriptionUrl(_)
    ));
}

#[test]
fn only_a_single_http_link_is_a_subscription_url() {
    assert!(is_subscription_url(" https://sub.example/list?token=abc "));
    assert!(!is_subscription_url(TOKYO));
    assert!(!is_subscription_url("ftp://files.example/list"));
    assert!(!is_subscription_url("https://a.example https://b.example"));
    assert!(!is_subscription_url(""));
}

#[test]
fn keys_are_outbounds_one_or_many() {
    assert_eq!(outbounds(TOKYO).len(), 1);
    assert_eq!(outbounds(&format!("{TOKYO}\n\n{RELAY}\n")).len(), 2);
}

#[test]
fn empty_or_unrecognised_text_is_an_error() {
    assert!(matches!(
        classify("   \n").unwrap_err(),
        ImportError::Subscription(SubscriptionError::Empty)
    ));
    assert!(classify("ftp://files.example/list").is_err());
    assert!(classify("just some words someone copied").is_err());
    assert!(classify("https://sub.example/a https://sub.example/b").is_err());
}

#[test]
fn a_subscription_is_named_from_its_header_then_its_host() {
    assert_eq!(
        subscription_name(Some("base64:0JzQvtC5IFZQTg=="), "https://x.example"),
        "Мой VPN"
    );
    assert_eq!(
        subscription_name(Some(" Work "), "https://x.example"),
        "Work"
    );
    assert_eq!(
        subscription_name(Some("  "), "https://www.sub.example/p"),
        "sub.example"
    );
    assert_eq!(
        subscription_name(Some("base64:%%%"), "https://sub.example"),
        "sub.example",
        "an undecodable title falls back rather than showing garbage"
    );
    assert_eq!(subscription_name(None, "not a url"), "Subscription");
}

#[test]
fn flag_emoji_becomes_a_country_code() {
    assert_eq!(region_from_name("🇦🇹 ALL VPN | Австрия"), "AT");
    assert_eq!(region_from_name("  🇵🇱 ALL VPN"), "PL");
}

#[test]
fn names_without_a_flag_have_no_region() {
    assert_eq!(region_from_name("Fast Node 03"), "");
    assert_eq!(region_from_name(""), "");
    assert_eq!(region_from_name("🚀 Boost"), "");
    // A lone regional indicator is not a flag.
    assert_eq!(region_from_name("🇦 Node"), "");
}

#[test]
fn a_single_key_lands_in_default_as_a_local_profile() {
    let db = Db::open_in_memory().unwrap();

    let outcome = add_outbounds(&db, &outbounds(TOKYO)).unwrap();

    let ImportOutcome::ProfileAdded { profile_id, name } = outcome else {
        panic!("expected a profile, got {outcome:?}");
    };
    assert_eq!(name, "Tokyo");
    let profile = profiles::get(&db, &profile_id).unwrap();
    assert_eq!(profile.group_id, "default");
    assert_eq!(profile.origin, "local");
    assert_eq!(profile.source_id, None);
}

#[test]
fn the_same_key_under_another_name_is_already_present() {
    let db = Db::open_in_memory().unwrap();
    add_outbounds(&db, &outbounds(TOKYO)).unwrap();

    let renamed = TOKYO.replace("#Tokyo", "#Something else");
    let outcome = add_outbounds(&db, &outbounds(&renamed)).unwrap();

    assert_eq!(
        outcome,
        ImportOutcome::AlreadyPresent {
            group_id: "default".into()
        }
    );
    assert_eq!(profiles::list_all(&db).unwrap().len(), 1);
}

#[test]
fn several_keys_become_a_numbered_imported_group() {
    let db = Db::open_in_memory().unwrap();

    let first = add_outbounds(&db, &outbounds(&format!("{TOKYO}\n{RELAY}"))).unwrap();
    let second = add_outbounds(
        &db,
        &outbounds("trojan://a@a.example:443#A\ntrojan://b@b.example:443#B"),
    )
    .unwrap();

    let labels: Vec<_> = [first, second]
        .into_iter()
        .map(|outcome| match outcome {
            ImportOutcome::GroupAdded {
                group_id,
                added: 2,
                skipped: 0,
            } => groups::get(&db, &group_id).unwrap().label,
            other => panic!("expected a group of two, got {other:?}"),
        })
        .collect();
    assert_eq!(labels, vec!["Imported", "Imported 2"]);
}

#[test]
fn duplicates_in_a_batch_are_skipped_and_counted() {
    let db = Db::open_in_memory().unwrap();
    add_outbounds(&db, &outbounds(TOKYO)).unwrap();

    let outcome = add_outbounds(&db, &outbounds(&format!("{TOKYO}\n{RELAY}\n{RELAY}"))).unwrap();

    let ImportOutcome::GroupAdded {
        group_id,
        added,
        skipped,
    } = outcome
    else {
        panic!("expected a group, got {outcome:?}");
    };
    assert_eq!((added, skipped), (1, 2));
    assert_eq!(groups::get(&db, &group_id).unwrap().profile_ids.len(), 1);
}

#[test]
fn a_batch_of_nothing_but_duplicates_creates_no_group() {
    let db = Db::open_in_memory().unwrap();
    add_outbounds(&db, &outbounds(&format!("{TOKYO}\n{RELAY}"))).unwrap();
    let groups_before = groups::list_all(&db).unwrap().len();

    let outcome = add_outbounds(&db, &outbounds(&format!("{RELAY}\n{TOKYO}"))).unwrap();

    assert_eq!(outcome, ImportOutcome::NothingNew { skipped: 2 });
    assert_eq!(groups::list_all(&db).unwrap().len(), groups_before);
}

#[test]
fn a_subscription_gets_its_own_group_and_is_found_by_url() {
    let db = Db::open_in_memory().unwrap();
    let url = "https://sub.example/list";

    let outcome =
        add_subscription(&db, url, "Work", &outbounds(&format!("{TOKYO}\n{RELAY}"))).unwrap();

    let ImportOutcome::SubscriptionAdded { group_id, added: 2 } = outcome else {
        panic!("expected a subscription of two, got {outcome:?}");
    };
    let group = groups::get(&db, &group_id).unwrap();
    assert_eq!(
        (group.kind.as_str(), group.label.as_str()),
        ("subscription", "Work")
    );
    assert!(profiles::list_all(&db)
        .unwrap()
        .iter()
        .all(|p| p.origin == "imported" && p.source_id == group.source_id));
    assert_eq!(
        subscription_for_url(&db, url).unwrap().map(|g| g.id),
        Some(group_id)
    );
    assert_eq!(
        subscription_for_url(&db, "https://other.example").unwrap(),
        None
    );
}

fn subscription_with_active_profile(db: &Db) -> (String, String) {
    let ImportOutcome::SubscriptionAdded { group_id, .. } =
        add_subscription(db, "https://sub.example", "Work", &outbounds(TOKYO)).unwrap()
    else {
        unreachable!()
    };
    let active = groups::get(db, &group_id).unwrap().profile_ids[0].clone();
    settings::set_active_profile_id(db, Some(&active)).unwrap();
    (group_id, active)
}

#[test]
fn removing_a_subscription_takes_its_group_profiles_and_source() {
    let db = Db::open_in_memory().unwrap();
    let (group_id, _) = subscription_with_active_profile(&db);

    remove_subscription(&db, &group_id, None, true).unwrap();

    assert!(matches!(
        groups::get(&db, &group_id),
        Err(StorageError::NotFound)
    ));
    assert!(profiles::list_all(&db).unwrap().is_empty());
    assert!(sources::list_all(&db).unwrap().is_empty());
}

#[test]
fn removing_the_subscription_in_use_is_refused_while_connected() {
    let db = Db::open_in_memory().unwrap();
    let (group_id, active) = subscription_with_active_profile(&db);

    let err = remove_subscription(&db, &group_id, Some(&active), true).unwrap_err();

    assert!(matches!(err, ImportError::ActiveProfileInUse));
    assert_eq!(
        groups::get(&db, &group_id).unwrap().profile_ids,
        vec![active]
    );
}

#[test]
fn removing_the_subscription_in_use_while_disconnected_clears_the_active_choice() {
    let db = Db::open_in_memory().unwrap();
    let (group_id, active) = subscription_with_active_profile(&db);

    remove_subscription(&db, &group_id, Some(&active), false).unwrap();

    assert_eq!(settings::get_active_profile_id(&db).unwrap(), None);
    assert!(profiles::list_all(&db).unwrap().is_empty());
}

#[test]
fn only_subscription_groups_can_be_removed_this_way() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        remove_subscription(&db, "default", None, false).unwrap_err(),
        ImportError::NotASubscription(_)
    ));
    assert!(matches!(
        remove_subscription(&db, "ghost", None, false).unwrap_err(),
        ImportError::Storage(StorageError::NotFound)
    ));
}

#[test]
fn the_wire_format_matches_what_the_frontend_expects() {
    let json = serde_json::to_value(ImportOutcome::GroupAdded {
        group_id: "g".into(),
        added: 3,
        skipped: 1,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "kind": "groupAdded", "groupId": "g", "added": 3, "skipped": 1 })
    );
    let json = serde_json::to_value(ImportOutcome::AlreadyPresent {
        group_id: "default".into(),
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "kind": "alreadyPresent", "groupId": "default" })
    );
}

#[test]
fn what_was_left_out_rides_next_to_the_outcome() {
    let json = serde_json::to_value(Imported {
        outcome: ImportOutcome::AlreadyPresent {
            group_id: "default".into(),
        },
        unsupported: vec![Unsupported::Protocol("ssr".into())],
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "alreadyPresent",
            "groupId": "default",
            "unsupported": [{ "kind": "protocol", "name": "ssr" }],
        })
    );
}
