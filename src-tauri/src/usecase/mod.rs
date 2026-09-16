//! Application layer: what the app does, expressed once, above storage and
//! the sing-box process and below the Tauri commands that expose it.
//!
//! Everything here used to live in `commands.rs`, where it could not be
//! reached without an `AppHandle` and so went untested.

pub mod connection;
pub mod core;
pub mod error;
pub mod events;
pub mod subscriptions;
pub mod testing;
