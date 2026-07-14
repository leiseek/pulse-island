//! Fuel/quota logic separated from task lifecycle.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, ProviderId};

/// Independent Fuel source capability state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuelCapability {
    /// No approved quota/Fuel source is available.
    Unavailable,
    /// An approved source is currently fresh.
    Available,
    /// A previously approved source is stale and must not drive lifecycle.
    Stale,
    /// A previously approved source was revoked and must not drive lifecycle.
    Revoked,
}

/// Compact Fuel state kept separate from task lifecycle truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuelState {
    /// Source capability state.
    pub capability: FuelCapability,
    /// Non-blocking high-confidence risk indicator.
    pub risk: bool,
    /// Verified limit block indicator.
    pub blocking: bool,
}
impl FuelState {
    /// Construct a state with no available Fuel source.
    pub const fn unavailable() -> Self {
        Self {
            capability: FuelCapability::Unavailable,
            risk: false,
            blocking: false,
        }
    }

    /// Mark an approved source available with explicit risk/blocking facts.
    pub const fn available(risk: bool, blocking: bool) -> Self {
        Self {
            capability: FuelCapability::Available,
            risk,
            blocking,
        }
    }

    /// Mark the source stale without inventing a block.
    pub fn mark_stale(&mut self) {
        self.capability = FuelCapability::Stale;
        self.blocking = false;
    }

    /// Revoke the source and clear derived indicators.
    pub fn revoke(&mut self) {
        self.capability = FuelCapability::Revoked;
        self.risk = false;
        self.blocking = false;
    }
}

/// Provider-scoped Fuel bucket. Buckets are never aggregated across providers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuelBucket {
    /// Provider that owns this independent Fuel source.
    pub provider: ProviderId,
    /// Current provider-scoped Fuel state.
    pub state: FuelState,
}

/// Small deterministic Fuel ledger used by W1 fixtures.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FuelLedger {
    buckets: Vec<FuelBucket>,
}

impl FuelLedger {
    /// Upsert one provider-scoped Fuel bucket.
    pub fn upsert(&mut self, provider: ProviderId, state: FuelState) {
        if let Some(bucket) = self
            .buckets
            .iter_mut()
            .find(|bucket| bucket.provider == provider)
        {
            bucket.state = state;
        } else {
            self.buckets.push(FuelBucket { provider, state });
        }
    }

    /// Return a provider-specific bucket without aggregating with any other provider.
    pub fn get(&self, provider: &ProviderId) -> Option<&FuelState> {
        self.buckets
            .iter()
            .find(|bucket| &bucket.provider == provider)
            .map(|bucket| &bucket.state)
    }

    /// Number of independent provider buckets.
    pub fn len(&self) -> usize {
        self.buckets.len()
    }

    /// Whether the ledger has no provider buckets.
    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

/// Task-scoped token sample from an approved numeric source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSample {
    /// Provider that owns the token source.
    pub provider: ProviderId,
    /// Opaque task/session reference.
    pub task: BoundedText,
    /// Source sample identifier used for duplicate suppression.
    pub sample_id: BoundedText,
    /// Monotonic token total as reported by the source for this task window.
    pub total_tokens: u64,
}

/// Result of applying a token sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenDelta {
    /// Non-negative token delta accepted from the sample.
    pub delta_tokens: u64,
    /// Whether the source counter appeared to reset.
    pub counter_reset: bool,
    /// Whether this sample id was already applied.
    pub duplicate: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenEntry {
    provider: ProviderId,
    task: BoundedText,
    last_total: u64,
    seen_sample_ids: Vec<BoundedText>,
}

/// Deterministic token ledger that never invents negative or duplicate burn.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenLedger {
    entries: Vec<TokenEntry>,
}

