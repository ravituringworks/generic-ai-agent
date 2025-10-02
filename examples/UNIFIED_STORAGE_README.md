# Unified Storage System for AI Agent Infrastructure

## Overview

The Unified Storage System provides a comprehensive, single interface for managing all persistent data requirements of AI agent systems. It consolidates four critical storage domains:

- **Suspended Workflows**: Complete state serialization and resumption capabilities
- **Memory Management**: Thread-based conversation storage with message history
- **Trace Collection**: OpenTelemetry observability data from all system components
- **Evaluation Datasets**: ML model scoring data and evaluation run management

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    StorageManager                           │
│                 (Coordination Layer)                       │
├─────────────────────────────────────────────────────────────┤
│                 UnifiedStorage Trait                       │
│                  (Interface Layer)                         │
├─────────────────────────────────────────────────────────────┤
│  SQLiteStorage  │  PostgreSQLStorage  │  MongoDBStorage    │
│     Backend     │       Backend       │      Backend      │
└─────────────────────────────────────────────────────────────┘
```

### Key Features

#### 🔧 **Unified Interface**
- Single API for all storage operations
- Consistent error handling and retry logic
- Resource-scoped data isolation via `ResourceId`

#### 🔄 **Multiple Backend Support**
- SQLite implementation (production-ready)
- PostgreSQL and MongoDB interfaces (extensible)
- Pluggable backend architecture

#### 🏢 **Multi-tenancy**
- `ResourceId` namespace isolation
- Per-resource data scoping
- Cross-tenant data protection

#### 📊 **Retention Management**
- Configurable retention policies per data type
- Automated cleanup and archival
- Storage size monitoring

#### 🔍 **Observability**
- Built-in performance metrics
- Storage statistics and monitoring
- Query tracing and profiling

## Data Models

### Suspended Workflows
```rust
pub struct SuspendedWorkflow {
    pub workflow_id: String,
    pub resource_id: ResourceId,
    pub workflow_name: String,
    pub current_step: usize,
    pub context_state: Value,
    pub suspended_at: SystemTime,
    pub suspend_reason: SuspendReason,
    pub resume_conditions: Vec<ResumeCondition>,
}
```

### Memory Management
```rust
pub struct MemoryThread {
    pub thread_id: String,
    pub resource_id: ResourceId,
    pub title: String,
    pub message_count: usize,
    // ... metadata fields
}

pub struct MemoryMessage {
    pub message_id: String,
    pub thread_id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
    // ... metadata fields
}
```

### Trace Management
```rust
pub struct TraceData {
    pub trace_id: String,
    pub span_id: String,
    pub resource_id: ResourceId,
    pub operation_name: String,
    pub duration_ms: Option<u64>,
    pub status: TraceStatus,
    pub attributes: HashMap<String, String>,
    pub component: String,
}
```

### Evaluation Management
```rust
pub struct EvalDataset {
    pub dataset_id: String,
    pub name: String,
    pub resource_id: ResourceId,
    pub version: String,
    // ... metadata fields
}

pub struct EvalScore {
    pub score_id: String,
    pub run_id: String,
    pub metric_name: String,
    pub score: f64,
    pub reason: String,
    pub scorer_name: String,
}
```

## Usage Examples

### Basic Setup

```rust
use std::sync::Arc;
use std::time::Duration;

// Create storage backend
let storage = Arc::new(SQLiteUnifiedStorage::new("./app_storage.db"));

// Configure retention policies
let retention_policy = RetentionPolicy {
    traces_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
    memory_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
    eval_retention: Duration::from_secs(90 * 24 * 3600), // 90 days
    suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600), // 1 year
};

// Create storage manager
let storage_manager = Arc::new(StorageManager::new(storage, retention_policy));
```

### Workflow Suspension & Resumption

```rust
// Suspend a running workflow
storage_manager.suspend_workflow(
    "workflow_001",
    ResourceId::new("app", "user_123"),
    "data_processing_workflow",
    current_step,
    &context,
    SuspendReason::WaitingForEvent { 
        event_id: "data_ready".to_string(),
        timeout_ms: Some(300_000)
    },
).await?;

