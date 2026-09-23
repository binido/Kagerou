use super::*;

#[test]
fn user_agent_names_the_bundled_core_version() {
    let script = include_str!("../../../../scripts/fetch-singbox.mjs");
    assert!(
        script.contains(&format!("const VERSION = \"{SINGBOX_VERSION}\";")),
        "SINGBOX_VERSION is out of step with scripts/fetch-singbox.mjs"
    );
    assert!(user_agent().contains("singbox/"));
}

#[tokio::test]
async fn the_request_carries_the_user_agent_panels_match_on() {
    let url = crate::clash_api::test_support::spawn_http_mock(|request| {
        let body = if request
            .to_ascii_lowercase()
            .contains("user-agent: kagerou singbox/")
        {
            "matched"
        } else {
            "missing"
        };
        format!("HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\n{body}").into_bytes()
    })
    .await;
    assert_eq!(fetch(&url).await.unwrap().body, "matched");
}

#[tokio::test]
async fn a_provider_that_turns_this_client_away_is_told_apart_from_a_dead_link() {
    for status in ["401 Unauthorized", "403 Forbidden"] {
        let url = crate::clash_api::test_support::spawn_http_mock(move |_| {
            format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\n\r\n").into_bytes()
        })
        .await;
        assert!(matches!(fetch(&url).await, Err(FetchError::Refused(_))));
    }
    let url = crate::clash_api::test_support::spawn_http_mock(|_| {
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec()
    })
    .await;
    assert!(matches!(fetch(&url).await, Err(FetchError::Http(_))));
}
