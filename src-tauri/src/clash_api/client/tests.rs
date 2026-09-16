
use super::*;
use crate::clash_api::test_support::spawn_http_mock;

#[tokio::test]
async fn version_happy_path() {
    let body = br#"{"version":"1.8.0","premium":true}"#;
    let base_url = spawn_http_mock(move |_req| http_response(200, "OK", body)).await;
    let client = ClashApiClient::new(base_url);
    let version = client.get_version().await.unwrap();
    assert_eq!(version.version, "1.8.0");
    assert!(version.premium);
}

#[tokio::test]
async fn non_success_status_becomes_an_http_error() {
    let base_url = spawn_http_mock(|_req| http_response(404, "Not Found", b"not found")).await;
    let client = ClashApiClient::new(base_url);
    let err = client.get_version().await.unwrap_err();
    match err {
        ClashApiError::Http { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "not found");
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_json_body_becomes_a_decode_error() {
    let base_url = spawn_http_mock(|_req| http_response(200, "OK", b"this is not json")).await;
    let client = ClashApiClient::new(base_url);
    let err = client.get_version().await.unwrap_err();
    assert!(matches!(err, ClashApiError::Decode(_)));
}

#[tokio::test]
async fn connection_refused_is_a_connection_error() {
    // Nothing is listening on this port.
    let client = ClashApiClient::new("http://127.0.0.1:1");
    let err = client.get_version().await.unwrap_err();
    assert!(matches!(err, ClashApiError::Connection(_)));
}

#[tokio::test]
async fn a_server_that_never_responds_times_out() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // Accept and hold the connection open (the accepted socket
        // must stay alive, not be dropped) without ever writing a
        // response, so the client has something to time out on.
        if let Ok((_socket, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
    let client = ClashApiClient::with_config(
        format!("http://{addr}"),
        reqwest::Client::new(),
        Duration::from_millis(200),
    );
    let err = client.get_version().await.unwrap_err();
    assert!(matches!(err, ClashApiError::Timeout));
}

#[tokio::test]
async fn select_outbound_sends_a_put_with_the_target_name() {
    use std::sync::{Arc, Mutex};
    let captured = Arc::new(Mutex::new(String::new()));
    let captured_clone = Arc::clone(&captured);
    let base_url = spawn_http_mock(move |req| {
        *captured_clone.lock().unwrap() = req;
        http_response(204, "No Content", b"")
    })
    .await;
    let client = ClashApiClient::new(base_url);
    client
        .select_outbound("proxy", "profile-tokyo")
        .await
        .unwrap();
    let request = captured.lock().unwrap().clone();
    assert!(
        request.starts_with("PUT /proxies/proxy"),
        "request was: {request}"
    );
    assert!(
        request.contains("profile-tokyo"),
        "request body should carry the target name: {request}"
    );
}

#[tokio::test]
async fn select_outbound_on_an_unknown_selector_is_reported() {
    let base_url =
        spawn_http_mock(|_req| http_response(404, "Not Found", b"proxy group not found")).await;
    let client = ClashApiClient::new(base_url);
    let err = client.select_outbound("proxy", "ghost").await.unwrap_err();
    assert!(matches!(err, ClashApiError::Http { status: 404, .. }));
}

fn http_response(status: u16, reason: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(body);
    response
}
