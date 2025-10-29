# Pause Execution

The Generic AI Agent framework provides comprehensive pause execution functionality that allows workflows to pause for specific durations, until timestamps, or for external events. This extends the suspend/resume system with fine-grained execution control.

## Overview

The pause execution system provides four main capabilities:
- **`sleep(duration_ms)`**: Pause for a set number of milliseconds
- **`sleepUntil(timestamp)`**: Pause until a specific timestamp  
- **`waitForEvent(event_id, timeout_ms)`**: Pause until an external event is received
- **`sendEvent(event)`**: Send an event to resume waiting workflows

## Core Components

### 1. Extended Suspend Reasons

The `SuspendReason` enum now includes pause-specific variants:

```rust
pub enum SuspendReason {
    // ... existing reasons ...
    
    /// Sleep for specific duration
    Sleep { 
        duration_ms: u64,
        started_at: DateTime<Utc>
    },
    
    /// Sleep until specific timestamp
    SleepUntil(DateTime<Utc>),
    
    /// Waiting for specific event
    WaitingForEvent {
        event_id: String,
        timeout_ms: Option<u64>,
        started_at: DateTime<Utc>
    },
}
```

### 2. EventBus System

The `EventBus` provides inter-workflow and external communication:

```rust
pub struct EventBus {
    // Event broadcasters by event type
    broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<WorkflowEvent>>>>,
    channel_capacity: usize,
}
```

#### Key Methods:
- `send_event(event)` - Send event to all subscribers
- `subscribe(event_type)` - Subscribe to specific event type
- `wait_for_event(event_id, timeout_ms)` - Wait for specific event with optional timeout

### 3. WorkflowEvent Structure

Events carry structured data between workflows:

```rust
pub struct WorkflowEvent {
    pub id: String,                    // Unique identifier
    pub event_type: String,            // Event type/category
    pub payload: serde_json::Value,    // Event data
    pub timestamp: DateTime<Utc>,      // When created
    pub target_workflow_id: Option<String>, // Optional target
}
```

## Usage Examples

### Basic Sleep Functionality

```rust
// Direct sleep using WorkflowEngine
let engine = WorkflowEngine::new();

// Sleep for 500 milliseconds
engine.sleep(500).await?;

// Sleep until specific timestamp
let future_time = Utc::now() + Duration::seconds(30);
engine.sleep_until(future_time).await?;
```

### Workflow Steps with Pause

```rust
use the-agency::workflow::{SleepStep, SleepUntilStep, WaitForEventStep};

// Create workflow with pause steps
let engine = WorkflowEngine::new()
    .add_step(Box::new(SleepStep::new(1000)))  // Sleep 1 second
    .add_step(Box::new(SleepUntilStep::new(wake_time)))
    .add_step(Box::new(WaitForEventStep::new("user_input".to_string(), Some(30000))));
```

### Event-Driven Workflows

```rust
// Set up event bus
let event_bus = Arc::new(EventBus::new(100));

let engine = WorkflowEngine::new()
    .with_event_bus(Arc::clone(&event_bus))
    .add_step(Box::new(WaitForEventStep::new("approval".to_string(), Some(60000))));

// Send approval event
let approval_event = WorkflowEvent {
    id: "approval-123".to_string(),
    event_type: "approval".to_string(),
    payload: serde_json::json!({"approved": true, "user": "admin"}),
    timestamp: Utc::now(),
    target_workflow_id: None,
};

engine.send_event(approval_event)?;
```

### Conditional Pausing

```rust
use the-agency::workflow::{ConditionalPauseStep, PauseType};

// Pause only if specific condition is met
let conditional_step = ConditionalPauseStep::new(
    "needs_approval".to_string(),
    PauseType::WaitForEvent {
        event_id: "manager_approval".to_string(),
        timeout_ms: Some(300000), // 5 minutes
    }
);

let engine = WorkflowEngine::new()
    .add_step(Box::new(conditional_step));

// Set condition in workflow context
context.metadata.insert("needs_approval".to_string(), "true".to_string());
```

## Pause Types and Workflow Steps

### 1. SleepStep
Unconditionally pauses for a specified duration:

```rust
let sleep_step = SleepStep::new(5000); // 5 seconds
```

### 2. SleepUntilStep  
Pauses until a specific timestamp:

