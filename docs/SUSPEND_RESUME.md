# Workflow Suspend & Resume

The Generic AI Agent framework provides comprehensive suspend and resume functionality for workflows, allowing them to be paused at any step and resumed later. This is essential for long-running workflows that may need to wait for external resources, human input, or handle rate limiting scenarios.

## Features

- **Persistent Snapshots**: Complete workflow state is serialized and stored persistently
- **Flexible Storage**: File-based and database storage options for snapshots
- **Automatic Checkpointing**: Configurable automatic checkpoint creation
- **Step-Specific State**: Custom state capture and restoration for individual workflow steps
- **Suspend Reasons**: Rich suspend reason tracking for better observability
- **Cleanup Management**: Automatic cleanup of old snapshots based on retention policies

## Common Use Cases

### 1. Human Approval Workflows

```rust
use the_agency::workflow::{HumanApprovalStep, WorkflowEngine};

let engine = WorkflowEngine::new()
    .with_snapshot_storage(Box::new(file_storage))
    .add_step(Box::new(HumanApprovalStep::new(
        "Please review and approve this action".to_string()
    )));

let result = engine.execute(context).await?;
// Workflow suspends waiting for human input

// Later, resume with approval
let mut approved_context = snapshot.context.clone();
approved_context.metadata.insert("human_approval".to_string(), "granted".to_string());
let resumed = engine.resume_from_snapshot(snapshot_id).await?;
```

### 2. Rate-Limited API Calls

```rust
use the_agency::workflow::RateLimitedApiStep;

let engine = WorkflowEngine::new()
    .add_step(Box::new(RateLimitedApiStep::new(
        "external_api".to_string(),
        10 // 10 calls per minute
    )));

// Workflow automatically suspends when rate limit is hit
// Resume after waiting period
```

### 3. External Resource Availability

```rust
use the_agency::workflow::EnhancedMemoryRetrievalStep;

let mut context = WorkflowContext::new(10);
// Mark external resource as unavailable
context.metadata.insert("external_memory_unavailable".to_string(), "true".to_string());

// Workflow will suspend until resource becomes available
```

## Configuration

### Basic Configuration

```toml
[workflow]
enable_suspend_resume = true
snapshot_storage_dir = "./snapshots"
auto_checkpoint = true
checkpoint_interval = 5
max_snapshots = 10
snapshot_retention_days = 7
debug_steps = false
```

### Programmatic Configuration

```rust
use the_agency::workflow::{WorkflowSuspendConfig, FileSnapshotStorage};

let suspend_config = WorkflowSuspendConfig {
    auto_checkpoint: true,
    checkpoint_interval: 3,
    max_snapshots: 15,
    snapshot_retention: chrono::Duration::days(14),
};

let storage = FileSnapshotStorage::new("./workflow_snapshots");

let engine = WorkflowEngine::new()
    .with_suspend_config(suspend_config)
    .with_snapshot_storage(Box::new(storage))
    .with_default_steps();
```

## Snapshot Structure

### WorkflowSnapshot

The `WorkflowSnapshot` structure captures the complete state of a workflow:

```rust
pub struct WorkflowSnapshot {
    pub id: Uuid,                           // Unique snapshot ID
    pub created_at: DateTime<Utc>,          // When snapshot was created
    pub context: WorkflowContext,           // Complete workflow context
    pub current_step: usize,                // Current step index
    pub suspend_reason: SuspendReason,      // Why it was suspended
    pub metadata: HashMap<String, String>,  // Additional metadata
    pub step_state: HashMap<String, serde_json::Value>, // Step-specific state
}
```

### Suspend Reasons

```rust
pub enum SuspendReason {
    Manual,                                 // Manual suspension
    WaitingForInput(String),               // Waiting for external input
    WaitingForResource(String),            // Waiting for resource availability
    RateLimit,                             // Rate limiting
    Scheduled,                             // Scheduled checkpoint
    Error(String),                         // Error occurred
}
```

## Storage Options

### File-Based Storage

```rust
use the_agency::workflow::FileSnapshotStorage;

let storage = FileSnapshotStorage::new("./snapshots");

// Storage automatically creates directory structure
// Snapshots are stored as JSON files named by UUID
```

### Custom Storage Implementation

Implement the `SnapshotStorage` trait for custom storage backends:

```rust
use the_agency::workflow::SnapshotStorage;
use async_trait::async_trait;

pub struct DatabaseSnapshotStorage {
    // Your database connection
}

#[async_trait]
impl SnapshotStorage for DatabaseSnapshotStorage {
    async fn store_snapshot(&self, snapshot: &WorkflowSnapshot) -> Result<()> {
        // Store snapshot in database
        todo!()
    }
    
    async fn get_snapshot(&self, id: Uuid) -> Result<Option<WorkflowSnapshot>> {
        // Retrieve snapshot from database
        todo!()
    }
    
    // Implement other required methods...
}
```

## Creating Suspendable Steps

### Implementing SuspendableWorkflowStep

