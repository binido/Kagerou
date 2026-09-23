use super::*;
use crate::clash_api::test_support::spawn_http_mock;
use crate::storage::models::ProviderInfo;
use crate::subscription::parse_subscription;

/// Two keys the parser accepts, named so a refresh can be seen to keep one
/// and drop the other.
const TOKYO: &str = "vless://uuid-a@1.2.3.4:443?security=tls#Tokyo";
const OSAKA: &str = "vless://uuid-b@5.6.7.8:443?security=tls#Osaka";
const KYOTO: &str = "vless://uuid-c@9.10.11.12:443?security=tls#Kyoto";

fn outbounds(keys: &[&str]) -> Vec<ParsedOutbound> {
    parse_subscription(&keys.join("\n")).unwrap().outbounds
}

/// A database holding one subscription group with Tokyo and Osaka in it.
fn subscribed() -> (Db, String) {
    let db = Db::open_in_memory().unwrap();
    let outcome = import::add_subscription(
        &db,
        "https://example.com/sub",
        "Example",
        &ProviderInfo::default(),
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

fn http_ok(body: &'static str) -> impl Fn(String) -> Vec<u8> + Send + Sync + 'static {
    move |_| {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }
}

#[tokio::test]
async fn a_refresh_with_nothing_importable_leaves_the_group_as_it_was() {
    let (db, source_id) = subscribed();
    let url = spawn_http_mock(http_ok("vless://uuid@example.com:443?type=xhttp#Gone")).await;
    update_source(&db, &source_id, None, Some(&url)).unwrap();

    let error = refresh(&db, &source_id).await.unwrap_err();

    assert!(matches!(error, SubscriptionsError::Parse(_)));
    assert_eq!(names(&db), vec!["Osaka".to_string(), "Tokyo".to_string()]);
}

#[tokio::test]
async fn a_refresh_reports_what_it_left_out() {
    let (db, source_id) = subscribed();
    let url = spawn_http_mock(http_ok(
        "vless://uuid-a@1.2.3.4:443?security=tls#Tokyo\nvless://uuid@example.com:443?type=xhttp#Gone",
    ))
    .await;
    update_source(&db, &source_id, None, Some(&url)).unwrap();

    let unsupported = refresh(&db, &source_id).await.unwrap();

    assert_eq!(unsupported, vec![Unsupported::Transport("xhttp".into())]);
    assert_eq!(names(&db), vec!["Tokyo".to_string()]);
}

#[tokio::test]
async fn a_refresh_keeps_what_the_provider_says_now_and_forgets_what_it_stopped_saying() {
    let (db, source_id) = subscribed();
    sources::set_provider(
        &db,
        &source_id,
        &ProviderInfo {
            announce: Some("Old news".into()),
            ..ProviderInfo::default()
        },
    )
    .unwrap();
    let url = spawn_http_mock(move |_| {
        format!(
            "HTTP/1.1 200 OK\r\n\
             subscription-userinfo: upload=1; download=2; total=100; expire=1767225600\r\n\
             support-url: https://support.example/\r\n\
             Content-Length: {}\r\n\r\n{TOKYO}",
            TOKYO.len()
        )
        .into_bytes()
    })
    .await;
    update_source(&db, &source_id, None, Some(&url)).unwrap();

    refresh(&db, &source_id).await.unwrap();

    assert_eq!(
        sources::get(&db, &source_id).unwrap().provider,
        ProviderInfo {
            traffic_used: Some(3),
            traffic_total: Some(100),
            expires_at: Some(1_767_225_600_000),
            announce: None,
            support_url: Some("https://support.example/".into()),
        }
    );
}
