//! Provider-neutral bounded domain model for Pulse Island.

/// Maximum bytes for safe display labels and categories.
pub const SAFE_TEXT_MAX_BYTES: usize = 64;

/// Bounded text that may be stored or displayed by core crates.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoundedText(String);
impl BoundedText {
    /// Create bounded text after rejecting oversized or content-like values.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if value.len() > SAFE_TEXT_MAX_BYTES {
            return Err(DomainError::TooLong);
        }
        if looks_forbidden(value) {
            return Err(DomainError::ForbiddenContent);
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the safe bounded string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
fn looks_forbidden(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "prompt",
        "transcript",
        "api_key",
        "secret",
        "token=",
        "password",
        "credential",
        "bearer",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// Domain constructor error safe for logs and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainError {
    /// String exceeded the bounded-text byte cap.
    TooLong,
    /// String looked like forbidden task content or secret material.
    ForbiddenContent,
}
impl core::fmt::Display for DomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooLong => f.write_str("bounded text is too long"),
            Self::ForbiddenContent => f.write_str("bounded text contains forbidden content"),
        }
    }
}
impl std::error::Error for DomainError {}

/// Monotonic or wall-clock timestamp expressed as milliseconds at the edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimestampMs(pub u64);

/// Safe bounded provider identifier or display label.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProviderId(pub BoundedText);
/// Opaque bounded task/session key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub BoundedText);

/// Capability-specific provider release status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderReleaseStatus {
    /// Provider has not been probed.
    NotProbed,
    /// Only provider process observation is available.
    ProcessObserved,
    /// Experimental attachment exists but is not supported observe.
    ExperimentalAttached,
    /// Observation support has passed release gates.
    SupportedObserve,
    /// Fuel support has independently passed release gates.
    SupportedFuel,
    /// Control support has independently passed release gates.
    SupportedControl,
}

/// Current reliability of task evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskHealth {
    /// Fresh formal source is attached.
    Attached,
    /// Only observed/passive evidence is available.
    Observed,
    /// Previously attached evidence is stale or recovering.
    Degraded,
    /// No live evidence remains.
    Offline,
}

/// Return-to-context capability for a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteCapability {
    /// No safe route target.
    None,
    /// Agent/provider surface can be focused.
    AgentReady,
    /// Workspace route is available.
    WorkspaceReady,
    /// Exact context route is available.
    ContextReady,
}

/// Independently verified per-task feature capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureCapability {
    /// Waiting state can be observed.
    ObserveWaiting,
    /// Workspace can be opened.
    OpenWorkspace,
    /// Exact original task context can be opened.
    OpenExactContext,
    /// Quota snapshot can be observed.
    ObserveQuotaSnapshot,
    /// Session token source can be observed.
    ObserveSessionTokens,
}

/// Truthful lifecycle state for a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    /// No lifecycle truth is available.
    Unknown,
    /// Process or weak observation only.
    Observed,
    /// Formal running/activity evidence.
    Running,
    /// Provider-verified user decision is needed.
    WaitingUser,
    /// Verified usage/limit block is stopping progress.
    Limited,
    /// Explicit completion terminal state.
    Completed,
    /// Explicit failure terminal state.
    Failed,
}
impl Lifecycle {
    /// Whether this lifecycle is terminal and protected from lower-rank events.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

/// Attention class derived from lifecycle/evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attention {
    /// No attention requested.
    None,
    /// Active/running work.
    Active,
    /// User attention needed.
    Waiting,
    /// Verified limit block.
    Limited,
    /// Failure attention.
    Failed,
}

/// Safe summary category; never raw provider text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeSummary {
    /// Generic provider/workspace label.
    Generic,
    /// Static waiting summary.
    WaitingForUser,
    /// Static failure summary.
    Failed,
    /// Static limit summary.
    LimitReached,
    /// Static process-observed summary.
    ObservedProcess,
}

/// Local privacy profile and retention ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivacyProfile {
    /// Minimal local compact state is allowed.
    Minimal,
    /// Only nonterminal active breadcrumbs may remain.
    Strict,
    /// Passive-only mode creates no integration breadcrumbs.
    PassiveOnly,
}

/// Evidence strength for return-to-context routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStrength {
    /// No route evidence.
    None,
    /// Process identity only.
    Weak,
    /// Workspace/provider surface route.
    Useful,
    /// Related provider/agent window, not exact task.
    Strong,
    /// Exact task/thread/tab route.
    Exact,
}

/// Process fingerprint using PID plus start time, never command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessFingerprint {
    /// Operating-system process identifier.
    pub pid: u32,
    /// Process start timestamp used to avoid PID reuse merges.
    pub start_ms: TimestampMs,
}

/// Compact provider-neutral task snapshot consumed by routing/arbitration/UI seams.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSnapshot {
    /// Opaque task key.
    pub task_id: TaskId,
    /// Provider identifier.
    pub provider: ProviderId,
    /// Provider release status axis.
    pub provider_status: ProviderReleaseStatus,
    /// Per-task health axis.
    pub health: TaskHealth,
    /// Per-task route capability axis.
    pub route_capability: RouteCapability,
    /// Per-task feature capabilities.
    pub features: Vec<FeatureCapability>,
    /// Lifecycle truth.
    pub lifecycle: Lifecycle,
    /// Attention class.
    pub attention: Attention,
    /// Safe summary category.
    pub summary: SafeSummary,
    /// Current route evidence strength.
    pub route_strength: RouteStrength,
    /// Optional process fingerprint.
    pub process: Option<ProcessFingerprint>,
    /// Verified Fuel/usage block flag.
    pub fuel_blocking: bool,
    /// Non-blocking Fuel risk flag.
    pub fuel_risk: bool,
    /// Confirmed local resource condition is causally stalling the task.
    pub resource_stall: bool,
    /// Last update time.
    pub updated_at: TimestampMs,
}
impl TaskSnapshot {
    /// Construct a generic snapshot with no inferred task title or lifecycle truth.
    pub fn generic(provider: ProviderId, task_id: TaskId, now: TimestampMs) -> Self {
        Self {
            task_id,
            provider,
            provider_status: ProviderReleaseStatus::NotProbed,
            health: TaskHealth::Observed,
            route_capability: RouteCapability::None,
            features: Vec::new(),
            lifecycle: Lifecycle::Unknown,
            attention: Attention::None,
            summary: SafeSummary::Generic,
            route_strength: RouteStrength::None,
            process: None,
            fuel_blocking: false,
            fuel_risk: false,
            resource_stall: false,
            updated_at: now,
        }
    }
}
