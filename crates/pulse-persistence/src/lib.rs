//! Bounded breadcrumb persistence contracts for Pulse Link.
#![deny(missing_docs)]

use std::io::Write;
use std::path::PathBuf;

use pulse_domain::{
    Attention, BoundedText, FeatureCapability, Lifecycle, ProcessFingerprint, ProviderId,
    ProviderReleaseStatus, RouteCapability, RouteStrength, SafeSummary, TaskHealth, TaskId,
    TaskSnapshot, TimestampMs,
};

/// Maximum active task breadcrumbs retained by W3 Link.
pub const ACTIVE_TASK_LIMIT: usize = 128;
/// Maximum recent terminal task breadcrumbs retained by W3 Link.
pub const RECENT_TERMINAL_LIMIT: usize = 20;
/// Maximum encoded on-disk breadcrumb snapshot size.
pub const MAX_BREADCRUMB_SNAPSHOT_BYTES: usize = 256 * 1024;
/// Maximum estimated encoded size for one task breadcrumb.
pub const MAX_SERIALIZED_TASK_BYTES: usize = 1024;

/// Safe aggregate diagnostic counter. It carries categories, never raw events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticCounter {
    /// Safe bounded category label.
    pub category: BoundedText,
    /// Aggregate count for the category.
    pub count: u64,
}

/// Bounded replacement snapshot persisted by Pulse Link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbSet {
    /// Breadcrumb schema/protocol version.
    pub protocol_version: u16,
    /// Snapshot write timestamp.
    pub written_at: TimestampMs,
    /// Nonterminal active task breadcrumbs.
    pub active_tasks: Vec<TaskSnapshot>,
    /// Bounded terminal task breadcrumbs retained under Minimal profile.
    pub recent_terminal_tasks: Vec<TaskSnapshot>,
    /// Aggregate diagnostics without raw event content.
    pub diagnostic_counters: Vec<DiagnosticCounter>,
}

impl BreadcrumbSet {
    /// Create a bounded breadcrumb snapshot after cap and bucket validation.
    pub fn new(
        protocol_version: u16,
        written_at: TimestampMs,
        active_tasks: Vec<TaskSnapshot>,
        recent_terminal_tasks: Vec<TaskSnapshot>,
        diagnostic_counters: Vec<DiagnosticCounter>,
    ) -> Result<Self, StoreError> {
        if active_tasks.len() > ACTIVE_TASK_LIMIT {
            return Err(StoreError::TooManyActiveTasks);
        }
        if recent_terminal_tasks.len() > RECENT_TERMINAL_LIMIT {
            return Err(StoreError::TooManyRecentTerminalTasks);
        }
        if active_tasks.iter().any(|task| task.lifecycle.is_terminal()) {
            return Err(StoreError::TerminalTaskInActiveBucket);
        }
        if recent_terminal_tasks
            .iter()
            .any(|task| !task.lifecycle.is_terminal())
        {
            return Err(StoreError::NonTerminalTaskInRecentTerminalBucket);
        }
        if active_tasks
            .iter()
            .chain(recent_terminal_tasks.iter())
            .any(|task| estimated_task_bytes(task) > MAX_SERIALIZED_TASK_BYTES)
        {
            return Err(StoreError::TaskTooLarge);
        }

        let set = Self {
            protocol_version,
            written_at,
            active_tasks,
            recent_terminal_tasks,
            diagnostic_counters,
        };
        if set.estimated_snapshot_bytes() > MAX_BREADCRUMB_SNAPSHOT_BYTES {
            return Err(StoreError::SnapshotTooLarge);
        }
        Ok(set)
    }