```rust
let wake_time = Utc::now() + Duration::hours(1);
let sleep_until_step = SleepUntilStep::new(wake_time);
```

### 3. WaitForEventStep
Pauses until an event is received:

```rust
let event_step = WaitForEventStep::new(
    "data_ready".to_string(),
    Some(10000) // 10 second timeout
);
```

### 4. ConditionalPauseStep
Conditionally pauses based on workflow context:

```rust
let conditional_step = ConditionalPauseStep::new(
    "should_pause".to_string(),
    PauseType::Sleep(2000)
);
```

## Advanced Event Patterns

### Request-Response Pattern

```rust
// Set up response listener
let mut response_receiver = event_bus.subscribe("response_123");

// Send request
let request = WorkflowEvent {
    id: "request_123".to_string(),
    event_type: "api_request".to_string(),
    payload: serde_json::json!({"endpoint": "/data", "method": "GET"}),
    timestamp: Utc::now(),
    target_workflow_id: None,
};
event_bus.send_event(request)?;

// Wait for response
if let Some(response) = event_bus.wait_for_event("response_123", Some(5000)).await? {
    println!("Received response: {:?}", response.payload);
}
```

### Workflow Coordination

```rust
// Coordinate multiple workflow steps
let coordination_events = ["step_1_complete", "step_2_complete", "step_3_complete"];

for event_type in coordination_events {
    let event = WorkflowEvent {
        id: format!("coord_{}", event_type),
        event_type: event_type.to_string(),
        payload: serde_json::json!({"workflow_id": "main", "step": event_type}),
        timestamp: Utc::now(),
        target_workflow_id: Some("main".to_string()),
    };
    event_bus.send_event(event)?;
}
```

### Broadcast to Multiple Subscribers

```rust
// Set up multiple subscribers
let mut receiver1 = event_bus.subscribe("broadcast");
let mut receiver2 = event_bus.subscribe("broadcast");

// Send broadcast event
let broadcast = WorkflowEvent {
    id: "broadcast_001".to_string(),
    event_type: "broadcast".to_string(),
    payload: serde_json::json!({"message": "Hello all!"}),
    timestamp: Utc::now(),
    target_workflow_id: None,
};

let sent_count = event_bus.send_event(broadcast)?;
println!("Sent to {} subscribers", sent_count);

// Both receivers will get the event
let event1 = receiver1.recv().await?;
let event2 = receiver2.recv().await?;
```

## Integration with Suspend/Resume

All pause functionality integrates seamlessly with the existing suspend/resume system:

### Workflow Snapshots

When a workflow pauses, it creates a snapshot with pause-specific information:

```rust
// Snapshot includes pause reason and timing
let snapshot = WorkflowSnapshot {
    suspend_reason: SuspendReason::Sleep {
        duration_ms: 5000,
        started_at: Utc::now(),
    },
    // ... other snapshot data
};
```

### Resume After Pause

Paused workflows can be resumed manually or automatically:

```rust
// Manual resume
let snapshots = engine.list_snapshots(None).await?;
if let Some(snapshot) = snapshots.first() {
    let result = engine.resume_from_snapshot(snapshot.id).await?;
}

// Event-triggered resume
engine.send_simple_event(
    "resume_signal",
    "workflow_control", 
    serde_json::json!({"action": "resume"})
)?;
```

## Configuration

### EventBus Configuration

```rust
// Create event bus with custom capacity
let event_bus = EventBus::new(500); // 500 events per channel

// Check subscriber counts
let count = event_bus.subscriber_count("my_event_type");
```

### Workflow Engine Configuration

```rust
let engine = WorkflowEngine::new()
    .with_event_bus(Arc::new(EventBus::new(100)))
    .with_suspend_config(WorkflowSuspendConfig {
        auto_checkpoint: true,
        checkpoint_interval: 5,
        max_snapshots: 20,
        snapshot_retention: chrono::Duration::hours(2),
    });
```

## Error Handling

### Timeout Handling

```rust
// Event wait with timeout
match event_bus.wait_for_event("user_input", Some(30000)).await? {
    Some(event) => {
        println!("Received event: {}", event.id);
        // Process event
    }
    None => {
        println!("Timeout waiting for user input");
        // Handle timeout
    }
}
```

### Event Send Failures

