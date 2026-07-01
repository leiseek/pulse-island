//! Pure deterministic task-state reducer.
#![deny(missing_docs)]
use pulse_domain::*;
use pulse_protocol::{AdmittedEvent, EvidenceKind};

/// Result of applying one admitted event to a compact snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReductionResult {
    /// Updated compact task snapshot.
    pub snapshot: TaskSnapshot,
    /// Whether persistence is allowed to retain a compact breadcrumb.
    pub retain_breadcrumb: bool,
}

/// Construct an initial generic snapshot for an admitted event.
pub fn initial(event: &AdmittedEvent, now: TimestampMs) -> TaskSnapshot {
    TaskSnapshot::generic(
        ProviderId(event.provider.clone()),
        TaskId(event.task.clone()),
        now,
    )
}

/// Apply one admitted provider-neutral event without I/O, clocks, or provider APIs.
pub fn reduce(
    mut prior: TaskSnapshot,
    event: &AdmittedEvent,
    now: TimestampMs,
    privacy: PrivacyProfile,
) -> ReductionResult {
    let was_terminal = prior.lifecycle.is_terminal();
    match &event.evidence {
        EvidenceKind::ProcessObserved { process } => {
            prior.provider_status = ProviderReleaseStatus::ProcessObserved;
            prior.health = TaskHealth::Observed;
            prior.summary = SafeSummary::ObservedProcess;
            prior.process = Some(process.clone());
            if !was_terminal && matches!(prior.lifecycle, Lifecycle::Unknown) {
                prior.lifecycle = Lifecycle::Observed;
            }
            if matches!(prior.route_strength, RouteStrength::None) {
                prior.route_strength = RouteStrength::Weak;
            }
        }
        EvidenceKind::Started => {
            if !was_terminal {
                prior.lifecycle = Lifecycle::Running;
                prior.health = TaskHealth::Attached;
                prior.attention = Attention::Active;
            }
        }
        EvidenceKind::Activity => {
            if !was_terminal
                && !matches!(prior.lifecycle, Lifecycle::WaitingUser | Lifecycle::Limited)
            {
                prior.lifecycle = Lifecycle::Running;
                prior.health = TaskHealth::Attached;
                prior.attention = Attention::Active;
            }
        }
        EvidenceKind::Waiting => {
            if !was_terminal {
                prior.lifecycle = Lifecycle::WaitingUser;
                prior.health = TaskHealth::Attached;
                prior.attention = Attention::Waiting;
                prior.summary = SafeSummary::WaitingForUser;
            }
        }
        EvidenceKind::WaitingCleared => {
            if !was_terminal {
                prior.lifecycle = Lifecycle::Running;
                prior.attention = Attention::Active;
            }
        }
        EvidenceKind::Completed => {
            prior.lifecycle = Lifecycle::Completed;
            prior.attention = Attention::None;
        }
        EvidenceKind::Failed => {
            prior.lifecycle = Lifecycle::Failed;
            prior.attention = Attention::Failed;
            prior.summary = SafeSummary::Failed;
        }
        EvidenceKind::LimitBlocked => {
            if !was_terminal {
                prior.lifecycle = Lifecycle::Limited;
                prior.attention = Attention::Limited;
                prior.summary = SafeSummary::LimitReached;
                prior.fuel_blocking = true;
            }
        }
        EvidenceKind::FuelRisk => {
            prior.fuel_risk = true;
        }
        EvidenceKind::FuelRevoked => {
            prior.fuel_risk = false;
            prior.fuel_blocking = false;
        }
        EvidenceKind::Route(strength) => {
            prior.route_strength = *strength;
            prior.route_capability = match strength {
                RouteStrength::Exact => RouteCapability::ContextReady,
                RouteStrength::Strong => RouteCapability::AgentReady,
                RouteStrength::Useful => RouteCapability::WorkspaceReady,
                RouteStrength::Weak | RouteStrength::None => RouteCapability::None,
            };
        }
    }
    prior.updated_at = now;
    let retain_breadcrumb = !(privacy == PrivacyProfile::Strict && prior.lifecycle.is_terminal())
        && privacy != PrivacyProfile::PassiveOnly;
    ReductionResult {
        snapshot: prior,
        retain_breadcrumb,
    }
}