    /// Conservative encoded-size estimate used before any file backend exists.
    pub fn estimated_snapshot_bytes(&self) -> usize {
        const SNAPSHOT_BASE_BYTES: usize = 32;
        let task_bytes = self
            .active_tasks
            .iter()
            .chain(self.recent_terminal_tasks.iter())
            .map(estimated_task_bytes)
            .sum::<usize>();
        let counter_bytes = self
            .diagnostic_counters
            .iter()
            .map(|counter| counter.category.as_str().len() + 16)
            .sum::<usize>();
        SNAPSHOT_BASE_BYTES + task_bytes + counter_bytes
    }
}

/// Breadcrumb persistence error categories safe for diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// Active task cap exceeded.
    TooManyActiveTasks,
    /// Recent terminal cap exceeded.
    TooManyRecentTerminalTasks,
    /// Terminal task appeared in active bucket.
    TerminalTaskInActiveBucket,
    /// Nonterminal task appeared in recent terminal bucket.
    NonTerminalTaskInRecentTerminalBucket,
    /// One task exceeded the per-task serialized cap.
    TaskTooLarge,
    /// Snapshot exceeded the total serialized cap.
    SnapshotTooLarge,
    /// Filesystem operation failed; details are intentionally not stored in diagnostics.
    Io,
    /// On-disk snapshot was malformed or outside the bounded schema.
    CorruptSnapshot,
}

impl core::fmt::Display for StoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooManyActiveTasks => f.write_str("too many active breadcrumbs"),
            Self::TooManyRecentTerminalTasks => f.write_str("too many recent terminal breadcrumbs"),
            Self::TerminalTaskInActiveBucket => f.write_str("terminal task in active bucket"),
            Self::NonTerminalTaskInRecentTerminalBucket => {
                f.write_str("nonterminal task in recent terminal bucket")
            }
            Self::TaskTooLarge => f.write_str("task breadcrumb exceeds size cap"),
            Self::SnapshotTooLarge => f.write_str("breadcrumb snapshot exceeds size cap"),
            Self::Io => f.write_str("breadcrumb filesystem operation failed"),
            Self::CorruptSnapshot => f.write_str("breadcrumb snapshot is corrupt"),
        }
    }
}

impl std::error::Error for StoreError {}

/// W3 breadcrumb persistence abstraction. Implementations replace whole snapshots atomically.
pub trait BreadcrumbStore {
    /// Load the current bounded breadcrumb snapshot.
    fn load(&self) -> Result<BreadcrumbSet, StoreError>;

    /// Checkpoint a complete replacement snapshot.
    fn checkpoint(&mut self, set: &BreadcrumbSet) -> Result<(), StoreError>;

    /// Clear expired state using a caller-owned clock.
    fn clear_expired(&mut self, now: TimestampMs) -> Result<(), StoreError>;
}

/// In-memory breadcrumb store for tests and fake Link flows.
#[derive(Clone, Debug, Default)]
pub struct MemoryBreadcrumbStore {
    current: Option<BreadcrumbSet>,
}

impl BreadcrumbStore for MemoryBreadcrumbStore {
    fn load(&self) -> Result<BreadcrumbSet, StoreError> {
        match &self.current {
            Some(current) => Ok(current.clone()),
            None => Ok(empty_set()),
        }
    }

    fn checkpoint(&mut self, set: &BreadcrumbSet) -> Result<(), StoreError> {
        if set.estimated_snapshot_bytes() > MAX_BREADCRUMB_SNAPSHOT_BYTES {
            return Err(StoreError::SnapshotTooLarge);
        }
        self.current = Some(set.clone());
        Ok(())
    }

    fn clear_expired(&mut self, now: TimestampMs) -> Result<(), StoreError> {
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.written_at < now)
        {
            self.current = Some(empty_set());
        }
        Ok(())
    }
}

/// File-backed W3 breadcrumb store using bounded complete-replacement snapshots.
#[derive(Clone, Debug)]
pub struct FileBreadcrumbStore {
    path: PathBuf,
}

impl FileBreadcrumbStore {
    /// Create a file-backed store at the exact snapshot path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn temp_path(&self) -> PathBuf {
        match self
            .path
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some(extension) => self.path.with_extension(format!("{extension}.tmp")),
            None => self.path.with_extension("tmp"),
        }
    }
}

