//! Deterministic test helpers for Pulse Island core crates.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, ProviderId, TaskId, TimestampMs};

/// Fixed clock used by deterministic fixtures.
#[derive(Clone, Debug)]
pub struct FixedClock {
    now: TimestampMs,
}
impl FixedClock {
    /// Create a fixed clock at the given millisecond timestamp.
    pub fn new(ms: u64) -> Self {
        Self {
            now: TimestampMs(ms),
        }
    }

    /// Return the current fixed timestamp.
    pub fn now(&self) -> TimestampMs {
        self.now
    }

    /// Advance the clock by a deterministic delta.
    pub fn advance(&mut self, delta_ms: u64) {
        self.now.0 += delta_ms;
    }
}

/// Construct a safe provider id for tests.
pub fn provider(name: &str) -> Result<ProviderId, pulse_domain::DomainError> {
    BoundedText::new(name).map(ProviderId)
}

/// Construct a safe opaque task id for tests.
pub fn task(name: &str) -> Result<TaskId, pulse_domain::DomainError> {
    BoundedText::new(name).map(TaskId)
}
