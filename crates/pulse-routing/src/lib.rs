//! Route strength to honest action-label policy.
#![deny(missing_docs)]

use pulse_domain::{RouteStrength, TimestampMs};

/// User-facing route action label selected from route strength.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteActionLabel {
    /// Exact-only action that returns to the original task/thread/tab.
    OpenOriginalTask,
    /// Exact-only action for a documented provider thread route.
    OpenProviderThread,
    /// Exact-only action for a verified terminal tab/session route.
    FocusTerminalTab,
    /// Strong route action for a related agent/provider window.
    FocusAgentWindow,
    /// Strong route action for a related terminal without exact tab proof.
    FocusRelatedTerminal,
    /// Useful route action for a verified workspace.
    OpenWorkspace,
    /// Useful route action for revealing a verified project folder.
    RevealProjectFolder,
    /// Useful route action for opening a provider/agent surface.
    OpenAgent,
    /// Useful route action for opening an official usage surface.
    OpenOfficialUsage,
    /// Weak route action for process-only evidence.
    ShowProcessDetails,
}

/// Provider-neutral route kind declared by routing evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKind {
    /// Generic exact original-task route.
    OpenOriginalTask,
    /// Documented provider task/thread route.
    OpenProviderThread,
    /// Verified exact terminal tab/session route.
    FocusTerminalTab,
    /// Validated related provider/agent window.
    FocusAgentWindow,
    /// Validated related terminal without exact tab proof.
    FocusRelatedTerminal,
    /// Verified workspace route.
    OpenWorkspace,
    /// Verified folder reveal route.
    RevealProjectFolder,
    /// Verified provider/agent surface route.
    OpenAgent,
    /// Verified official usage destination.
    OpenOfficialUsage,
    /// Weak safe process details surface.
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

/// Return an honest route label only when the kind is allowed for the supplied strength.
pub fn label_for_kind(strength: RouteStrength, kind: RouteKind) -> Option<RouteActionLabel> {
    match (strength, kind) {
        (RouteStrength::Exact, RouteKind::OpenOriginalTask) => {
            Some(RouteActionLabel::OpenOriginalTask)
        }
        (RouteStrength::Exact, RouteKind::OpenProviderThread) => {
            Some(RouteActionLabel::OpenProviderThread)
        }
        (RouteStrength::Exact, RouteKind::FocusTerminalTab) => {
            Some(RouteActionLabel::FocusTerminalTab)
        }
        (RouteStrength::Strong, RouteKind::FocusAgentWindow) => {
            Some(RouteActionLabel::FocusAgentWindow)
        }
        (RouteStrength::Strong, RouteKind::FocusRelatedTerminal) => {
            Some(RouteActionLabel::FocusRelatedTerminal)
        }
        (RouteStrength::Useful, RouteKind::OpenWorkspace) => Some(RouteActionLabel::OpenWorkspace),
        (RouteStrength::Useful, RouteKind::RevealProjectFolder) => {
            Some(RouteActionLabel::RevealProjectFolder)
        }
        (RouteStrength::Useful, RouteKind::OpenAgent) => Some(RouteActionLabel::OpenAgent),
        (RouteStrength::Useful, RouteKind::OpenOfficialUsage) => {
            Some(RouteActionLabel::OpenOfficialUsage)
        }
        (RouteStrength::Weak, RouteKind::ShowProcessDetails) => {
            Some(RouteActionLabel::ShowProcessDetails)
        }
        _ => None,
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

    #[test]
    fn route_kind_labels_must_match_strength() {
        assert_eq!(
            label_for_kind(RouteStrength::Exact, RouteKind::OpenProviderThread),
            Some(RouteActionLabel::OpenProviderThread)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Exact, RouteKind::FocusTerminalTab),
            Some(RouteActionLabel::FocusTerminalTab)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Strong, RouteKind::FocusRelatedTerminal),
            Some(RouteActionLabel::FocusRelatedTerminal)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Useful, RouteKind::RevealProjectFolder),
            Some(RouteActionLabel::RevealProjectFolder)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Useful, RouteKind::OpenAgent),
            Some(RouteActionLabel::OpenAgent)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Useful, RouteKind::OpenOfficialUsage),
            Some(RouteActionLabel::OpenOfficialUsage)
        );
        assert_eq!(
            label_for_kind(RouteStrength::Strong, RouteKind::OpenOriginalTask),
            None
        );
        assert_eq!(
            label_for_kind(RouteStrength::Strong, RouteKind::FocusTerminalTab),
            None
        );
    }
}