impl BreadcrumbStore for FileBreadcrumbStore {
    fn load(&self) -> Result<BreadcrumbSet, StoreError> {
        if !self.path.exists() {
            return Ok(empty_set());
        }
        let encoded = std::fs::read_to_string(&self.path).map_err(|_| StoreError::Io)?;
        decode_snapshot(&encoded)
    }

    fn checkpoint(&mut self, set: &BreadcrumbSet) -> Result<(), StoreError> {
        if set.estimated_snapshot_bytes() > MAX_BREADCRUMB_SNAPSHOT_BYTES {
            return Err(StoreError::SnapshotTooLarge);
        }
        let encoded = encode_snapshot(set)?;
        if encoded.len() > MAX_BREADCRUMB_SNAPSHOT_BYTES {
            return Err(StoreError::SnapshotTooLarge);
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| StoreError::Io)?;
        }
        let temp_path = self.temp_path();
        {
            let mut file = std::fs::File::create(&temp_path).map_err(|_| StoreError::Io)?;
            file.write_all(encoded.as_bytes())
                .map_err(|_| StoreError::Io)?;
            file.sync_all().map_err(|_| StoreError::Io)?;
        }
        std::fs::rename(&temp_path, &self.path).map_err(|_| StoreError::Io)?;
        Ok(())
    }

    fn clear_expired(&mut self, now: TimestampMs) -> Result<(), StoreError> {
        let current = self.load()?;
        if current.written_at < now {
            self.checkpoint(&empty_set())?;
        }
        Ok(())
    }
}

fn empty_set() -> BreadcrumbSet {
    BreadcrumbSet {
        protocol_version: 1,
        written_at: TimestampMs(0),
        active_tasks: Vec::new(),
        recent_terminal_tasks: Vec::new(),
        diagnostic_counters: Vec::new(),
    }
}

fn estimated_task_bytes(task: &TaskSnapshot) -> usize {
    const TASK_BASE_BYTES: usize = 128;
    const FEATURE_BYTES: usize = 8;
    TASK_BASE_BYTES
        + task.provider.0.as_str().len()
        + task.task_id.0.as_str().len()
        + task.features.len() * FEATURE_BYTES
}

fn encode_snapshot(set: &BreadcrumbSet) -> Result<String, StoreError> {
    let mut encoded = format!(
        "pulse-breadcrumb-v1\n{}\t{}\n",
        set.protocol_version, set.written_at.0
    );
    for task in &set.active_tasks {
        encoded.push_str(&encode_task_line('A', task)?);
    }
    for task in &set.recent_terminal_tasks {
        encoded.push_str(&encode_task_line('R', task)?);
    }
    for counter in &set.diagnostic_counters {
        encoded.push_str(&format!(
            "C\t{}\t{}\n",
            safe_field(counter.category.as_str())?,
            counter.count
        ));
    }
    Ok(encoded)
}

fn encode_task_line(bucket: char, task: &TaskSnapshot) -> Result<String, StoreError> {
    Ok(format!(
        "{bucket}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        safe_field(task.task_id.0.as_str())?,
        safe_field(task.provider.0.as_str())?,
        provider_status_code(task.provider_status),
        task_health_code(task.health),
        route_capability_code(task.route_capability),
        feature_codes(&task.features),
        lifecycle_code(task.lifecycle),
        attention_code(task.attention),
        safe_summary_code(task.summary),
        route_strength_code(task.route_strength),
        process_field(&task.process),
        bool_code(task.fuel_blocking),
        bool_code(task.fuel_risk),
        bool_code(task.resource_stall),
        task.updated_at.0
    ))
}

