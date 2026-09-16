
use super::*;
use crate::clash_api::test_support::spawn_ws_mock;

#[tokio::test]
async fn receives_traffic_samples_in_order() {
    let url = spawn_ws_mock(vec![vec![
        r#"{"up":100,"down":200}"#.to_string(),
        r#"{"up":150,"down":250}"#.to_string(),
    ]])
    .await;
    let mut watcher = watch_traffic(url, Duration::from_millis(20));

    let first = watcher.events.recv().await.unwrap();
    let second = watcher.events.recv().await.unwrap();
    assert_eq!(
        first,
        TrafficEvent::Sample(TrafficSample { up: 100, down: 200 })
    );
    assert_eq!(
        second,
        TrafficEvent::Sample(TrafficSample { up: 150, down: 250 })
    );
    watcher.stop();
}

#[tokio::test]
async fn a_malformed_frame_is_skipped_without_killing_the_stream() {
    let url = spawn_ws_mock(vec![vec![
        "not json at all".to_string(),
        r#"{"up":1,"down":2}"#.to_string(),
    ]])
    .await;
    let mut watcher = watch_traffic(url, Duration::from_millis(20));

    let event = watcher.events.recv().await.unwrap();
    assert_eq!(
        event,
        TrafficEvent::Sample(TrafficSample { up: 1, down: 2 }),
        "the malformed frame must be skipped, not surfaced or fatal"
    );
    watcher.stop();
}

#[tokio::test]
async fn reconnects_after_the_server_drops_the_connection() {
    let url = spawn_ws_mock(vec![
        vec![r#"{"up":1,"down":1}"#.to_string()],
        vec![r#"{"up":2,"down":2}"#.to_string()],
    ])
    .await;
    let mut watcher = watch_traffic(url, Duration::from_millis(20));

    assert_eq!(
        watcher.events.recv().await.unwrap(),
        TrafficEvent::Sample(TrafficSample { up: 1, down: 1 })
    );
    assert_eq!(
        watcher.events.recv().await.unwrap(),
        TrafficEvent::Disconnected
    );
    assert_eq!(
        watcher.events.recv().await.unwrap(),
        TrafficEvent::Reconnecting
    );
    assert_eq!(
        watcher.events.recv().await.unwrap(),
        TrafficEvent::Sample(TrafficSample { up: 2, down: 2 }),
        "a fresh connection must be established after the drop"
    );
    watcher.stop();
}

#[tokio::test]
async fn connecting_to_a_dead_endpoint_reports_disconnected_and_keeps_retrying() {
    // Nothing listens on this port; the connect attempt itself fails.
    let mut watcher = watch_traffic("ws://127.0.0.1:1/traffic", Duration::from_millis(20));
    let first = watcher.events.recv().await.unwrap();
    let second = watcher.events.recv().await.unwrap();
    assert_eq!(first, TrafficEvent::Disconnected);
    assert_eq!(second, TrafficEvent::Reconnecting);
    watcher.stop();
}

#[tokio::test]
async fn stop_ends_the_background_task_and_no_more_events_arrive() {
    let url = spawn_ws_mock(vec![vec![r#"{"up":1,"down":1}"#.to_string()]]).await;
    let mut watcher = watch_traffic(url, Duration::from_millis(20));
    assert_eq!(
        watcher.events.recv().await.unwrap(),
        TrafficEvent::Sample(TrafficSample { up: 1, down: 1 })
    );
    watcher.stop();

    // The server closes after its one scripted frame, so a trailing
    // Disconnected/Reconnecting pair may still be in flight; drain
    // whatever arrives and require the channel to close soon after,
    // rather than reconnecting forever despite stop().
    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if watcher.events.recv().await.is_none() {
                return;
            }
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "background task should exit (and drop its sender) soon after stop()"
    );
}
