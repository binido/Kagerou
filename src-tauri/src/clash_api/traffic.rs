use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use super::model::TrafficSample;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
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

    /// Splits the watcher into its receiver and a cloneable stop handle,
    /// for callers that want to move the receiver into a forwarding task
    /// (e.g. one that re-emits each `TrafficEvent` as a Tauri event) while
    /// keeping the ability to stop that task from elsewhere.
    pub fn into_parts(self) -> (mpsc::UnboundedReceiver<TrafficEvent>, watch::Sender<bool>) {
        (self.events, self.stop)
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
mod tests;