fn decode_snapshot(encoded: &str) -> Result<BreadcrumbSet, StoreError> {
    let mut lines = encoded.lines();
    if lines.next() != Some("pulse-breadcrumb-v1") {
        return Err(StoreError::CorruptSnapshot);
    }
    let meta = lines.next().ok_or(StoreError::CorruptSnapshot)?;
    let mut meta_fields = meta.split('\t');
    let protocol_version = parse_u16(meta_fields.next())?;
    let written_at = TimestampMs(parse_u64(meta_fields.next())?);
    if meta_fields.next().is_some() {
        return Err(StoreError::CorruptSnapshot);
    }

    let mut active_tasks = Vec::new();
    let mut recent_terminal_tasks = Vec::new();
    let mut diagnostic_counters = Vec::new();
    for line in lines {
        let mut fields = line.split('\t');
        match fields.next() {
            Some("A") => active_tasks.push(decode_task(fields)?),
            Some("R") => recent_terminal_tasks.push(decode_task(fields)?),
            Some("C") => diagnostic_counters.push(decode_counter(fields)?),
            _ => return Err(StoreError::CorruptSnapshot),
        }
    }
    BreadcrumbSet::new(
        protocol_version,
        written_at,
        active_tasks,
        recent_terminal_tasks,
        diagnostic_counters,
    )
}

fn decode_task<'a>(mut fields: impl Iterator<Item = &'a str>) -> Result<TaskSnapshot, StoreError> {
    let task_id = TaskId(
        BoundedText::new(next_field(&mut fields)?).map_err(|_| StoreError::CorruptSnapshot)?,
    );
    let provider = ProviderId(
        BoundedText::new(next_field(&mut fields)?).map_err(|_| StoreError::CorruptSnapshot)?,
    );
    let provider_status = provider_status_from_code(parse_u8(fields.next())?)?;
    let health = task_health_from_code(parse_u8(fields.next())?)?;
    let route_capability = route_capability_from_code(parse_u8(fields.next())?)?;
    let features = feature_codes_from_field(next_field(&mut fields)?)?;
    let lifecycle = lifecycle_from_code(parse_u8(fields.next())?)?;
    let attention = attention_from_code(parse_u8(fields.next())?)?;
    let summary = safe_summary_from_code(parse_u8(fields.next())?)?;
    let route_strength = route_strength_from_code(parse_u8(fields.next())?)?;
    let process = process_from_field(next_field(&mut fields)?)?;
    let fuel_blocking = bool_from_code(parse_u8(fields.next())?)?;
    let fuel_risk = bool_from_code(parse_u8(fields.next())?)?;
    let resource_stall = bool_from_code(parse_u8(fields.next())?)?;
    let updated_at = TimestampMs(parse_u64(fields.next())?);
    if fields.next().is_some() {
        return Err(StoreError::CorruptSnapshot);
    }
    Ok(TaskSnapshot {
        task_id,
        provider,
        provider_status,
        health,
        route_capability,
        features,
        lifecycle,
        attention,
        summary,
        route_strength,
        process,
        fuel_blocking,
        fuel_risk,
        resource_stall,
        updated_at,
    })
}

fn decode_counter<'a>(
    mut fields: impl Iterator<Item = &'a str>,
) -> Result<DiagnosticCounter, StoreError> {
    let category =
        BoundedText::new(next_field(&mut fields)?).map_err(|_| StoreError::CorruptSnapshot)?;
    let count = parse_u64(fields.next())?;
    if fields.next().is_some() {
        return Err(StoreError::CorruptSnapshot);
    }
    Ok(DiagnosticCounter { category, count })
}

fn safe_field(value: &str) -> Result<&str, StoreError> {
    if value.contains('\t') || value.contains('\n') || value.contains('\r') {
        Err(StoreError::CorruptSnapshot)
    } else {
        Ok(value)
    }
}

fn next_field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, StoreError> {
    fields.next().ok_or(StoreError::CorruptSnapshot)
}

fn parse_u8(value: Option<&str>) -> Result<u8, StoreError> {
    next_optional(value)?
        .parse()
        .map_err(|_| StoreError::CorruptSnapshot)
}