```rust
use the_agency::workflow::{WorkflowStep, SuspendableWorkflowStep};

pub struct CustomSuspendableStep;

#[async_trait]
impl WorkflowStep for CustomSuspendableStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        // Check if suspension conditions are met
        if should_suspend(context) {
            return Ok(WorkflowDecision::Suspend(SuspendReason::WaitingForResource(
                "External service unavailable".to_string()
            )));
        }
        
        // Normal execution logic
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        "custom_suspendable_step"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for CustomSuspendableStep {
    async fn can_suspend(&self, context: &WorkflowContext) -> bool {
        // Define when this step can be suspended
        true
    }
    
    async fn capture_state(&self, context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        // Capture step-specific state for restoration
        Ok(Some(serde_json::json!({
            "processing_id": "some_id",
            "progress": 50
        })))
    }
    
    async fn restore_state(&self, context: &mut WorkflowContext, state: Option<&serde_json::Value>) -> Result<()> {
        // Restore step-specific state after resumption
        if let Some(state) = state {
            // Restore state from JSON value
        }
        Ok(())
    }
    
    fn suspend_points(&self) -> Vec<String> {
        vec![
            "resource_unavailable".to_string(),
            "rate_limit_exceeded".to_string(),
        ]
    }
}
```

## Advanced Usage

### Manual Suspension

```rust
// Manually suspend a workflow
let snapshot_id = engine.suspend(
    &context, 
    current_step, 
    SuspendReason::Manual
).await?;

println!("Workflow suspended with ID: {}", snapshot_id);
```

### Snapshot Management

```rust
// List all snapshots
let snapshots = engine.list_snapshots(None).await?;

// Filter snapshots by metadata
let mut filter = HashMap::new();
filter.insert("user_id".to_string(), "123".to_string());
let user_snapshots = engine.list_snapshots(Some(filter)).await?;

// Delete specific snapshot
let deleted = engine.delete_snapshot(snapshot_id).await?;

// Clean up old snapshots
let cleaned_count = engine.cleanup_snapshots().await?;
```

### Checkpoint Strategies

```rust
// Automatic checkpoints every 3 steps
let config = WorkflowSuspendConfig {
    auto_checkpoint: true,
    checkpoint_interval: 3,
    max_snapshots: 10,
    snapshot_retention: chrono::Duration::days(7),
};

// Manual checkpoints at critical points
if at_critical_point {
    let checkpoint_id = engine.suspend(
        &context,
        current_step,
        SuspendReason::Scheduled
    ).await?;
}
```

## Error Handling

### Resume Errors

```rust
match engine.resume_from_snapshot(snapshot_id).await {
    Ok(result) => {
        if result.completed {
            println!("Workflow completed: {}", result.response);
        } else {
            println!("Workflow suspended again: {}", result.response);
        }
    }
    Err(e) => {
        eprintln!("Failed to resume workflow: {}", e);
        // Handle resume failure - snapshot may be corrupted or missing
    }
}
```

### Storage Errors

```rust
// Check if snapshot exists before resuming
if let Ok(Some(_)) = engine.get_snapshot(snapshot_id).await {
    let result = engine.resume_from_snapshot(snapshot_id).await?;
} else {
    eprintln!("Snapshot not found: {}", snapshot_id);
}
```

## Best Practices

### 1. Snapshot Naming and Metadata

```rust
let mut snapshot = engine.create_snapshot(&context, step, reason).await?;
snapshot.metadata.insert("user_id".to_string(), user_id);
snapshot.metadata.insert("workflow_type".to_string(), "approval".to_string());
snapshot.metadata.insert("priority".to_string(), "high".to_string());
```

### 2. Graceful Degradation

```rust
// Always handle missing snapshot storage gracefully
let engine = WorkflowEngine::new()
    .with_suspend_config(config);

// Optionally add snapshot storage if available
if let Ok(storage) = create_snapshot_storage() {
    engine = engine.with_snapshot_storage(Box::new(storage));
}
```

### 3. State Validation

```rust
#[async_trait]
impl SuspendableWorkflowStep for MyStep {
    async fn restore_state(&self, context: &mut WorkflowContext, state: Option<&serde_json::Value>) -> Result<()> {
        if let Some(state) = state {
            // Validate state before restoring
            if let Some(id) = state.get("processing_id").and_then(|v| v.as_str()) {
                if !is_valid_processing_id(id) {
                    return Err(AgentError::Workflow("Invalid processing ID in snapshot".to_string()));
                }
            }
        }
        Ok(())
    }
}
```

### 4. Monitoring and Observability

```rust
// Log snapshot creation for monitoring
let snapshot_id = engine.suspend(&context, step, reason).await?;
tracing::info!(
    snapshot_id = %snapshot_id,
    step = step,
    reason = ?reason,
    "Workflow suspended"
);

// Monitor snapshot storage health
if let Ok(snapshots) = engine.list_snapshots(None).await {
    tracing::info!(count = snapshots.len(), "Active snapshots");
}
```

## Testing

The framework includes comprehensive tests for suspend/resume functionality:

```bash
# Run all workflow tests
cargo test workflow::tests

# Run suspend/resume specific tests
cargo test test_workflow_snapshot_serialization
cargo test test_file_snapshot_storage
cargo test test_workflow_engine_with_suspend_resume

# Run the example
cargo run --example suspend_resume_example
```

## Performance Considerations

- **Snapshot Size**: Large contexts with many messages/memories will create large snapshots
- **Storage I/O**: File-based storage performs one I/O operation per snapshot operation
- **Memory Usage**: Snapshots are held in memory during serialization/deserialization
- **Cleanup Frequency**: Configure appropriate retention policies to avoid storage bloat

## Troubleshooting

### Common Issues

1. **Snapshot Not Found**: Verify snapshot ID and storage configuration
2. **Deserialization Errors**: Ensure snapshot format compatibility across versions
3. **Storage Permissions**: Verify write permissions for snapshot storage directory
4. **Large Snapshots**: Consider reducing context size or implementing compression

### Debug Mode

```rust
let config = WorkflowConfig {
    debug_steps: true,
    // ... other config
};
```

This enables detailed logging of workflow execution and suspend/resume operations.