/// Apply freshness decay without inventing terminal or waiting state.
pub fn apply_freshness(
    mut snapshot: TaskSnapshot,
    now: TimestampMs,
    stale_after_ms: u64,
) -> TaskSnapshot {
    let age = now.0.saturating_sub(snapshot.updated_at.0);
    if snapshot.health == TaskHealth::Attached && age > stale_after_ms {
        snapshot.health = TaskHealth::Degraded;
        snapshot.updated_at = now;
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_protocol::*;
    use pulse_testkit::*;
    fn ev(kind: EvidenceKind) -> Result<AdmittedEvent, Box<dyn std::error::Error>> {
        Ok(AdmittedEvent {
            provider: provider("Codex")?.0,
            task: task("opaque-1")?.0,
            evidence: kind,
            occurred_at: TimestampMs(1),
        })
    }
    #[test]
    fn lifecycle_and_terminal_protection() -> Result<(), Box<dyn std::error::Error>> {
        let e = ev(EvidenceKind::Started)?;
        let s = initial(&e, TimestampMs(1));
        let s = reduce(s, &e, TimestampMs(1), PrivacyProfile::Minimal).snapshot;
        assert_eq!(s.lifecycle, Lifecycle::Running);
        let done = ev(EvidenceKind::Completed)?;
        let s = reduce(s, &done, TimestampMs(2), PrivacyProfile::Minimal).snapshot;
        let late = ev(EvidenceKind::Activity)?;
        let s = reduce(s, &late, TimestampMs(3), PrivacyProfile::Minimal).snapshot;
        assert_eq!(s.lifecycle, Lifecycle::Completed);
        Ok(())
    }
    #[test]
    fn waiting_truthfulness_and_clear() -> Result<(), Box<dyn std::error::Error>> {
        let e = ev(EvidenceKind::Waiting)?;
        let s = initial(&e, TimestampMs(1));
        let s = reduce(s, &e, TimestampMs(1), PrivacyProfile::Minimal).snapshot;
        assert_eq!(s.lifecycle, Lifecycle::WaitingUser);
        let s = reduce(
            s,
            &ev(EvidenceKind::Activity)?,
            TimestampMs(2),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(s.lifecycle, Lifecycle::WaitingUser);
        let s = reduce(
            s,
            &ev(EvidenceKind::WaitingCleared)?,
            TimestampMs(3),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(s.lifecycle, Lifecycle::Running);
        Ok(())
    }
    #[test]
    fn freshness_degrades_and_fresh_source_recovers() -> Result<(), Box<dyn std::error::Error>> {
        let e = ev(EvidenceKind::Started)?;
        let s = reduce(
            initial(&e, TimestampMs(1)),
            &e,
            TimestampMs(1),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(s.health, TaskHealth::Attached);
        let stale = apply_freshness(s, TimestampMs(10_000), 100);
        assert_eq!(stale.health, TaskHealth::Degraded);
        let recovered = reduce(
            stale,
            &ev(EvidenceKind::Activity)?,
            TimestampMs(10_001),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(recovered.health, TaskHealth::Attached);
        Ok(())
    }

    #[test]
    fn process_only_ceiling_and_strict_retention() -> Result<(), Box<dyn std::error::Error>> {
        let p = ProcessFingerprint {
            pid: 7,
            start_ms: TimestampMs(10),
        };
        let e = ev(EvidenceKind::ProcessObserved { process: p })?;
        let s = reduce(
            initial(&e, TimestampMs(1)),
            &e,
            TimestampMs(1),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(s.lifecycle, Lifecycle::Observed);
        assert_eq!(s.health, TaskHealth::Observed);
        assert!(!s.fuel_risk);
        let r = reduce(
            s,
            &ev(EvidenceKind::Failed)?,
            TimestampMs(2),
            PrivacyProfile::Strict,
        );
        assert!(!r.retain_breadcrumb);
        Ok(())
    }
    #[test]
    fn malformed_oversized_rejected_before_mutation() -> Result<(), Box<dyn std::error::Error>> {
        let base = ev(EvidenceKind::Started)?;
        let s = initial(&base, TimestampMs(1));
        let bad = PulseHookEnvelope {
            version: 1,
            provider: base.provider,
            task: base.task,
            evidence: EvidenceKind::Started,
            byte_len: MAX_FRAME_BYTES + 1,
            forbidden_field_seen: false,
            structured_source_approved: true,
            occurred_at: TimestampMs(1),
        };
        assert_eq!(admit(bad), Err(RejectionCategory::Oversized));
        assert_eq!(s.lifecycle, Lifecycle::Unknown);
        Ok(())
    }
}

/// Resolved identity target for the pure identity-safety slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedTarget {
    /// Evidence belongs to a known task key.
    Known(TaskId),
    /// Evidence must create a separate task key.
    New(TaskId),
    /// Conflicting evidence must degrade/split rather than merge blindly.
    SplitRequired,
}

/// Minimal in-memory identity index for deterministic reducer fixtures.
#[derive(Clone, Debug, Default)]
pub struct IdentityIndex {
    entries: Vec<(ProcessFingerprint, TaskId)>,
}

impl IdentityIndex {
    /// Record a PID/start-time fingerprint for a task.
    pub fn record(&mut self, process: ProcessFingerprint, task: TaskId) {
        self.entries.push((process, task));
    }

    /// Resolve by PID and process start time; PID reuse with a new start time creates a new task.
    pub fn resolve_process(
        &self,
        process: &ProcessFingerprint,
        proposed: TaskId,
    ) -> ResolvedTarget {
        let same_pid: Vec<_> = self
            .entries
            .iter()
            .filter(|(known, _)| known.pid == process.pid)
            .collect();
        if let Some((_, task)) = same_pid
            .iter()
            .find(|(known, _)| known.start_ms == process.start_ms)
        {
            return ResolvedTarget::Known((*task).clone());
        }
        ResolvedTarget::New(proposed)
    }

    /// Conflicting session IDs for one identical process fingerprint are split, never merged.
    pub fn resolve_session_conflict(
        &self,
        process: &ProcessFingerprint,
        proposed: &TaskId,
    ) -> ResolvedTarget {
        if self
            .entries
            .iter()
            .any(|(known, task)| known == process && task != proposed)
        {
            ResolvedTarget::SplitRequired
        } else {
            ResolvedTarget::New(proposed.clone())
        }
    }
}

#[cfg(test)]
mod cross_layer_tests {
    use super::*;
    use pulse_protocol::*;
    use pulse_testkit::*;

    fn admitted(
        provider_name: &str,
        kind: EvidenceKind,
    ) -> Result<AdmittedEvent, Box<dyn std::error::Error>> {
        Ok(AdmittedEvent {
            provider: provider(provider_name)?.0,
            task: task("opaque-cross")?.0,
            evidence: kind,
            occurred_at: TimestampMs(1),
        })
    }

    #[test]
    fn process_only_antigravity_has_observed_ceiling() -> Result<(), Box<dyn std::error::Error>> {
        let event = admitted(
            "Antigravity",
            EvidenceKind::ProcessObserved {
                process: ProcessFingerprint {
                    pid: 42,
                    start_ms: TimestampMs(100),
                },
            },
        )?;
        let snapshot = reduce(
            initial(&event, TimestampMs(1)),
            &event,
            TimestampMs(1),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(
            snapshot.provider_status,
            ProviderReleaseStatus::ProcessObserved
        );
        assert_eq!(snapshot.health, TaskHealth::Observed);
        assert_eq!(snapshot.lifecycle, Lifecycle::Observed);
        assert!(!matches!(
            snapshot.lifecycle,
            Lifecycle::Running
                | Lifecycle::WaitingUser
                | Lifecycle::Completed
                | Lifecycle::Failed
                | Lifecycle::Limited
        ));
        assert!(!snapshot.fuel_risk);
        assert!(!snapshot.fuel_blocking);
        Ok(())
    }

    #[test]
    fn fuel_revoked_leaves_lifecycle_unchanged() -> Result<(), Box<dyn std::error::Error>> {
        let start = admitted("Codex", EvidenceKind::Started)?;
        let snapshot = reduce(
            initial(&start, TimestampMs(1)),
            &start,
            TimestampMs(1),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        let snapshot = reduce(
            snapshot,
            &admitted("Codex", EvidenceKind::FuelRisk)?,
            TimestampMs(2),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        let snapshot = reduce(
            snapshot,
            &admitted("Codex", EvidenceKind::FuelRevoked)?,
            TimestampMs(3),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        assert_eq!(snapshot.lifecycle, Lifecycle::Running);
        assert!(!snapshot.fuel_risk);
        assert!(!snapshot.fuel_blocking);
        Ok(())
    }

    #[test]
    fn pid_identity_uses_start_time_and_splits_conflicts() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut index = IdentityIndex::default();
        let first = ProcessFingerprint {
            pid: 7,
            start_ms: TimestampMs(1),
        };
        let second = ProcessFingerprint {
            pid: 7,
            start_ms: TimestampMs(2),
        };
        let task_a = task("session-a")?;
        let task_b = task("session-b")?;
        index.record(first.clone(), task_a.clone());
        assert_eq!(
            index.resolve_process(&first, task_b.clone()),
            ResolvedTarget::Known(task_a)
        );
        assert_eq!(
            index.resolve_process(&second, task_b.clone()),
            ResolvedTarget::New(task_b.clone())
        );
        assert_eq!(
            index.resolve_session_conflict(&first, &task_b),
            ResolvedTarget::SplitRequired
        );
        Ok(())
    }

    #[test]
    fn unsupported_structured_state_source_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let env = PulseHookEnvelope {
            version: PROTOCOL_VERSION,
            provider: provider("Codex")?.0,
            task: task("opaque-source")?.0,
            evidence: EvidenceKind::Activity,
            byte_len: 64,
            forbidden_field_seen: false,
            structured_source_approved: false,
            occurred_at: TimestampMs(1),
        };
        assert_eq!(
            admit(env),
            Err(RejectionCategory::UnsupportedStructuredSource)
        );
        Ok(())
    }
}