fn parse_u16(value: Option<&str>) -> Result<u16, StoreError> {
    next_optional(value)?
        .parse()
        .map_err(|_| StoreError::CorruptSnapshot)
}

fn parse_u32(value: Option<&str>) -> Result<u32, StoreError> {
    next_optional(value)?
        .parse()
        .map_err(|_| StoreError::CorruptSnapshot)
}

fn parse_u64(value: Option<&str>) -> Result<u64, StoreError> {
    next_optional(value)?
        .parse()
        .map_err(|_| StoreError::CorruptSnapshot)
}

fn next_optional(value: Option<&str>) -> Result<&str, StoreError> {
    value.ok_or(StoreError::CorruptSnapshot)
}

fn feature_codes(features: &[FeatureCapability]) -> String {
    features
        .iter()
        .map(|feature| feature_code(*feature).to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn feature_codes_from_field(value: &str) -> Result<Vec<FeatureCapability>, StoreError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|code| feature_from_code(code.parse().map_err(|_| StoreError::CorruptSnapshot)?))
        .collect()
}

fn process_field(process: &Option<ProcessFingerprint>) -> String {
    match process {
        Some(process) => format!("{}:{}", process.pid, process.start_ms.0),
        None => "-".to_owned(),
    }
}

fn process_from_field(value: &str) -> Result<Option<ProcessFingerprint>, StoreError> {
    if value == "-" {
        return Ok(None);
    }
    let mut parts = value.split(':');
    let pid = parse_u32(parts.next())?;
    let start_ms = TimestampMs(parse_u64(parts.next())?);
    if parts.next().is_some() {
        return Err(StoreError::CorruptSnapshot);
    }
    Ok(Some(ProcessFingerprint { pid, start_ms }))
}

const fn bool_code(value: bool) -> u8 {
    if value {
        1
    } else {
        0
    }
}

