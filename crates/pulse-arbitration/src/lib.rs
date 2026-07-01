//! Deterministic presentation-plan arbitration.
#![deny(missing_docs)]

use pulse_domain::{Lifecycle, TaskSnapshot, TimestampMs};

/// Compact presentation plan consumed by the future UI seam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentationPlan {
    /// Primary task selected by canonical tier order, if any.
    pub primary: Option<TaskSnapshot>,
    /// Secondary tasks capped for Peek presentation.
    pub peek: Vec<TaskSnapshot>,
    /// Deterministic generation time supplied by caller.
    pub generated_at: TimestampMs,
}

fn tier(s: &TaskSnapshot, pinned: bool) -> u8 {
    match s.lifecycle {
        Lifecycle::Failed => 1,
        Lifecycle::WaitingUser => 2,
        Lifecycle::Limited if s.fuel_blocking => 3,
        _ if pinned => 4,
        _ if s.fuel_risk => 5,
        Lifecycle::Running => 7,
        Lifecycle::Completed => 8,
        Lifecycle::Observed | Lifecycle::Unknown | Lifecycle::Limited => 9,
    }
}

/// Select one primary task and up to three Peek tasks using deterministic ordering.
pub fn arbitrate(
    snapshots: &[TaskSnapshot],
    pinned_task: Option<&str>,
    now: TimestampMs,
) -> PresentationPlan {
    let mut ranked: Vec<TaskSnapshot> = snapshots.to_vec();
    ranked.sort_by_key(|s| {
        (
            tier(s, pinned_task == Some(s.task_id.0.as_str())),
            s.updated_at.0,
            s.task_id.0.as_str().to_owned(),
        )
    });
    let primary = ranked.first().cloned();
    let peek = ranked.into_iter().skip(1).take(3).collect();
    PresentationPlan {
        primary,
        peek,
        generated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_domain::{Lifecycle, TaskSnapshot, TimestampMs};
    use pulse_testkit::*;

    fn snap(id: &str, life: Lifecycle) -> Result<TaskSnapshot, Box<dyn std::error::Error>> {
        let mut s = TaskSnapshot::generic(provider("Codex")?, task(id)?, TimestampMs(1));
        s.lifecycle = life;
        if life == Lifecycle::Limited {
            s.fuel_blocking = true;
        }
        Ok(s)
    }

    #[test]
    fn canonical_ordering() -> Result<(), Box<dyn std::error::Error>> {
        let v = vec![
            snap("run", Lifecycle::Running)?,
            snap("wait", Lifecycle::WaitingUser)?,
            snap("lim", Lifecycle::Limited)?,
            snap("fail", Lifecycle::Failed)?,
        ];
        assert_eq!(
            arbitrate(&v, None, TimestampMs(1))
                .primary
                .map(|s| s.task_id.0.as_str().to_owned()),
            Some("fail".to_owned())
        );
        let v = vec![
            snap("run", Lifecycle::Running)?,
            snap("wait", Lifecycle::WaitingUser)?,
            snap("lim", Lifecycle::Limited)?,
        ];
        assert_eq!(
            arbitrate(&v, None, TimestampMs(1))
                .primary
                .map(|s| s.task_id.0.as_str().to_owned()),
            Some("wait".to_owned())
        );
        let v = vec![
            snap("run", Lifecycle::Running)?,
            snap("lim", Lifecycle::Limited)?,
        ];
        assert_eq!(
            arbitrate(&v, Some("run"), TimestampMs(1)).primary.map(|s| s
                .task_id
                .0
                .as_str()
                .to_owned()),
            Some("lim".to_owned())
        );
        Ok(())
    }

    #[test]
    fn fuel_risk_below_waiting_and_peek_capped() -> Result<(), Box<dyn std::error::Error>> {
        let mut risk = snap("risk", Lifecycle::Running)?;
        risk.fuel_risk = true;
        let v = vec![
            risk,
            snap("wait", Lifecycle::WaitingUser)?,
            snap("a", Lifecycle::Running)?,
            snap("b", Lifecycle::Running)?,
            snap("c", Lifecycle::Running)?,
            snap("d", Lifecycle::Running)?,
        ];
        let p = arbitrate(&v, None, TimestampMs(1));
        assert_eq!(
            p.primary.map(|s| s.task_id.0.as_str().to_owned()),
            Some("wait".to_owned())
        );
        assert_eq!(p.peek.len(), 3);
        Ok(())
    }
}