// Resume the workflow later
let (restored_context, step_index) = storage_manager
    .resume_workflow("workflow_001").await?;
```

### Memory Management

```rust
// Create conversation thread
let thread_id = storage_manager.create_conversation_thread(
    ResourceId::new("app", "user_123"),
    "Customer Support Session"
).await?;

// Add messages to thread
storage_manager.add_message(
    &thread_id,
    ResourceId::new("app", "user_123"),
    MessageRole::User,
    "Hello, I need help with my account"
).await?;

storage_manager.add_message(
    &thread_id,
    ResourceId::new("app", "user_123"),
    MessageRole::Assistant,
    "I'd be happy to help! What's the issue?"
).await?;
```

### Trace Recording

```rust
// Record operation traces
let trace_id = storage_manager.record_trace(
    ResourceId::new("app", "user_123"),
    "llm",
    "generate_response",
    start_time,
    end_time,
    TraceStatus::Ok,
    attributes_map,
).await?;
```

### Evaluation Management

```rust
// Create evaluation dataset
let dataset_id = storage_manager.create_evaluation_dataset(
    ResourceId::new("app", "team_ml"),
    "customer_support_qa",
    "Question-answering evaluation dataset",
    "v2.1",
).await?;

// Start evaluation run
let run_id = storage_manager.start_evaluation_run(
    &dataset_id,
    ResourceId::new("app", "team_ml"),
    "gpt-4",
    json!({"temperature": 0.7, "max_tokens": 1000}),
).await?;

// Record evaluation scores
storage_manager.record_evaluation_score(
    &run_id,
    "item_001",
    ResourceId::new("app", "team_ml"),
    "accuracy",
    0.92,
    "High accuracy based on reference answers",
    "automated_scorer_v1",
).await?;
```

### Storage Statistics & Maintenance

```rust
// Get storage statistics
let stats = storage_manager.get_statistics().await?;
println!("Storage usage: {:.2} MB", stats.storage_size_bytes as f64 / (1024.0 * 1024.0));
println!("Active traces: {}", stats.traces);
println!("Memory threads: {}", stats.memory_threads);

// Perform maintenance cleanup
let cleanup_stats = storage_manager.perform_maintenance().await?;
println!("Cleanup freed: {:.2} MB", cleanup_stats.bytes_freed as f64 / (1024.0 * 1024.0));
```

## Production Considerations

### Database Backend Implementation

For production use, implement backends for PostgreSQL and MongoDB:

```rust
pub struct PostgreSQLUnifiedStorage {
    pool: sqlx::PgPool,
}

impl PostgreSQLUnifiedStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }
}

