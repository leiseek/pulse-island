//! W3 bounded breadcrumb persistence contract tests.

use pulse_domain::{
    BoundedText, Lifecycle, ProviderId, ProviderReleaseStatus, RouteCapability, RouteStrength,
    SafeSummary, TaskHealth, TaskId, TaskSnapshot, TimestampMs,
};
use pulse_persistence::{
    BreadcrumbSet, BreadcrumbStore, DiagnosticCounter, FileBreadcrumbStore, StoreError,
    ACTIVE_TASK_LIMIT, MAX_BREADCRUMB_SNAPSHOT_BYTES, MAX_SERIALIZED_TASK_BYTES,
    RECENT_TERMINAL_LIMIT,
};
use std::path::PathBuf;

#[test]
fn breadcrumb_set_enforces_active_and_recent_terminal_caps(
) -> Result<(), Box<dyn std::error::Error>> {
    let active_tasks = make_tasks(ACTIVE_TASK_LIMIT + 1, Lifecycle::Running)?;

    assert_eq!(
        BreadcrumbSet::new(1, TimestampMs(10), active_tasks, Vec::new(), Vec::new()),
        Err(StoreError::TooManyActiveTasks)
    );

    let recent_terminal_tasks = make_tasks(RECENT_TERMINAL_LIMIT + 1, Lifecycle::Completed)?;

    assert_eq!(
        BreadcrumbSet::new(
            1,
            TimestampMs(10),
            Vec::new(),
            recent_terminal_tasks,
            Vec::new()
        ),
        Err(StoreError::TooManyRecentTerminalTasks)
    );

    Ok(())
}

#[test]
fn breadcrumb_set_rejects_wrong_lifecycle_bucket() -> Result<(), Box<dyn std::error::Error>> {
    let terminal_as_active = make_task("terminal-active", Lifecycle::Completed)?;
    assert_eq!(
        BreadcrumbSet::new(
            1,
            TimestampMs(10),
            vec![terminal_as_active],
            Vec::new(),
            Vec::new()
        ),
        Err(StoreError::TerminalTaskInActiveBucket)
    );

    let active_as_recent_terminal = make_task("active-terminal", Lifecycle::Running)?;
    assert_eq!(
        BreadcrumbSet::new(
            1,
            TimestampMs(10),
            Vec::new(),
            vec![active_as_recent_terminal],
            Vec::new()
        ),
        Err(StoreError::NonTerminalTaskInRecentTerminalBucket)
    );

    Ok(())
}

#[test]
fn breadcrumb_set_rejects_per_task_and_snapshot_size_overflow(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut huge_task = make_task("huge-task", Lifecycle::Running)?;
    huge_task.features =
        vec![pulse_domain::FeatureCapability::ObserveWaiting; (MAX_SERIALIZED_TASK_BYTES / 8) + 1];

    assert_eq!(
        BreadcrumbSet::new(1, TimestampMs(10), vec![huge_task], Vec::new(), Vec::new()),
        Err(StoreError::TaskTooLarge)
    );

    let counters = vec![
        DiagnosticCounter {
            category: BoundedText::new("overflow")?,
            count: u64::MAX,
        };
        MAX_BREADCRUMB_SNAPSHOT_BYTES
    ];

    assert_eq!(
        BreadcrumbSet::new(1, TimestampMs(10), Vec::new(), Vec::new(), counters),
        Err(StoreError::SnapshotTooLarge)
    );

    Ok(())
}

#[test]
fn memory_store_checkpoints_complete_replacement_snapshot() -> Result<(), Box<dyn std::error::Error>>
{
    let first = BreadcrumbSet::new(
        1,
        TimestampMs(10),
        vec![make_task("first", Lifecycle::Running)?],
        Vec::new(),
        Vec::new(),
    )?;
    let second = BreadcrumbSet::new(
        1,
        TimestampMs(20),
        Vec::new(),
        vec![make_task("second", Lifecycle::Completed)?],
        Vec::new(),
    )?;
    let mut store = pulse_persistence::MemoryBreadcrumbStore::default();

    store.checkpoint(&first)?;
    store.checkpoint(&second)?;

    assert_eq!(store.load()?, second);
    Ok(())
}

