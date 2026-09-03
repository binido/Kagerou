use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use super::model::TrafficSample;

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficEvent {
    Sample(TrafficSample),
    Disconnected,
    Reconnecting,
}

/// A live handle to a background task streaming `/traffic` samples from
/// the Clash API over WebSocket. Dropping or calling `stop()` ends the
/// background task; a connection drop is not fatal — the task reports
/// `Disconnected`/`Reconnecting` and keeps retrying on `reconnect_delay`
/// until stopped.
pub struct TrafficWatcher {
    pub events: mpsc::UnboundedReceiver<TrafficEvent>,
    stop: watch::Sender<bool>,
}

impl TrafficWatcher {
    pub fn stop(&self) {
        let _ = self.stop.send(true);
    }
}

pub fn watch_traffic(ws_url: impl Into<String>, reconnect_delay: Duration) -> TrafficWatcher {
    let ws_url = ws_url.into();
    let (tx, rx) = mpsc::unbounded_channel();
    let (stop_tx, mut stop_rx) = watch::channel(false);

    tokio::spawn(async move {
        loop {
            if *stop_rx.borrow() {
                return;
            }

            if let Ok((mut stream, _)) = tokio_tungstenite::connect_async(&ws_url).await {
                loop {
                    tokio::select! {
                        _ = stop_rx.changed() => {
                            if *stop_rx.borrow() {
                                return;
                            }
                        }
                        message = stream.next() => {
                            match message {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(sample) = serde_json::from_str::<TrafficSample>(&text) {
                                        if tx.send(TrafficEvent::Sample(sample)).is_err() {
                                            return;
                                        }
                                    }
                                    // A malformed frame is silently skipped rather than
                                    // tearing down the connection: one bad sample must
                                    // not interrupt an otherwise-healthy stream.
                                }
                                Some(Ok(Message::Close(_))) | None => break,
                                Some(Ok(_)) => {}
                                Some(Err(_)) => break,
                            }
                        }
                    }
                }
            }

            if tx.send(TrafficEvent::Disconnected).is_err() {
                return;
            }
            if *stop_rx.borrow() {
                return;
            }
            if tx.send(TrafficEvent::Reconnecting).is_err() {
                return;
            }
            tokio::time::sleep(reconnect_delay).await;
        }
    });

    TrafficWatcher {
        events: rx,
        stop: stop_tx,
    }
}

#[cfg(test)]
mod tests {
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
}
