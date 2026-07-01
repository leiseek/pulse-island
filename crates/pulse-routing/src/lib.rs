//! Route strength to honest action-label policy.
#![deny(missing_docs)]

use pulse_domain::{RouteStrength, TimestampMs};

/// User-facing route action label selected from route strength.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteActionLabel {
    /// Exact-only action that returns to the original task/thread/tab.
    OpenOriginalTask,
    /// Strong route action for a related agent/provider window.
    FocusAgentWindow,
    /// Useful route action for a verified workspace.
    OpenWorkspace,
    /// Weak route action for process-only evidence.
    ShowProcessDetails,
}

/// Bounded route evidence with an explicit freshness window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteEvidence {
    /// Claimed route strength while evidence is fresh.
    pub strength: RouteStrength,
    /// Timestamp when the evidence was observed.
    pub observed_at: TimestampMs,
    /// Time-to-live for this evidence in milliseconds.
    pub ttl_ms: u64,
}

impl RouteEvidence {
    /// Construct route evidence with explicit strength and freshness.
    pub const fn new(strength: RouteStrength, observed_at: TimestampMs, ttl_ms: u64) -> Self {
        Self {
            strength,
            observed_at,
            ttl_ms,
        }
    }
}

/// Return the strongest honest label allowed for the supplied route strength.
pub fn label_for(strength: RouteStrength) -> Option<RouteActionLabel> {
    match strength {
        RouteStrength::Exact => Some(RouteActionLabel::OpenOriginalTask),
        RouteStrength::Strong => Some(RouteActionLabel::FocusAgentWindow),
        RouteStrength::Useful => Some(RouteActionLabel::OpenWorkspace),
        RouteStrength::Weak => Some(RouteActionLabel::ShowProcessDetails),
        RouteStrength::None => None,
    }
}

/// Downgrade stale route evidence to an explicit fallback strength.
pub fn effective_strength(
    evidence: RouteEvidence,
    now: TimestampMs,
    fallback: RouteStrength,
) -> RouteStrength {
    let age = now.0.saturating_sub(evidence.observed_at.0);
    if age > evidence.ttl_ms {
        fallback
    } else {
        evidence.strength
    }
}

/// Return the honest label after freshness-based route downgrade.
pub fn label_for_evidence(
    evidence: RouteEvidence,
    now: TimestampMs,
    fallback: RouteStrength,
) -> Option<RouteActionLabel> {
    label_for(effective_strength(evidence, now, fallback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_is_never_exact() {
        assert_eq!(
            label_for(RouteStrength::Strong),
            Some(RouteActionLabel::FocusAgentWindow)
        );
    }

    #[test]
    fn stale_exact_route_downgrades_to_fallback_label() {
        let evidence = RouteEvidence::new(RouteStrength::Exact, TimestampMs(100), 10);
        assert_eq!(
            label_for_evidence(evidence, TimestampMs(111), RouteStrength::Useful),
            Some(RouteActionLabel::OpenWorkspace)
        );
    }
}