```rust
// Check if event was delivered
let sent_count = match event_bus.send_event(my_event) {
    Ok(count) => count,
    Err(e) => {
        eprintln!("Failed to send event: {}", e);
        0
    }
};

if sent_count == 0 {
    println!("No subscribers received the event");
}
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_sleep_functionality() {
    let engine = WorkflowEngine::new();
    
    // Test basic sleep
    let start = std::time::Instant::now();
    engine.sleep(100).await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 95);
}

#[tokio::test]
async fn test_event_system() {
    let event_bus = EventBus::new(10);
    
    // Test event sending and receiving
    let mut receiver = event_bus.subscribe("test");
    
    let event = WorkflowEvent {
        id: "test_event".to_string(),
        event_type: "test".to_string(),
        payload: serde_json::json!({"data": "test"}),
        timestamp: Utc::now(),
        target_workflow_id: None,
    };
    
    event_bus.send_event(event.clone()).unwrap();
    let received = receiver.recv().await.unwrap();
    assert_eq!(received.id, event.id);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_workflow_with_pause() {
    let engine = WorkflowEngine::new()
        .add_step(Box::new(SleepStep::new(10)));
    
    let context = WorkflowContext::new(10);
    let result = engine.execute(context).await.unwrap();
    
    // Should suspend for sleep
    assert!(!result.completed);
    assert!(result.response.contains("Sleeping"));
    
    // Verify snapshot
    let snapshots = engine.list_snapshots(None).await.unwrap();
    assert!(!snapshots.is_empty());
}
```

## Performance Considerations

### Event Bus Scaling

- **Channel Capacity**: Configure based on expected event volume
- **Subscriber Management**: Unused receivers should be dropped
- **Event Size**: Keep payloads reasonably sized for memory efficiency

### Sleep Precision

- **Millisecond Precision**: Sleep timing is accurate to millisecond level
- **System Load**: Heavy system load may affect timing precision
- **Tokio Runtime**: Uses Tokio's timer system for accurate timing

### Memory Usage

- **Event Storage**: Events are held in memory until consumed
- **Snapshot Storage**: Paused workflows create persistent snapshots
- **Receiver Cleanup**: Drop unused receivers to free memory

## Best Practices

### 1. Event Naming Conventions

```rust
// Use descriptive, hierarchical names
"user.approval.required"
"system.maintenance.scheduled"
"workflow.step.completed"
```

### 2. Timeout Strategies

```rust
// Always include reasonable timeouts
let timeout = match operation_type {
    OperationType::UserInput => Some(300_000), // 5 minutes
    OperationType::ApiCall => Some(30_000),    // 30 seconds
    OperationType::FileProcessing => Some(600_000), // 10 minutes
};
```

### 3. Error Recovery

```rust
// Handle pause failures gracefully
if let Err(e) = engine.sleep(duration).await {
    warn!("Sleep failed, continuing anyway: {}", e);
    // Continue execution or implement retry logic
}
```

### 4. Event Payload Design

```rust
// Include enough context for receivers
let event = WorkflowEvent {
    id: generate_unique_id(),
    event_type: "data.processing.complete".to_string(),
    payload: serde_json::json!({
        "workflow_id": workflow_id,
        "processing_time_ms": elapsed_time,
        "records_processed": record_count,
        "success": true,
        "metadata": {
            "source": "batch_processor",
            "version": "1.2.0"
        }
    }),
    timestamp: Utc::now(),
    target_workflow_id: Some(workflow_id),
};
```

## Examples

Run the comprehensive pause execution example:

```bash
cargo run --example pause_execution_example
```

This example demonstrates:
- All four pause methods (`sleep`, `sleepUntil`, `waitForEvent`, `sendEvent`)
- Event bus operations and patterns
- Workflow integration and state management
- Error handling and timeout scenarios
- Performance characteristics and best practices

## Troubleshooting

### Common Issues

1. **Events Not Received**: Check subscriber setup and event type matching
2. **Timeout Too Short**: Increase timeout values for slow operations  
3. **Memory Leaks**: Ensure receivers are properly dropped when done
4. **Timing Inaccuracy**: Consider system load and timer precision limits

### Debug Logging

```rust
// Enable debug logging to trace pause execution
tracing_subscriber::init();

// Logs will show:
// - Event send/receive operations
// - Sleep start/end timing
// - Workflow pause/resume cycles
// - Snapshot creation and cleanup
```