#[async_trait]
impl UnifiedStorage for PostgreSQLUnifiedStorage {
    // Implement all trait methods with real SQL queries
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()> {
        sqlx::query!(
            "INSERT INTO suspended_workflows (id, resource_id, workflow_name, current_step, context_state, suspended_at, suspend_reason) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            workflow.workflow_id,
            workflow.resource_id.to_key(),
            workflow.workflow_name,
            workflow.current_step as i32,
            serde_json::to_value(&workflow.context_state)?,
            workflow.suspended_at,
            serde_json::to_string(&workflow.suspend_reason)?
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Schema Migrations

```sql
-- PostgreSQL schema example
CREATE TABLE suspended_workflows (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL,
    workflow_name TEXT NOT NULL,
    current_step INTEGER NOT NULL,
    context_state JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    suspended_at TIMESTAMPTZ NOT NULL,
    suspend_reason JSONB NOT NULL,
    resume_conditions JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    INDEX idx_resource_id (resource_id),
    INDEX idx_suspended_at (suspended_at)
);

CREATE TABLE memory_threads (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    message_count INTEGER DEFAULT 0,
    INDEX idx_resource_id (resource_id),
    INDEX idx_created_at (created_at)
);

CREATE TABLE memory_messages (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    parent_message_id TEXT,
    FOREIGN KEY (thread_id) REFERENCES memory_threads(id) ON DELETE CASCADE,
    INDEX idx_thread_id (thread_id),
    INDEX idx_timestamp (timestamp)
);

CREATE TABLE traces (
    trace_id TEXT PRIMARY KEY,
    span_id TEXT NOT NULL,
    parent_span_id TEXT,
    resource_id TEXT NOT NULL,
    operation_name TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_ms BIGINT,
    status JSONB NOT NULL,
    attributes JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    component TEXT NOT NULL,
    INDEX idx_resource_id (resource_id),
    INDEX idx_component (component),
    INDEX idx_start_time (start_time),
    INDEX idx_operation_name (operation_name)
);

CREATE TABLE eval_datasets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    resource_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    version TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    INDEX idx_resource_id (resource_id),
    INDEX idx_name (name)
);

CREATE TABLE eval_runs (
    id TEXT PRIMARY KEY,
    dataset_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    run_config JSONB NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    status JSONB NOT NULL,
    summary JSONB,
    FOREIGN KEY (dataset_id) REFERENCES eval_datasets(id) ON DELETE CASCADE,
    INDEX idx_dataset_id (dataset_id),
    INDEX idx_resource_id (resource_id),
    INDEX idx_started_at (started_at)
);

CREATE TABLE eval_scores (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    reason TEXT NOT NULL,
    scorer_name TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    scored_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (run_id) REFERENCES eval_runs(id) ON DELETE CASCADE,
    INDEX idx_run_id (run_id),
    INDEX idx_metric_name (metric_name),
    INDEX idx_scored_at (scored_at)
);
```

### Security & Encryption

```rust
// Example encryption layer
pub struct EncryptedStorage<T: UnifiedStorage> {
    inner: T,
    encryptor: Arc<dyn DataEncryptor>,
}

impl<T: UnifiedStorage> EncryptedStorage<T> {
    pub fn new(storage: T, encryptor: Arc<dyn DataEncryptor>) -> Self {
        Self {
            inner: storage,
            encryptor,
        }
    }
}

#[async_trait]
impl<T: UnifiedStorage> UnifiedStorage for EncryptedStorage<T> {
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()> {
        let encrypted_workflow = self.encryptor.encrypt_workflow(workflow).await?;
        self.inner.store_suspended_workflow(&encrypted_workflow).await
    }
    // ... implement encryption/decryption for all methods
}
```

### Performance Optimizations

1. **Connection Pooling**: Use proper connection pools for database backends
2. **Batch Operations**: Implement batch insert/update operations for high throughput
3. **Caching**: Add Redis-based caching for frequently accessed data
4. **Indexing**: Create appropriate database indices for query patterns
5. **Partitioning**: Partition large tables by resource_id or timestamp
6. **Compression**: Compress large JSON payloads before storage

### Monitoring & Alerting

```rust
// Example metrics integration
pub struct MetricsCollector {
    pub operations_total: Counter,
    pub operation_duration: Histogram,
    pub storage_size: Gauge,
    pub error_rate: Counter,
}

impl MetricsCollector {
    pub fn record_operation(&self, operation: &str, duration: Duration, success: bool) {
        self.operations_total.with_label_values(&[operation]).inc();
        self.operation_duration.with_label_values(&[operation]).observe(duration.as_secs_f64());
        
        if !success {
            self.error_rate.with_label_values(&[operation]).inc();
        }
    }
}
```

## Running the Demo

```bash
cd /path/to/agent-one
cargo run --example unified_storage_system
```

The demo showcases:
- Workflow suspension and resumption
- Conversation thread management
- Trace collection and querying
- Evaluation dataset creation and scoring
- Storage statistics and cleanup
- Integrated workflow operations with storage

## Future Enhancements

1. **Distributed Storage**: Implement sharding across multiple database instances
2. **Replication**: Add master-slave replication for high availability  
3. **Event Streaming**: Integrate with Kafka for real-time data streaming
4. **Backup & Recovery**: Automated backup strategies and point-in-time recovery
5. **Query Interface**: GraphQL or REST API for external data access
6. **Data Lake Integration**: Export to S3/MinIO for long-term analytical storage
7. **Compliance**: GDPR/CCPA compliance features for data privacy
8. **Multi-Region**: Cross-region data replication and disaster recovery