fn bool_from_code(code: u8) -> Result<bool, StoreError> {
    match code {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn provider_status_code(value: ProviderReleaseStatus) -> u8 {
    match value {
        ProviderReleaseStatus::NotProbed => 0,
        ProviderReleaseStatus::ProcessObserved => 1,
        ProviderReleaseStatus::ExperimentalAttached => 2,
        ProviderReleaseStatus::SupportedObserve => 3,
        ProviderReleaseStatus::SupportedFuel => 4,
        ProviderReleaseStatus::SupportedControl => 5,
    }
}

fn provider_status_from_code(code: u8) -> Result<ProviderReleaseStatus, StoreError> {
    match code {
        0 => Ok(ProviderReleaseStatus::NotProbed),
        1 => Ok(ProviderReleaseStatus::ProcessObserved),
        2 => Ok(ProviderReleaseStatus::ExperimentalAttached),
        3 => Ok(ProviderReleaseStatus::SupportedObserve),
        4 => Ok(ProviderReleaseStatus::SupportedFuel),
        5 => Ok(ProviderReleaseStatus::SupportedControl),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn task_health_code(value: TaskHealth) -> u8 {
    match value {
        TaskHealth::Attached => 0,
        TaskHealth::Observed => 1,
        TaskHealth::Degraded => 2,
        TaskHealth::Offline => 3,
    }
}

fn task_health_from_code(code: u8) -> Result<TaskHealth, StoreError> {
    match code {
        0 => Ok(TaskHealth::Attached),
        1 => Ok(TaskHealth::Observed),
        2 => Ok(TaskHealth::Degraded),
        3 => Ok(TaskHealth::Offline),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn route_capability_code(value: RouteCapability) -> u8 {
    match value {
        RouteCapability::None => 0,
        RouteCapability::AgentReady => 1,
        RouteCapability::WorkspaceReady => 2,
        RouteCapability::ContextReady => 3,
    }
}

fn route_capability_from_code(code: u8) -> Result<RouteCapability, StoreError> {
    match code {
        0 => Ok(RouteCapability::None),
        1 => Ok(RouteCapability::AgentReady),
        2 => Ok(RouteCapability::WorkspaceReady),
        3 => Ok(RouteCapability::ContextReady),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn feature_code(value: FeatureCapability) -> u8 {
    match value {
        FeatureCapability::ObserveWaiting => 0,
        FeatureCapability::OpenWorkspace => 1,
        FeatureCapability::OpenExactContext => 2,
        FeatureCapability::ObserveQuotaSnapshot => 3,
        FeatureCapability::ObserveSessionTokens => 4,
    }
}

fn feature_from_code(code: u8) -> Result<FeatureCapability, StoreError> {
    match code {
        0 => Ok(FeatureCapability::ObserveWaiting),
        1 => Ok(FeatureCapability::OpenWorkspace),
        2 => Ok(FeatureCapability::OpenExactContext),
        3 => Ok(FeatureCapability::ObserveQuotaSnapshot),
        4 => Ok(FeatureCapability::ObserveSessionTokens),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn lifecycle_code(value: Lifecycle) -> u8 {
    match value {
        Lifecycle::Unknown => 0,
        Lifecycle::Observed => 1,
        Lifecycle::Running => 2,
        Lifecycle::WaitingUser => 3,
        Lifecycle::Limited => 4,
        Lifecycle::Completed => 5,
        Lifecycle::Failed => 6,
    }
}

fn lifecycle_from_code(code: u8) -> Result<Lifecycle, StoreError> {
    match code {
        0 => Ok(Lifecycle::Unknown),
        1 => Ok(Lifecycle::Observed),
        2 => Ok(Lifecycle::Running),
        3 => Ok(Lifecycle::WaitingUser),
        4 => Ok(Lifecycle::Limited),
        5 => Ok(Lifecycle::Completed),
        6 => Ok(Lifecycle::Failed),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn attention_code(value: Attention) -> u8 {
    match value {
        Attention::None => 0,
        Attention::Active => 1,
        Attention::Waiting => 2,
        Attention::Limited => 3,
        Attention::Failed => 4,
    }
}

fn attention_from_code(code: u8) -> Result<Attention, StoreError> {
    match code {
        0 => Ok(Attention::None),
        1 => Ok(Attention::Active),
        2 => Ok(Attention::Waiting),
        3 => Ok(Attention::Limited),
        4 => Ok(Attention::Failed),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn safe_summary_code(value: SafeSummary) -> u8 {
    match value {
        SafeSummary::Generic => 0,
        SafeSummary::WaitingForUser => 1,
        SafeSummary::Failed => 2,
        SafeSummary::LimitReached => 3,
        SafeSummary::ObservedProcess => 4,
    }
}

fn safe_summary_from_code(code: u8) -> Result<SafeSummary, StoreError> {
    match code {
        0 => Ok(SafeSummary::Generic),
        1 => Ok(SafeSummary::WaitingForUser),
        2 => Ok(SafeSummary::Failed),
        3 => Ok(SafeSummary::LimitReached),
        4 => Ok(SafeSummary::ObservedProcess),
        _ => Err(StoreError::CorruptSnapshot),
    }
}

const fn route_strength_code(value: RouteStrength) -> u8 {
    match value {
        RouteStrength::None => 0,
        RouteStrength::Weak => 1,
        RouteStrength::Useful => 2,
        RouteStrength::Strong => 3,
        RouteStrength::Exact => 4,
    }
}

fn route_strength_from_code(code: u8) -> Result<RouteStrength, StoreError> {
    match code {
        0 => Ok(RouteStrength::None),
        1 => Ok(RouteStrength::Weak),
        2 => Ok(RouteStrength::Useful),
        3 => Ok(RouteStrength::Strong),
        4 => Ok(RouteStrength::Exact),
        _ => Err(StoreError::CorruptSnapshot),
    }
}
