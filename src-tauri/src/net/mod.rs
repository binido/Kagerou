//! Requests that leave the machine.
//!
//! Three of them, each with a different rule about how it may go out: the
//! exit-location lookup only through the tunnel, the latency probe only
//! through the core being measured, and the update check directly, quietly.

pub mod geo;
pub mod probe;
pub mod updates;
