
use super::*;
use crate::clash_api::test_support::spawn_http_mock;

fn ok_body(json: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{json}",
        json.len()
    )
    .into_bytes()
}

const SHORT: Duration = Duration::from_millis(500);

#[tokio::test]
async fn a_successful_lookup_carries_the_city_country_and_address() {
    let url = spawn_http_mock(|_| {
            ok_body(r#"{"ip":"81.2.69.142","success":true,"city":"London","country":"United Kingdom","country_code":"GB"}"#)
        })
        .await;
    let client = GeoClient::with_config(reqwest::Client::new(), url);

    assert_eq!(
        client.lookup(SHORT).await,
        Ok(ExitLocation {
            ip: "81.2.69.142".into(),
            city: "London".into(),
            country: "United Kingdom".into(),
            country_code: "GB".into(),
        })
    );
}

#[tokio::test]
async fn a_refusal_arrives_as_http_200_and_must_not_read_as_an_answer() {
    let url = spawn_http_mock(|_| {
        ok_body(r#"{"success":false,"message":"Reserved range","ip":"127.0.0.1"}"#)
    })
    .await;
    let client = GeoClient::with_config(reqwest::Client::new(), url);

    assert_eq!(
        client.lookup(SHORT).await,
        Err(GeoError::Rejected("Reserved range".into()))
    );
}

#[tokio::test]
async fn a_non_2xx_status_is_not_decoded() {
    let url = spawn_http_mock(|_| {
        b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n".to_vec()
    })
    .await;
    let client = GeoClient::with_config(reqwest::Client::new(), url);

    assert_eq!(client.lookup(SHORT).await, Err(GeoError::Http(429)));
}

#[tokio::test]
async fn a_malformed_body_is_a_decode_error_and_not_a_panic() {
    let url = spawn_http_mock(|_| ok_body("not json at all")).await;
    let client = GeoClient::with_config(reqwest::Client::new(), url);

    assert!(matches!(
        client.lookup(SHORT).await,
        Err(GeoError::Decode(_))
    ));
}

#[tokio::test]
async fn a_refused_connection_is_unreachable() {
    // Port 1 on loopback: nothing is listening, and binding it needs root.
    let client = GeoClient::with_config(reqwest::Client::new(), "http://127.0.0.1:1/");

    assert!(matches!(
        client.lookup(SHORT).await,
        Err(GeoError::Unreachable(_))
    ));
}

#[tokio::test]
async fn a_server_that_never_answers_times_out() {
    let url = spawn_http_mock(|_| {
        std::thread::sleep(Duration::from_secs(2));
        ok_body("{}")
    })
    .await;
    let client = GeoClient::with_config(reqwest::Client::new(), url);

    assert_eq!(client.lookup(SHORT).await, Err(GeoError::Timeout));
}

#[test]
fn only_a_proxy_that_never_answered_is_worth_asking_again() {
    assert!(is_transient(&GeoError::Unreachable("refused".into())));
    assert!(is_transient(&GeoError::Timeout));
    assert!(
        !is_transient(&GeoError::Rejected("Reserved range".into())),
        "the service answered; asking again gets the same answer"
    );
    assert!(
        !is_transient(&GeoError::Http(429)),
        "retrying a rate limit is how a rate limit becomes a ban"
    );
    assert!(!is_transient(&GeoError::Decode("bad json".into())));
}

#[test]
fn a_missing_city_is_an_empty_string_rather_than_a_failed_lookup() {
    let body = serde_json::from_str::<Response>(
        r#"{"success":true,"ip":"1.1.1.1","country":"Australia","country_code":"AU"}"#,
    )
    .expect("partial payloads are still payloads");
    let location = to_location(body).expect("a lookup without a city still located the exit");
    assert_eq!(location.city, "");
    assert_eq!(location.country, "Australia");
}