impl TokenLedger {
    /// Apply a task-scoped token sample and return a non-negative delta.
    pub fn apply_sample(&mut self, sample: TokenSample) -> TokenDelta {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.provider == sample.provider && entry.task == sample.task)
        {
            if entry.seen_sample_ids.contains(&sample.sample_id) {
                return TokenDelta {
                    delta_tokens: 0,
                    counter_reset: false,
                    duplicate: true,
                };
            }
            entry.seen_sample_ids.push(sample.sample_id);
            if sample.total_tokens < entry.last_total {
                entry.last_total = sample.total_tokens;
                return TokenDelta {
                    delta_tokens: 0,
                    counter_reset: true,
                    duplicate: false,
                };
            }
            let delta_tokens = sample.total_tokens - entry.last_total;
            entry.last_total = sample.total_tokens;
            TokenDelta {
                delta_tokens,
                counter_reset: false,
                duplicate: false,
            }
        } else {
            self.entries.push(TokenEntry {
                provider: sample.provider,
                task: sample.task,
                last_total: sample.total_tokens,
                seen_sample_ids: vec![sample.sample_id],
            });
            TokenDelta {
                delta_tokens: 0,
                counter_reset: false,
                duplicate: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_domain::{BoundedText, DomainError};

    fn provider(name: &str) -> Result<ProviderId, DomainError> {
        BoundedText::new(name).map(ProviderId)
    }

    #[test]
    fn stale_and_revoked_clear_block_without_lifecycle_coupling() {
        let unavailable = FuelState::unavailable();
        assert_eq!(unavailable.capability, FuelCapability::Unavailable);
        assert!(!unavailable.risk);
        assert!(!unavailable.blocking);

        let mut state = FuelState::available(true, true);
        state.mark_stale();
        assert_eq!(state.capability, FuelCapability::Stale);
        assert!(!state.blocking);
        state.revoke();
        assert_eq!(state.capability, FuelCapability::Revoked);
        assert!(!state.risk);
        assert!(!state.blocking);
    }

    #[test]
    fn provider_buckets_do_not_aggregate() -> Result<(), Box<dyn std::error::Error>> {
        let codex = provider("Codex")?;
        let claude = provider("Claude")?;
        let mut ledger = FuelLedger::default();
        ledger.upsert(codex.clone(), FuelState::available(true, false));
        ledger.upsert(claude.clone(), FuelState::available(false, true));
        assert_eq!(ledger.len(), 2);
        assert_eq!(ledger.get(&codex).map(|state| state.risk), Some(true));
        assert_eq!(ledger.get(&codex).map(|state| state.blocking), Some(false));
        assert_eq!(ledger.get(&claude).map(|state| state.risk), Some(false));
        assert_eq!(ledger.get(&claude).map(|state| state.blocking), Some(true));
        Ok(())
    }

    #[test]
    fn token_samples_never_create_negative_or_duplicate_burn(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let codex = provider("Codex")?;
        let task = BoundedText::new("task-a")?;
        let mut ledger = TokenLedger::default();

        let first = ledger.apply_sample(TokenSample {
            provider: codex.clone(),
            task: task.clone(),
            sample_id: BoundedText::new("sample-1")?,
            total_tokens: 100,
        });
        assert_eq!(first.delta_tokens, 0);

        let second = ledger.apply_sample(TokenSample {
            provider: codex.clone(),
            task: task.clone(),
            sample_id: BoundedText::new("sample-2")?,
            total_tokens: 125,
        });
        assert_eq!(second.delta_tokens, 25);

        let duplicate = ledger.apply_sample(TokenSample {
            provider: codex.clone(),
            task: task.clone(),
            sample_id: BoundedText::new("sample-2")?,
            total_tokens: 125,
        });
        assert_eq!(duplicate.delta_tokens, 0);

        let reset = ledger.apply_sample(TokenSample {
            provider: codex,
            task,
            sample_id: BoundedText::new("sample-3")?,
            total_tokens: 10,
        });
        assert_eq!(reset.delta_tokens, 0);
        assert!(reset.counter_reset);

        Ok(())
    }
}