#[test]
fn file_store_loads_empty_set_when_snapshot_file_is_missing(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("missing")?;
    let path = dir.join("breadcrumbs.snapshot");
    let store = FileBreadcrumbStore::new(path);

    assert_eq!(store.load()?, empty_set());

    remove_dir_if_exists(dir)?;
    Ok(())
}

#[test]
fn file_store_checkpoints_complete_replacement_snapshot_without_append(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("replace")?;
    let path = dir.join("breadcrumbs.snapshot");
    let first = BreadcrumbSet::new(
        1,
        TimestampMs(10),
        vec![make_task("first", Lifecycle::Running)?],
        Vec::new(),
        Vec::new(),
    )?;
    let second = BreadcrumbSet::new(
        1,
        TimestampMs(20),
        Vec::new(),
        vec![make_task("second", Lifecycle::Completed)?],
        vec![DiagnosticCounter {
            category: BoundedText::new("ingress_rejected")?,
            count: 2,
        }],
    )?;
    let mut store = FileBreadcrumbStore::new(path.clone());

    store.checkpoint(&first)?;
    store.checkpoint(&second)?;

    assert_eq!(store.load()?, second);
    let encoded = std::fs::read_to_string(path)?;
    assert!(!encoded.contains("first"));
    assert!(encoded.contains("second"));
    assert!(!dir.join("breadcrumbs.snapshot.tmp").exists());

    remove_dir_if_exists(dir)?;
    Ok(())
}

#[test]
fn file_store_rejects_oversized_checkpoint_without_replacing_previous_snapshot(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("oversized")?;
    let path = dir.join("breadcrumbs.snapshot");
    let first = BreadcrumbSet::new(
        1,
        TimestampMs(10),
        vec![make_task("first", Lifecycle::Running)?],
        Vec::new(),
        Vec::new(),
    )?;
    let oversized = BreadcrumbSet {
        protocol_version: 1,
        written_at: TimestampMs(20),
        active_tasks: Vec::new(),
        recent_terminal_tasks: Vec::new(),
        diagnostic_counters: vec![
            DiagnosticCounter {
                category: BoundedText::new("overflow")?,
                count: 1,
            };
            MAX_BREADCRUMB_SNAPSHOT_BYTES
        ],
    };
    let mut store = FileBreadcrumbStore::new(path.clone());

    store.checkpoint(&first)?;
    assert_eq!(
        store.checkpoint(&oversized),
        Err(StoreError::SnapshotTooLarge)
    );

    assert_eq!(store.load()?, first);
    let encoded = std::fs::read_to_string(path)?;
    assert!(encoded.contains("first"));
    assert!(!encoded.contains("overflow"));

    remove_dir_if_exists(dir)?;
    Ok(())
}

fn make_tasks(
    count: usize,
    lifecycle: Lifecycle,
) -> Result<Vec<TaskSnapshot>, Box<dyn std::error::Error>> {
    let mut tasks = Vec::with_capacity(count);
    for index in 0..count {
        tasks.push(make_task(&format!("task-{index}"), lifecycle)?);
    }
    Ok(tasks)
}

fn make_task(id: &str, lifecycle: Lifecycle) -> Result<TaskSnapshot, Box<dyn std::error::Error>> {
    Ok(TaskSnapshot {
        task_id: TaskId(BoundedText::new(id)?),
        provider: ProviderId(BoundedText::new("synthetic")?),
        provider_status: ProviderReleaseStatus::NotProbed,
        health: TaskHealth::Observed,
        route_capability: RouteCapability::None,
        features: Vec::new(),
        lifecycle,
        attention: pulse_domain::Attention::None,
        summary: SafeSummary::Generic,
        route_strength: RouteStrength::None,
        process: None,
        fuel_blocking: false,
        fuel_risk: false,
        resource_stall: false,
        updated_at: TimestampMs(1),
    })
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

fn unique_test_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "pulse-persistence-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn remove_dir_if_exists(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
