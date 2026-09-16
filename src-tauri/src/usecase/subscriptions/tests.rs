use super::*;
use crate::subscription::parse_subscription;

/// Two keys the parser accepts, named so a refresh can be seen to keep one
/// and drop the other.
const TOKYO: &str = "vless://uuid-a@1.2.3.4:443?security=tls#Tokyo";
const OSAKA: &str = "vless://uuid-b@5.6.7.8:443?security=tls#Osaka";
const KYOTO: &str = "vless://uuid-c@9.10.11.12:443?security=tls#Kyoto";

fn outbounds(keys: &[&str]) -> Vec<ParsedOutbound> {
    parse_subscription(&keys.join("\n")).unwrap()
}

/// A database holding one subscription group with Tokyo and Osaka in it.
fn subscribed() -> (Db, String) {
    let db = Db::open_in_memory().unwrap();
    let outcome = import::add_subscription(
        &db,
        "https://example.com/sub",
        "Example",
        &outbounds(&[TOKYO, OSAKA]),
    )
    .unwrap();
    let ImportOutcome::SubscriptionAdded { group_id, .. } = outcome else {
        panic!("expected a subscription, got {outcome:?}");
    };
    let source_id = groups::get(&db, &group_id).unwrap().source_id.unwrap();
    (db, source_id)
}

fn names(db: &Db) -> Vec<String> {
    let mut names: Vec<String> = profiles::list_all(db)
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    names.sort();
    names
}

#[test]
fn a_refresh_keeps_the_id_of_a_profile_the_provider_still_offers() {
    let (db, source_id) = subscribed();
    let before: Vec<(String, String)> = profiles::list_all(&db)
        .unwrap()
        .into_iter()
        .map(|p| (p.key, p.id))
        .collect();
    let tokyo_id = before
        .iter()
        .find(|(key, _)| key.contains("uuid-a"))
        .map(|(_, id)| id.clone())
        .unwrap();

    replace_group_profiles(&db, &source_id, &outbounds(&[TOKYO, KYOTO])).unwrap();

    let after = profiles::list_all(&db).unwrap();
    let tokyo = after.iter().find(|p| p.key.contains("uuid-a")).unwrap();
    assert_eq!(
        tokyo.id, tokyo_id,
        "keeping the id is what keeps a selection pointed at the same server"
    );
}

#[test]
fn a_refresh_drops_what_the_provider_stopped_offering_and_adds_what_is_new() {
    let (db, source_id) = subscribed();

    replace_group_profiles(&db, &source_id, &outbounds(&[TOKYO, KYOTO])).unwrap();

    assert_eq!(names(&db), vec!["Kyoto".to_string(), "Tokyo".to_string()]);
}

#[test]
fn a_refresh_that_comes_back_empty_empties_the_group() {
    let (db, source_id) = subscribed();

    replace_group_profiles(&db, &source_id, &[]).unwrap();

    assert!(names(&db).is_empty());
}

#[test]
fn a_refresh_records_when_it_happened() {
    let (db, source_id) = subscribed();
    sources::update(
        &db,
        &source_id,
        &sources::SourcePatch {
            name: None,
            value: None,
            status: None,
            last_refresh: Some(""),
        },
    )
    .unwrap();

    replace_group_profiles(&db, &source_id, &outbounds(&[TOKYO])).unwrap();

    let stamp = sources::get(&db, &source_id).unwrap().last_refresh;
    assert!(
        stamp.parse::<u64>().is_ok(),
        "the stamp is unix milliseconds, not prose: {stamp:?}"
    );
}

#[test]
fn a_source_with_no_group_behind_it_is_reported_rather_than_silently_ignored() {
    let db = Db::open_in_memory().unwrap();
    let error =
        replace_group_profiles(&db, "source-nobody-owns", &outbounds(&[TOKYO])).unwrap_err();
    assert!(matches!(error, SubscriptionsError::NoGroup));
}

#[test]
fn a_subscription_url_has_to_be_an_http_link() {
    let (db, source_id) = subscribed();
    let error = update_source(&db, &source_id, None, Some("not a url")).unwrap_err();
    assert!(matches!(error, SubscriptionsError::NotASubscriptionUrl));
    assert_eq!(
        sources::get(&db, &source_id).unwrap().value,
        "https://example.com/sub",
        "a rejected URL must not have been written first"
    );
}

#[test]
fn a_renamed_subscription_keeps_its_url() {
    let (db, source_id) = subscribed();
    update_source(&db, &source_id, Some("Renamed"), None).unwrap();
    let source = sources::get(&db, &source_id).unwrap();
    assert_eq!(source.name, "Renamed");
    assert_eq!(source.value, "https://example.com/sub");
}
