//! Workflow engine for orchestrating agent behavior

use crate::error::{AgentError, Result};
use crate::llm::{Message, Role};
use crate::mcp::{ToolCall, ToolResult};
use crate::memory::SearchResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Serializable snapshot of workflow state for suspend/resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSnapshot {
    /// Unique identifier for this snapshot
    pub id: Uuid,

    /// When this snapshot was created
    pub created_at: DateTime<Utc>,

    /// Workflow context at time of suspension
    pub context: WorkflowContext,

    /// Current step index in execution
    pub current_step: usize,

    /// Reason for suspension
    pub suspend_reason: SuspendReason,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Step-specific state data
    pub step_state: HashMap<String, serde_json::Value>,
}

/// Reasons why a workflow was suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspendReason {
    /// Manual suspension
    Manual,
    /// Waiting for external input
    WaitingForInput(String),
    /// Waiting for external resource availability
    WaitingForResource(String),
    /// Rate limiting or throttling
    RateLimit,
    /// Scheduled pause
    Scheduled,
    /// Error occurred, suspended for recovery
    Error(String),
    /// Sleep for specific duration
    Sleep {
        duration_ms: u64,
        started_at: DateTime<Utc>,
    },
    /// Sleep until specific timestamp
    SleepUntil(DateTime<Utc>),
    /// Waiting for specific event
    WaitingForEvent {
        event_id: String,
        timeout_ms: Option<u64>,
        started_at: DateTime<Utc>,
    },
}

/// Schema definition for step inputs and outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSchema {
    /// Schema type (e.g., "object", "array", "string")
    pub schema_type: String,

    /// Properties for object schemas
    pub properties: HashMap<String, serde_json::Value>,

    /// Required fields
    pub required: Vec<String>,

    /// Additional schema metadata
    pub metadata: HashMap<String, String>,
}

impl StepSchema {
    pub fn new_object() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_property(mut self, name: &str, property_type: &str) -> Self {
        self.properties.insert(
            name.to_string(),
            serde_json::json!({
                "type": property_type
            }),
        );
        self
    }

    pub fn add_required(mut self, field: &str) -> Self {
        self.required.push(field.to_string());
        self
    }

    /// Check if data matches this schema (simplified validation)
    pub fn validates(&self, data: &serde_json::Value) -> bool {
        match self.schema_type.as_str() {
            "object" => {
                if !data.is_object() {
                    return false;
                }

                let obj = data.as_object().unwrap();

                // Check required fields
                for required_field in &self.required {
                    if !obj.contains_key(required_field) {
                        return false;
                    }
                }

                true
            }
            "array" => data.is_array(),
            "string" => data.is_string(),
            "number" => data.is_number(),
            "boolean" => data.is_boolean(),
            _ => true, // Default to valid for unknown types
        }
    }
}

/// Step execution result with data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Result data
    pub data: serde_json::Value,

    /// Step that produced this result
    pub step_name: String,

    /// Execution metadata
    pub metadata: HashMap<String, String>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Control flow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowResult {
    /// Results from executed steps
    pub step_results: Vec<StepResult>,

    /// Whether execution completed successfully
    pub completed: bool,

    /// Final data from control flow
    pub final_data: serde_json::Value,

    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

/// Condition for control flow decisions
pub type ConditionFn =
    Arc<dyn Fn(&WorkflowContext, Option<&serde_json::Value>) -> bool + Send + Sync>;

/// Data mapping function
pub type MapperFn =
    Arc<dyn Fn(&WorkflowContext, &serde_json::Value) -> serde_json::Value + Send + Sync>;

/// Items extraction function for loops
pub type ItemsExtractorFn = Arc<dyn Fn(&WorkflowContext) -> Vec<serde_json::Value> + Send + Sync>;

/// Workflow builder for fluent API
pub struct WorkflowBuilder {
    pub id: String,
    pub steps: Vec<Box<dyn WorkflowStep + Send + Sync>>,
    pub input_schema: Option<StepSchema>,
    pub output_schema: Option<StepSchema>,
    pub metadata: HashMap<String, String>,
    pub concurrency_limit: Option<usize>,
    pub initial_data: Option<serde_json::Value>,
}

impl WorkflowBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            steps: Vec::new(),
            input_schema: None,
            output_schema: None,
            metadata: HashMap::new(),
            concurrency_limit: None,
            initial_data: None,
        }
    }

    /// Add a step to execute sequentially
    pub fn then(mut self, step: Box<dyn WorkflowStep + Send + Sync>) -> Self {
        self.steps.push(step);
        self
    }

    /// Execute multiple steps in parallel
    pub fn parallel(
        self,
        steps: Vec<Box<dyn WorkflowStep + Send + Sync>>,
    ) -> ParallelWorkflowBuilder {
        ParallelWorkflowBuilder {
            builder: self,
            parallel_steps: steps,
        }
    }

    /// Add conditional branching
    pub fn branch(
        self,
        condition: ConditionFn,
        if_true: Box<dyn WorkflowStep + Send + Sync>,
        if_false: Option<Box<dyn WorkflowStep + Send + Sync>>,
    ) -> BranchWorkflowBuilder {
        BranchWorkflowBuilder {
            builder: self,
            condition,
            if_true,
            if_false,
        }
    }

    /// Add do-while loop
    pub fn dowhile(
        self,
        step: Box<dyn WorkflowStep + Send + Sync>,
        condition: ConditionFn,
    ) -> LoopWorkflowBuilder {
        LoopWorkflowBuilder {
            builder: self,
            step,
            condition,
            loop_type: LoopType::DoWhile,
        }
    }

    /// Add do-until loop
    pub fn dountil(
        self,
        step: Box<dyn WorkflowStep + Send + Sync>,
        condition: ConditionFn,
    ) -> LoopWorkflowBuilder {
        LoopWorkflowBuilder {
            builder: self,
            step,
            condition,
            loop_type: LoopType::DoUntil,
        }
    }

    /// Add for-each loop
    pub fn foreach(
        self,
        step: Box<dyn WorkflowStep + Send + Sync>,
        items_extractor: ItemsExtractorFn,
    ) -> ForEachWorkflowBuilder {
        ForEachWorkflowBuilder {
            builder: self,
            step,
            items_extractor,
        }
    }

    /// Add data mapping transformation
    pub fn map(self, mapper: MapperFn) -> MappedWorkflowBuilder {
        MappedWorkflowBuilder {
            builder: self,
            mapper,
        }
    }

    /// Set input schema
    pub fn with_input_schema(mut self, schema: StepSchema) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Set output schema
    pub fn with_output_schema(mut self, schema: StepSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Set concurrency limit for parallel operations
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = Some(limit);
        self
    }

    /// Set initial data
    pub fn with_initial_data(mut self, data: serde_json::Value) -> Self {
        self.initial_data = Some(data);
        self
    }

    /// Clone this workflow builder with a new ID
    pub fn clone_workflow(&self, new_id: &str) -> Self {
        Self {
            id: new_id.to_string(),
            steps: Vec::new(), // Steps need to be re-added as they can't be cloned
            input_schema: self.input_schema.clone(),
            output_schema: self.output_schema.clone(),
            metadata: self.metadata.clone(),
            concurrency_limit: self.concurrency_limit,
            initial_data: self.initial_data.clone(),
        }
    }

    /// Get initial workflow data
    pub fn get_init_data(&self) -> Option<&serde_json::Value> {
        self.initial_data.as_ref()
    }

    /// Build the workflow into an executable engine
    pub fn build(self) -> WorkflowEngine {
        let mut engine = WorkflowEngine::new();
        for step in self.steps {
            engine = engine.add_step(step);
        }
        engine
    }
}

/// Loop type for workflow loops
#[derive(Debug, Clone, Copy)]
pub enum LoopType {
    DoWhile,
    DoUntil,
}

/// Builder for parallel workflow execution
pub struct ParallelWorkflowBuilder {
    builder: WorkflowBuilder,
    parallel_steps: Vec<Box<dyn WorkflowStep + Send + Sync>>,
}

impl ParallelWorkflowBuilder {
    /// Execute the parallel steps and continue with sequential execution
    pub fn then(self, step: Box<dyn WorkflowStep + Send + Sync>) -> WorkflowBuilder {
        // Add parallel execution step
        let parallel_step = ParallelExecutionStep::new(self.parallel_steps);
        self.builder.then(Box::new(parallel_step)).then(step)
    }

    /// Complete the parallel execution
    pub fn build(self) -> WorkflowEngine {
        let parallel_step = ParallelExecutionStep::new(self.parallel_steps);
        self.builder.then(Box::new(parallel_step)).build()
    }
}

/// Builder for conditional workflow execution
pub struct BranchWorkflowBuilder {
    builder: WorkflowBuilder,
    condition: ConditionFn,
    if_true: Box<dyn WorkflowStep + Send + Sync>,
    if_false: Option<Box<dyn WorkflowStep + Send + Sync>>,
}

impl BranchWorkflowBuilder {
    /// Continue with sequential execution after branch
    pub fn then(self, step: Box<dyn WorkflowStep + Send + Sync>) -> WorkflowBuilder {
        let branch_step = BranchExecutionStep::new(self.condition, self.if_true, self.if_false);
        self.builder.then(Box::new(branch_step)).then(step)
    }

    /// Complete the branch execution
    pub fn build(self) -> WorkflowEngine {
        let branch_step = BranchExecutionStep::new(self.condition, self.if_true, self.if_false);
        self.builder.then(Box::new(branch_step)).build()
    }
}

/// Builder for loop workflow execution
pub struct LoopWorkflowBuilder {
    builder: WorkflowBuilder,
    step: Box<dyn WorkflowStep + Send + Sync>,
    condition: ConditionFn,
    loop_type: LoopType,
}

impl LoopWorkflowBuilder {
    /// Continue with sequential execution after loop
    pub fn then(self, step: Box<dyn WorkflowStep + Send + Sync>) -> WorkflowBuilder {
        let loop_step = LoopExecutionStep::new(self.step, self.condition, self.loop_type);
        self.builder.then(Box::new(loop_step)).then(step)
    }

    /// Complete the loop execution
    pub fn build(self) -> WorkflowEngine {
        let loop_step = LoopExecutionStep::new(self.step, self.condition, self.loop_type);
        self.builder.then(Box::new(loop_step)).build()
    }
}

/// Builder for for-each workflow execution
pub struct ForEachWorkflowBuilder {
    builder: WorkflowBuilder,
    step: Box<dyn WorkflowStep + Send + Sync>,
    items_extractor: ItemsExtractorFn,
}

impl ForEachWorkflowBuilder {
    /// Continue with sequential execution after for-each
    pub fn then(self, step: Box<dyn WorkflowStep + Send + Sync>) -> WorkflowBuilder {
        let foreach_step = ForEachExecutionStep::new(self.step, self.items_extractor);
        self.builder.then(Box::new(foreach_step)).then(step)
    }

    /// Complete the for-each execution
    pub fn build(self) -> WorkflowEngine {
        let foreach_step = ForEachExecutionStep::new(self.step, self.items_extractor);
        self.builder.then(Box::new(foreach_step)).build()
    }
}

/// Builder for mapped workflow execution
pub struct MappedWorkflowBuilder {
    builder: WorkflowBuilder,
    mapper: MapperFn,
}

impl MappedWorkflowBuilder {
    /// Continue with sequential execution after mapping
    pub fn then(self, step: Box<dyn WorkflowStep + Send + Sync>) -> WorkflowBuilder {
        let map_step = MapExecutionStep::new(self.mapper);
        self.builder.then(Box::new(map_step)).then(step)
    }

    /// Complete the mapping execution
    pub fn build(self) -> WorkflowEngine {
        let map_step = MapExecutionStep::new(self.mapper);
        self.builder.then(Box::new(map_step)).build()
    }
}

/// Event that can be sent to workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Unique event identifier
    pub id: String,

    /// Event type/name
    pub event_type: String,

    /// Event payload
    pub payload: serde_json::Value,

    /// When the event was created
    pub timestamp: DateTime<Utc>,

    /// Optional workflow ID this event is targeted for
    pub target_workflow_id: Option<String>,
}

/// Event bus for workflow communication
#[derive(Debug)]
pub struct EventBus {
    /// Event broadcasters by event type
    broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<WorkflowEvent>>>>,

    /// Default capacity for event channels
    channel_capacity: usize,
}

impl EventBus {
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            broadcasters: Arc::new(Mutex::new(HashMap::new())),
            channel_capacity,
        }
    }

    /// Send an event to all subscribers of the event type
    pub fn send_event(&self, event: WorkflowEvent) -> Result<usize> {
        let mut broadcasters = self.broadcasters.lock().unwrap();

        let sender = broadcasters
            .entry(event.event_type.clone())
            .or_insert_with(|| broadcast::channel(self.channel_capacity).0);

        let subscriber_count = sender.receiver_count();

        if subscriber_count > 0 {
            match sender.send(event.clone()) {
                Ok(_) => {
                    info!(
                        "Sent event '{}' to {} subscribers",
                        event.event_type, subscriber_count
                    );
                    Ok(subscriber_count)
                }
                Err(_) => {
                    warn!(
                        "Failed to send event '{}' - no active receivers",
                        event.event_type
                    );
                    Ok(0)
                }
            }
        } else {
            debug!("No subscribers for event type '{}'", event.event_type);
            Ok(0)
        }
    }

    /// Subscribe to events of a specific type
    pub fn subscribe(&self, event_type: &str) -> broadcast::Receiver<WorkflowEvent> {
        let mut broadcasters = self.broadcasters.lock().unwrap();

        let sender = broadcasters
            .entry(event_type.to_string())
            .or_insert_with(|| broadcast::channel(self.channel_capacity).0);

        sender.subscribe()
    }

    /// Wait for a specific event with optional timeout
    pub async fn wait_for_event(
        &self,
        event_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<Option<WorkflowEvent>> {
        let mut receiver = self.subscribe(event_id);

        if let Some(timeout) = timeout_ms {
            match tokio::time::timeout(Duration::from_millis(timeout), receiver.recv()).await {
                Ok(Ok(event)) => Ok(Some(event)),
                Ok(Err(_)) => Ok(None), // Channel closed
                Err(_) => Ok(None),     // Timeout
            }
        } else {
            match receiver.recv().await {
                Ok(event) => Ok(Some(event)),
                Err(_) => Ok(None), // Channel closed
            }
        }
    }

    /// Get the number of active subscribers for an event type
    pub fn subscriber_count(&self, event_type: &str) -> usize {
        let broadcasters = self.broadcasters.lock().unwrap();
        broadcasters
            .get(event_type)
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(100) // Default channel capacity
    }
}

/// Context passed between workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Current conversation messages
    pub messages: Vec<Message>,

    /// Retrieved memories
    pub memories: Vec<SearchResult>,

    /// Available tools
    pub available_tools: Vec<String>,

    /// Tool call results
    pub tool_results: HashMap<String, ToolResult>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Step counter
    pub step_count: usize,

    /// Maximum steps allowed
    pub max_steps: usize,
}

impl WorkflowContext {
    pub fn new(max_steps: usize) -> Self {
        Self {
            messages: Vec::new(),
            memories: Vec::new(),
            available_tools: Vec::new(),
            tool_results: HashMap::new(),
            metadata: HashMap::new(),
            step_count: 0,
            max_steps,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_tool_result(&mut self, tool_call_id: String, result: ToolResult) {
        self.tool_results.insert(tool_call_id, result);
    }

    pub fn should_continue(&self) -> bool {
        self.step_count < self.max_steps
    }

    pub fn increment_step(&mut self) {
        self.step_count += 1;
    }

    /// Check if the context can be suspended
    pub fn can_suspend(&self) -> bool {
        // Add any conditions that would prevent suspension
        true
    }

    /// Mark that memories have been retrieved
    pub fn mark_memories_retrieved(&mut self) {
        self.metadata
            .insert("memories_retrieved".to_string(), "true".to_string());
    }
}

/// Storage trait for workflow snapshots
#[async_trait]
pub trait SnapshotStorage: Send + Sync {
    /// Store a workflow snapshot
    async fn store_snapshot(&self, snapshot: &WorkflowSnapshot) -> Result<()>;

    /// Retrieve a workflow snapshot by ID
    async fn get_snapshot(&self, id: Uuid) -> Result<Option<WorkflowSnapshot>>;

    /// List all snapshots, optionally filtered by metadata
    async fn list_snapshots(
        &self,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<WorkflowSnapshot>>;

    /// Delete a snapshot
    async fn delete_snapshot(&self, id: Uuid) -> Result<bool>;

    /// Clean up old snapshots (older than specified duration)
    async fn cleanup_old_snapshots(&self, older_than: chrono::Duration) -> Result<usize>;
}

/// SQLite-based snapshot storage implementation
#[derive(Debug, Clone)]
pub struct SqliteSnapshotStorage {
    pool: Option<sqlx::SqlitePool>,
    database_url: String,
}

impl SqliteSnapshotStorage {
    pub fn new(database_url: String) -> Self {
        Self {
            pool: None,
            database_url,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let pool = sqlx::SqlitePool::connect(&self.database_url)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to connect to database: {}", e)))?;

        // Create snapshots table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflow_snapshots (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                context_json TEXT NOT NULL,
                current_step INTEGER NOT NULL,
                suspend_reason TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                step_state_json TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| AgentError::Workflow(format!("Failed to create snapshots table: {}", e)))?;

        // Create index on created_at for efficient cleanup
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_created_at ON workflow_snapshots(created_at)",
        )
        .execute(&pool)
        .await
        .map_err(|e| AgentError::Workflow(format!("Failed to create index: {}", e)))?;

        self.pool = Some(pool);
        info!(
            "Initialized SQLite snapshot storage at: {}",
            self.database_url
        );
        Ok(())
    }

    fn pool(&self) -> Result<&sqlx::SqlitePool> {
        self.pool
            .as_ref()
            .ok_or_else(|| AgentError::Workflow("Snapshot storage not initialized".to_string()))
    }
}

#[async_trait]
impl SnapshotStorage for SqliteSnapshotStorage {
    async fn store_snapshot(&self, snapshot: &WorkflowSnapshot) -> Result<()> {
        let pool = self.pool()?;

        let context_json = serde_json::to_string(&snapshot.context)
            .map_err(|e| AgentError::Workflow(format!("Failed to serialize context: {}", e)))?;

        let suspend_reason_json = serde_json::to_string(&snapshot.suspend_reason).map_err(|e| {
            AgentError::Workflow(format!("Failed to serialize suspend reason: {}", e))
        })?;

        let metadata_json = serde_json::to_string(&snapshot.metadata)
            .map_err(|e| AgentError::Workflow(format!("Failed to serialize metadata: {}", e)))?;

        let step_state_json = serde_json::to_string(&snapshot.step_state)
            .map_err(|e| AgentError::Workflow(format!("Failed to serialize step state: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO workflow_snapshots 
            (id, created_at, context_json, current_step, suspend_reason, metadata_json, step_state_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                created_at = excluded.created_at,
                context_json = excluded.context_json,
                current_step = excluded.current_step,
                suspend_reason = excluded.suspend_reason,
                metadata_json = excluded.metadata_json,
                step_state_json = excluded.step_state_json
            "#
        )
        .bind(snapshot.id.to_string())
        .bind(snapshot.created_at.to_rfc3339())
        .bind(context_json)
        .bind(snapshot.current_step as i64)
        .bind(suspend_reason_json)
        .bind(metadata_json)
        .bind(step_state_json)
        .execute(pool)
        .await
        .map_err(|e| AgentError::Workflow(format!("Failed to store snapshot: {}", e)))?;

        debug!("Stored workflow snapshot in database: {}", snapshot.id);
        Ok(())
    }

    async fn get_snapshot(&self, id: Uuid) -> Result<Option<WorkflowSnapshot>> {
        let pool = self.pool()?;

        let row = sqlx::query(
            "SELECT id, created_at, context_json, current_step, suspend_reason, metadata_json, step_state_json FROM workflow_snapshots WHERE id = ?1"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AgentError::Workflow(format!("Failed to fetch snapshot: {}", e)))?;

        if let Some(row) = row {
            let created_at_str: String = row.get("created_at");
            let context_json: String = row.get("context_json");
            let current_step: i64 = row.get("current_step");
            let suspend_reason_json: String = row.get("suspend_reason");
            let metadata_json: String = row.get("metadata_json");
            let step_state_json: String = row.get("step_state_json");

            let context = serde_json::from_str(&context_json).map_err(|e| {
                AgentError::Workflow(format!("Failed to deserialize context: {}", e))
            })?;

            let suspend_reason = serde_json::from_str(&suspend_reason_json).map_err(|e| {
                AgentError::Workflow(format!("Failed to deserialize suspend reason: {}", e))
            })?;

            let metadata = serde_json::from_str(&metadata_json).map_err(|e| {
                AgentError::Workflow(format!("Failed to deserialize metadata: {}", e))
            })?;

            let step_state = serde_json::from_str(&step_state_json).map_err(|e| {
                AgentError::Workflow(format!("Failed to deserialize step state: {}", e))
            })?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| AgentError::Workflow(format!("Failed to parse created_at: {}", e)))?
                .with_timezone(&Utc);

            Ok(Some(WorkflowSnapshot {
                id,
                created_at,
                context,
                current_step: current_step as usize,
                suspend_reason,
                metadata,
                step_state,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_snapshots(
        &self,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<WorkflowSnapshot>> {
        let pool = self.pool()?;

        let rows = sqlx::query("SELECT id FROM workflow_snapshots ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to list snapshots: {}", e)))?;

        let mut snapshots = Vec::new();

        for row in rows {
            let id_str: String = row.get("id");
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Ok(Some(snapshot)) = self.get_snapshot(id).await {
                    // Apply filter if provided
                    if let Some(ref filter_map) = filter {
                        let mut matches = true;
                        for (key, value) in filter_map {
                            if snapshot.metadata.get(key) != Some(value) {
                                matches = false;
                                break;
                            }
                        }
                        if matches {
                            snapshots.push(snapshot);
                        }
                    } else {
                        snapshots.push(snapshot);
                    }
                }
            }
        }

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, id: Uuid) -> Result<bool> {
        let pool = self.pool()?;

        let result = sqlx::query("DELETE FROM workflow_snapshots WHERE id = ?1")
            .bind(id.to_string())
            .execute(pool)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to delete snapshot: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn cleanup_old_snapshots(&self, older_than: chrono::Duration) -> Result<usize> {
        let pool = self.pool()?;
        let cutoff = Utc::now() - older_than;

        let result = sqlx::query("DELETE FROM workflow_snapshots WHERE created_at < ?1")
            .bind(cutoff.to_rfc3339())
            .execute(pool)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to cleanup snapshots: {}", e)))?;

        let deleted_count = result.rows_affected() as usize;
        if deleted_count > 0 {
            info!("Cleaned up {} old workflow snapshots", deleted_count);
        }

        Ok(deleted_count)
    }
}

/// File-based snapshot storage implementation
#[derive(Debug)]
pub struct FileSnapshotStorage {
    storage_dir: std::path::PathBuf,
}

impl FileSnapshotStorage {
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
        }
    }

    fn snapshot_path(&self, id: Uuid) -> std::path::PathBuf {
        self.storage_dir.join(format!("{}.json", id))
    }
}

#[async_trait]
impl SnapshotStorage for FileSnapshotStorage {
    async fn store_snapshot(&self, snapshot: &WorkflowSnapshot) -> Result<()> {
        // Ensure storage directory exists
        if !self.storage_dir.exists() {
            fs::create_dir_all(&self.storage_dir).await.map_err(|e| {
                AgentError::Workflow(format!("Failed to create snapshot directory: {}", e))
            })?;
        }

        let path = self.snapshot_path(snapshot.id);
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| AgentError::Workflow(format!("Failed to serialize snapshot: {}", e)))?;

        fs::write(&path, json)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to write snapshot file: {}", e)))?;

        debug!("Stored workflow snapshot at: {}", path.display());
        Ok(())
    }

    async fn get_snapshot(&self, id: Uuid) -> Result<Option<WorkflowSnapshot>> {
        let path = self.snapshot_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to read snapshot file: {}", e)))?;

        let snapshot: WorkflowSnapshot = serde_json::from_str(&json)
            .map_err(|e| AgentError::Workflow(format!("Failed to deserialize snapshot: {}", e)))?;

        Ok(Some(snapshot))
    }

    async fn list_snapshots(
        &self,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<WorkflowSnapshot>> {
        if !self.storage_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        let mut dir = fs::read_dir(&self.storage_dir).await.map_err(|e| {
            AgentError::Workflow(format!("Failed to read snapshot directory: {}", e))
        })?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| AgentError::Workflow(format!("Failed to read directory entry: {}", e)))?
        {
            if let Some(ext) = entry.path().extension() {
                if ext == "json" {
                    if let Some(stem) = entry.path().file_stem() {
                        if let Ok(id) = Uuid::parse_str(&stem.to_string_lossy()) {
                            if let Ok(Some(snapshot)) = self.get_snapshot(id).await {
                                // Apply filter if provided
                                if let Some(ref filter_map) = filter {
                                    let mut matches = true;
                                    for (key, value) in filter_map {
                                        if snapshot.metadata.get(key) != Some(value) {
                                            matches = false;
                                            break;
                                        }
                                    }
                                    if matches {
                                        snapshots.push(snapshot);
                                    }
                                } else {
                                    snapshots.push(snapshot);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by creation time, most recent first
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    async fn delete_snapshot(&self, id: Uuid) -> Result<bool> {
        let path = self.snapshot_path(id);

        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| {
                AgentError::Workflow(format!("Failed to delete snapshot file: {}", e))
            })?;
            debug!("Deleted workflow snapshot: {}", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn cleanup_old_snapshots(&self, older_than: chrono::Duration) -> Result<usize> {
        let cutoff = Utc::now() - older_than;
        let snapshots = self.list_snapshots(None).await?;
        let mut deleted_count = 0;

        for snapshot in snapshots {
            if snapshot.created_at < cutoff && self.delete_snapshot(snapshot.id).await? {
                deleted_count += 1;
            }
        }

        if deleted_count > 0 {
            info!("Cleaned up {} old workflow snapshots", deleted_count);
        }

        Ok(deleted_count)
    }
}

/// A single step in the workflow
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    /// Execute this step
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision>;

    /// Get step name for debugging
    fn name(&self) -> &str;
}

/// Extended trait for workflow steps that support suspension
#[async_trait]
pub trait SuspendableWorkflowStep: WorkflowStep {
    /// Check if this step can be suspended at this point
    async fn can_suspend(&self, context: &WorkflowContext) -> bool {
        // Default implementation - most steps can be suspended
        context.can_suspend()
    }

    /// Capture step-specific state for suspension
    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        // Default implementation - no step-specific state
        Ok(None)
    }

    /// Restore step-specific state after resumption
    async fn restore_state(
        &self,
        _context: &mut WorkflowContext,
        _state: Option<&serde_json::Value>,
    ) -> Result<()> {
        // Default implementation - no state to restore
        Ok(())
    }

    /// Get suspend points - specific conditions where this step should suspend
    fn suspend_points(&self) -> Vec<String> {
        // Default implementation - no specific suspend points
        Vec::new()
    }
}

/// Decision made by a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowDecision {
    /// Continue to next step
    Continue,

    /// Complete the workflow with final response
    Complete(String),

    /// Jump to a specific step
    Jump(String),

    /// Execute tool calls and continue
    ExecuteTools(Vec<ToolCall>),

    /// Retrieve memories and continue
    RetrieveMemories(String),

    /// Suspend workflow with reason
    Suspend(SuspendReason),

    /// Wait for external input/event
    WaitForInput(String),

    /// Sleep for specified milliseconds
    Sleep(u64),

    /// Sleep until specific timestamp
    SleepUntil(DateTime<Utc>),

    /// Wait for specific event
    WaitForEvent {
        event_id: String,
        timeout_ms: Option<u64>,
    },
}

/// Step that retrieves relevant memories
pub struct MemoryRetrievalStep;

#[async_trait]
impl WorkflowStep for MemoryRetrievalStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing memory retrieval step");

        // Only request memory retrieval once per user input
        let already_retrieved = context
            .metadata
            .get("memories_retrieved")
            .map(|v| v == "true")
            .unwrap_or(false);

        if let Some(last_message) = context.messages.last() {
            if matches!(last_message.role, Role::User) && !already_retrieved {
                let content = last_message.content.to_lowercase();

                // Check if this is a query about past conversations or memory-related
                let is_memory_query = content.contains("earlier")
                    || content.contains("before")
                    || content.contains("previous")
                    || content.contains("remember")
                    || content.contains("talked about")
                    || content.contains("discussed")
                    || content.contains("said")
                    || content.contains("conversation")
                    || content.contains("what do i")
                    || content.contains("what did i")
                    || content.contains("do i like")
                    || content.contains("did i tell")
                    || content.contains("did i mention")
                    || (content.contains("what")
                        && (content.contains("like") || content.contains("prefer")))
                    || (content.starts_with("do i") || content.starts_with("did i"));

                if is_memory_query {
                    return Ok(WorkflowDecision::RetrieveMemories(
                        last_message.content.clone(),
                    ));
                }
            }
        }

        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "memory_retrieval"
    }
}

/// Step that analyzes available tools and decides if any should be called
pub struct ToolAnalysisStep;

#[async_trait]
impl WorkflowStep for ToolAnalysisStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing tool analysis step");

        // Avoid re-calling tools if we already have tool results
        if !context.tool_results.is_empty() {
            return Ok(WorkflowDecision::Continue);
        }

        // Simple heuristic: if the user asks for system info, call that tool
        if let Some(last_message) = context.messages.last() {
            if matches!(last_message.role, Role::User) {
                let content = last_message.content.to_lowercase();

                if content.contains("system")
                    && content.contains("info")
                    && context.available_tools.contains(&"system_info".to_string())
                {
                    let tool_call = ToolCall {
                        id: Uuid::new_v4().to_string(),
                        name: "system_info".to_string(),
                        arguments: serde_json::json!({}),
                    };

                    return Ok(WorkflowDecision::ExecuteTools(vec![tool_call]));
                }
            }
        }

        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "tool_analysis"
    }
}

/// Step that generates the final response
pub struct ResponseGenerationStep;

#[async_trait]
impl WorkflowStep for ResponseGenerationStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing response generation step");

        // This would normally call the LLM to generate a response
        // For now, we'll create a simple response based on context

        let mut response_parts = Vec::new();

        // Include tool results if any
        if !context.tool_results.is_empty() {
            response_parts.push("Based on the tools I called:".to_string());
            for result in context.tool_results.values() {
                for content in &result.content {
                    if let crate::mcp::ToolContent::Text { text } = content {
                        response_parts.push(text.clone());
                    }
                }
            }
        }

        // Include memory context if any
        if !context.memories.is_empty() {
            response_parts.push(format!(
                "Based on our previous conversations, I found {} relevant memories:",
                context.memories.len()
            ));
            for (i, memory) in context.memories.iter().enumerate().take(3) {
                // Show top 3
                response_parts.push(format!("{}. {}", i + 1, memory.entry.content));
            }
        }

        // If we have specific content (tool results or memories), provide structured response
        if !response_parts.is_empty() {
            let final_response = response_parts.join("\n\n");
            Ok(WorkflowDecision::Complete(final_response))
        } else {
            // For general queries, we want the LLM to generate the response
            // Return an empty response to signal that LLM generation is needed
            Ok(WorkflowDecision::Complete(String::new()))
        }
    }

    fn name(&self) -> &str {
        "response_generation"
    }
}

/// Enhanced memory retrieval step with suspension support
pub struct EnhancedMemoryRetrievalStep;

#[async_trait]
impl WorkflowStep for EnhancedMemoryRetrievalStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing enhanced memory retrieval step");

        // Check for external memory source availability
        if context.metadata.contains_key("external_memory_unavailable") {
            return Ok(WorkflowDecision::Suspend(
                SuspendReason::WaitingForResource(
                    "External memory service unavailable".to_string(),
                ),
            ));
        }

        // Execute the base memory retrieval logic
        let base_step = MemoryRetrievalStep;
        base_step.execute(context).await
    }

    fn name(&self) -> &str {
        "enhanced_memory_retrieval"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for EnhancedMemoryRetrievalStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        // Can always suspend memory retrieval
        true
    }

    async fn capture_state(&self, context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        // Capture the query being processed
        if let Some(last_message) = context.messages.last() {
            Ok(Some(serde_json::json!({
                "query": last_message.content,
                "retrieved_count": context.memories.len()
            })))
        } else {
            Ok(None)
        }
    }

    fn suspend_points(&self) -> Vec<String> {
        vec![
            "external_memory_unavailable".to_string(),
            "rate_limit_exceeded".to_string(),
        ]
    }
}

/// Human approval step - always requires suspension
pub struct HumanApprovalStep {
    pub approval_message: String,
}

impl HumanApprovalStep {
    pub fn new(message: String) -> Self {
        Self {
            approval_message: message,
        }
    }
}

#[async_trait]
impl WorkflowStep for HumanApprovalStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Requesting human approval: {}", self.approval_message);

        // Check if approval has already been given
        if let Some(approval) = context.metadata.get("human_approval") {
            if approval == "granted" {
                info!("Human approval granted, continuing workflow");
                return Ok(WorkflowDecision::Continue);
            } else if approval == "denied" {
                info!("Human approval denied, completing workflow");
                return Ok(WorkflowDecision::Complete(
                    "Workflow stopped by user denial.".to_string(),
                ));
            }
        }

        // No approval yet, suspend and wait
        Ok(WorkflowDecision::Suspend(SuspendReason::WaitingForInput(
            self.approval_message.clone(),
        )))
    }

    fn name(&self) -> &str {
        "human_approval"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for HumanApprovalStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "approval_message": self.approval_message,
            "timestamp": Utc::now().to_rfc3339()
        })))
    }

    fn suspend_points(&self) -> Vec<String> {
        vec!["waiting_for_approval".to_string()]
    }
}

/// Rate-limited API step
pub struct RateLimitedApiStep {
    pub api_name: String,
    pub calls_per_minute: u32,
}

impl RateLimitedApiStep {
    pub fn new(api_name: String, calls_per_minute: u32) -> Self {
        Self {
            api_name,
            calls_per_minute,
        }
    }
}

#[async_trait]
impl WorkflowStep for RateLimitedApiStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing rate-limited API step for: {}", self.api_name);

        // Check if we're hitting rate limits
        if let Some(last_call) = context
            .metadata
            .get(&format!("{}_last_call", self.api_name))
        {
            if let Ok(last_call_time) = DateTime::parse_from_rfc3339(last_call) {
                let time_since_last = Utc::now() - last_call_time.with_timezone(&Utc);
                let min_interval = chrono::Duration::seconds(60 / self.calls_per_minute as i64);

                if time_since_last < min_interval {
                    let _wait_time = min_interval - time_since_last;
                    return Ok(WorkflowDecision::Suspend(SuspendReason::RateLimit));
                }
            }
        }

        // Record this API call
        context.metadata.insert(
            format!("{}_last_call", self.api_name),
            Utc::now().to_rfc3339(),
        );

        // Simulate API call success
        info!("API call to {} completed", self.api_name);
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "rate_limited_api"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for RateLimitedApiStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "api_name": self.api_name,
            "calls_per_minute": self.calls_per_minute,
            "suspended_at": Utc::now().to_rfc3339()
        })))
    }

    fn suspend_points(&self) -> Vec<String> {
        vec!["rate_limit_exceeded".to_string()]
    }
}

/// Sleep step - pauses workflow for specified duration
pub struct SleepStep {
    pub duration_ms: u64,
}

impl SleepStep {
    pub fn new(duration_ms: u64) -> Self {
        Self { duration_ms }
    }
}

#[async_trait]
impl WorkflowStep for SleepStep {
    async fn execute(&self, _context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Sleep step executing for {}ms", self.duration_ms);
        Ok(WorkflowDecision::Sleep(self.duration_ms))
    }

    fn name(&self) -> &str {
        "sleep"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for SleepStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true // Always suspendable
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "duration_ms": self.duration_ms,
            "step_type": "sleep"
        })))
    }
}

/// Sleep until timestamp step
pub struct SleepUntilStep {
    pub timestamp: DateTime<Utc>,
}

impl SleepUntilStep {
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self { timestamp }
    }
}

#[async_trait]
impl WorkflowStep for SleepUntilStep {
    async fn execute(&self, _context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Sleep until step executing until {}", self.timestamp);
        Ok(WorkflowDecision::SleepUntil(self.timestamp))
    }

    fn name(&self) -> &str {
        "sleep_until"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for SleepUntilStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "timestamp": self.timestamp.to_rfc3339(),
            "step_type": "sleep_until"
        })))
    }
}

/// Wait for event step
pub struct WaitForEventStep {
    pub event_id: String,
    pub timeout_ms: Option<u64>,
}

impl WaitForEventStep {
    pub fn new(event_id: String, timeout_ms: Option<u64>) -> Self {
        Self {
            event_id,
            timeout_ms,
        }
    }
}

#[async_trait]
impl WorkflowStep for WaitForEventStep {
    async fn execute(&self, _context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!(
            "Wait for event step executing for event '{}'",
            self.event_id
        );
        Ok(WorkflowDecision::WaitForEvent {
            event_id: self.event_id.clone(),
            timeout_ms: self.timeout_ms,
        })
    }

    fn name(&self) -> &str {
        "wait_for_event"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for WaitForEventStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "event_id": self.event_id,
            "timeout_ms": self.timeout_ms,
            "step_type": "wait_for_event"
        })))
    }

    fn suspend_points(&self) -> Vec<String> {
        vec![format!("waiting_for_event_{}", self.event_id)]
    }
}

/// Conditional pause step - pauses based on context conditions
pub struct ConditionalPauseStep {
    pub condition_key: String,
    pub pause_type: PauseType,
}

#[derive(Debug, Clone)]
pub enum PauseType {
    Sleep(u64),
    SleepUntil(DateTime<Utc>),
    WaitForEvent {
        event_id: String,
        timeout_ms: Option<u64>,
    },
}

impl ConditionalPauseStep {
    pub fn new(condition_key: String, pause_type: PauseType) -> Self {
        Self {
            condition_key,
            pause_type,
        }
    }
}

#[async_trait]
impl WorkflowStep for ConditionalPauseStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!(
            "Conditional pause step checking condition '{}'",
            self.condition_key
        );

        // Check if condition is met
        if let Some(condition_value) = context.metadata.get(&self.condition_key) {
            if condition_value == "true" {
                match &self.pause_type {
                    PauseType::Sleep(duration_ms) => {
                        info!("Condition met, sleeping for {}ms", duration_ms);
                        return Ok(WorkflowDecision::Sleep(*duration_ms));
                    }
                    PauseType::SleepUntil(timestamp) => {
                        info!("Condition met, sleeping until {}", timestamp);
                        return Ok(WorkflowDecision::SleepUntil(*timestamp));
                    }
                    PauseType::WaitForEvent {
                        event_id,
                        timeout_ms,
                    } => {
                        info!("Condition met, waiting for event '{}'", event_id);
                        return Ok(WorkflowDecision::WaitForEvent {
                            event_id: event_id.clone(),
                            timeout_ms: *timeout_ms,
                        });
                    }
                }
            }
        }

        // Condition not met, continue
        debug!("Condition not met, continuing execution");
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "conditional_pause"
    }
}

#[async_trait]
impl SuspendableWorkflowStep for ConditionalPauseStep {
    async fn can_suspend(&self, _context: &WorkflowContext) -> bool {
        true
    }

    async fn capture_state(&self, _context: &WorkflowContext) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "condition_key": self.condition_key,
            "pause_type": match &self.pause_type {
                PauseType::Sleep(ms) => serde_json::json!({ "type": "sleep", "duration_ms": ms }),
                PauseType::SleepUntil(ts) => serde_json::json!({ "type": "sleep_until", "timestamp": ts.to_rfc3339() }),
                PauseType::WaitForEvent { event_id, timeout_ms } => serde_json::json!({
                    "type": "wait_for_event",
                    "event_id": event_id,
                    "timeout_ms": timeout_ms
                }),
            },
            "step_type": "conditional_pause"
        })))
    }
}

/// Configuration for workflow suspend/resume functionality
#[derive(Debug, Clone)]
pub struct WorkflowSuspendConfig {
    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,

    /// Checkpoint interval (in steps)
    pub checkpoint_interval: usize,

    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,

    /// Auto-cleanup snapshots older than this duration
    pub snapshot_retention: chrono::Duration,
}

impl Default for WorkflowSuspendConfig {
    fn default() -> Self {
        Self {
            auto_checkpoint: true,
            checkpoint_interval: 3,
            max_snapshots: 10,
            snapshot_retention: chrono::Duration::days(7),
        }
    }
}

/// Parallel execution step
pub struct ParallelExecutionStep {
    steps: Vec<Box<dyn WorkflowStep + Send + Sync>>,
}

impl ParallelExecutionStep {
    pub fn new(steps: Vec<Box<dyn WorkflowStep + Send + Sync>>) -> Self {
        Self { steps }
    }
}

#[async_trait]
impl WorkflowStep for ParallelExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing {} steps in parallel", self.steps.len());

        let _handles: Vec<tokio::task::JoinHandle<Result<WorkflowDecision>>> = Vec::new();

        // Execute all steps in parallel
        for (i, step) in self.steps.iter().enumerate() {
            let mut context_clone = context.clone();
            let _step_name = step.name().to_string();

            // Since we can't clone trait objects, we'll execute them sequentially for now
            // In a real implementation, you'd need a different approach
            match step.execute(&mut context_clone).await {
                Ok(decision) => {
                    info!(
                        "Parallel step {} completed with decision: {:?}",
                        i, decision
                    );
                    // For parallel execution, we continue unless there's an error
                    if !matches!(decision, WorkflowDecision::Continue) {
                        return Ok(decision);
                    }
                }
                Err(e) => {
                    warn!("Parallel step {} failed: {}", i, e);
                    return Err(e);
                }
            }
        }

        info!("All parallel steps completed successfully");
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "parallel_execution"
    }
}

/// Branch execution step
pub struct BranchExecutionStep {
    condition: ConditionFn,
    if_true: Box<dyn WorkflowStep + Send + Sync>,
    if_false: Option<Box<dyn WorkflowStep + Send + Sync>>,
}

impl BranchExecutionStep {
    pub fn new(
        condition: ConditionFn,
        if_true: Box<dyn WorkflowStep + Send + Sync>,
        if_false: Option<Box<dyn WorkflowStep + Send + Sync>>,
    ) -> Self {
        Self {
            condition,
            if_true,
            if_false,
        }
    }
}

#[async_trait]
impl WorkflowStep for BranchExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing branch step");

        let condition_result = (self.condition)(context, None);

        if condition_result {
            info!("Branch condition is true, executing if_true branch");
            self.if_true.execute(context).await
        } else if let Some(ref if_false) = self.if_false {
            info!("Branch condition is false, executing if_false branch");
            if_false.execute(context).await
        } else {
            info!("Branch condition is false, no if_false branch, continuing");
            Ok(WorkflowDecision::Continue)
        }
    }

    fn name(&self) -> &str {
        "branch_execution"
    }
}

/// Loop execution step
pub struct LoopExecutionStep {
    step: Box<dyn WorkflowStep + Send + Sync>,
    condition: ConditionFn,
    loop_type: LoopType,
}

impl LoopExecutionStep {
    pub fn new(
        step: Box<dyn WorkflowStep + Send + Sync>,
        condition: ConditionFn,
        loop_type: LoopType,
    ) -> Self {
        Self {
            step,
            condition,
            loop_type,
        }
    }
}

#[async_trait]
impl WorkflowStep for LoopExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing loop step (type: {:?})", self.loop_type);

        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 1000; // Safety limit

        match self.loop_type {
            LoopType::DoWhile => {
                // Execute at least once, then continue while condition is true
                loop {
                    iteration += 1;
                    if iteration > MAX_ITERATIONS {
                        warn!(
                            "Loop exceeded maximum iterations ({}), breaking",
                            MAX_ITERATIONS
                        );
                        break;
                    }

                    debug!("DoWhile loop iteration {}", iteration);
                    let decision = self.step.execute(context).await?;

                    // Check if step wants to suspend or complete
                    if !matches!(decision, WorkflowDecision::Continue) {
                        return Ok(decision);
                    }

                    // Check loop condition
                    if !(self.condition)(context, None) {
                        debug!("DoWhile condition is false, exiting loop");
                        break;
                    }
                }
            }
            LoopType::DoUntil => {
                // Execute at least once, then continue until condition is true
                loop {
                    iteration += 1;
                    if iteration > MAX_ITERATIONS {
                        warn!(
                            "Loop exceeded maximum iterations ({}), breaking",
                            MAX_ITERATIONS
                        );
                        break;
                    }

                    debug!("DoUntil loop iteration {}", iteration);
                    let decision = self.step.execute(context).await?;

                    // Check if step wants to suspend or complete
                    if !matches!(decision, WorkflowDecision::Continue) {
                        return Ok(decision);
                    }

                    // Check loop condition
                    if (self.condition)(context, None) {
                        debug!("DoUntil condition is true, exiting loop");
                        break;
                    }
                }
            }
        }

        info!("Loop completed after {} iterations", iteration);
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "loop_execution"
    }
}

/// For-each execution step
pub struct ForEachExecutionStep {
    step: Box<dyn WorkflowStep + Send + Sync>,
    items_extractor: ItemsExtractorFn,
}

impl ForEachExecutionStep {
    pub fn new(
        step: Box<dyn WorkflowStep + Send + Sync>,
        items_extractor: ItemsExtractorFn,
    ) -> Self {
        Self {
            step,
            items_extractor,
        }
    }
}

#[async_trait]
impl WorkflowStep for ForEachExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing for-each step");

        let items = (self.items_extractor)(context);
        info!("Processing {} items in for-each loop", items.len());

        for (index, item) in items.iter().enumerate() {
            debug!("For-each iteration {} processing item: {:?}", index, item);

            // Add current item to context metadata
            context
                .metadata
                .insert("foreach_current_item".to_string(), item.to_string());
            context
                .metadata
                .insert("foreach_current_index".to_string(), index.to_string());

            let decision = self.step.execute(context).await?;

            // Check if step wants to suspend or complete
            if !matches!(decision, WorkflowDecision::Continue) {
                return Ok(decision);
            }
        }

        // Clean up metadata
        context.metadata.remove("foreach_current_item");
        context.metadata.remove("foreach_current_index");

        info!("For-each loop completed");
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "foreach_execution"
    }
}

/// Data mapping execution step
pub struct MapExecutionStep {
    mapper: MapperFn,
}

impl MapExecutionStep {
    pub fn new(mapper: MapperFn) -> Self {
        Self { mapper }
    }
}

#[async_trait]
impl WorkflowStep for MapExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing data mapping step");

        // Get the last result or use empty object as input
        let input_data = context
            .metadata
            .get("last_result")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::json!({}));

        let mapped_data = (self.mapper)(context, &input_data);

        // Store mapped result
        context.metadata.insert(
            "last_result".to_string(),
            serde_json::to_string(&mapped_data).unwrap_or_default(),
        );

        debug!("Data mapping completed");
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "map_execution"
    }
}

/// Workflow engine that orchestrates the execution of steps
pub struct WorkflowEngine {
    steps: Vec<Box<dyn WorkflowStep>>,
    suspend_config: WorkflowSuspendConfig,
    snapshot_storage: Option<Box<dyn SnapshotStorage>>,
    event_bus: Arc<EventBus>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            suspend_config: WorkflowSuspendConfig::default(),
            snapshot_storage: None,
            event_bus: Arc::new(EventBus::default()),
        }
    }

    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }

    pub fn with_suspend_config(mut self, config: WorkflowSuspendConfig) -> Self {
        self.suspend_config = config;
        self
    }

    pub fn with_snapshot_storage(mut self, storage: Box<dyn SnapshotStorage>) -> Self {
        self.snapshot_storage = Some(storage);
        self
    }

    pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_default_steps(self) -> Self {
        self.add_step(Box::new(MemoryRetrievalStep))
            .add_step(Box::new(ToolAnalysisStep))
            .add_step(Box::new(ResponseGenerationStep))
    }

    /// Create a snapshot of the current workflow state
    pub async fn create_snapshot(
        &self,
        context: &WorkflowContext,
        current_step: usize,
        reason: SuspendReason,
    ) -> Result<WorkflowSnapshot> {
        let step_state = HashMap::new();

        // Capture step-specific state if the step supports it
        // Note: This would require downcasting trait objects, which is complex
        // For now, we skip step-specific state capture in snapshots

        Ok(WorkflowSnapshot {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            context: context.clone(),
            current_step,
            suspend_reason: reason,
            metadata: HashMap::new(),
            step_state,
        })
    }

    /// Suspend the workflow and store a snapshot
    pub async fn suspend(
        &self,
        context: &WorkflowContext,
        current_step: usize,
        reason: SuspendReason,
    ) -> Result<Uuid> {
        let snapshot = self.create_snapshot(context, current_step, reason).await?;
        let snapshot_id = snapshot.id;

        if let Some(ref storage) = self.snapshot_storage {
            storage.store_snapshot(&snapshot).await?;
            info!("Workflow suspended with snapshot ID: {}", snapshot_id);

            // Cleanup old snapshots if configured
            if self.suspend_config.max_snapshots > 0 {
                self.cleanup_snapshots().await?;
            }
        } else {
            warn!("No snapshot storage configured, cannot persist workflow state");
        }

        Ok(snapshot_id)
    }

    /// Resume workflow execution from a snapshot
    pub async fn resume_from_snapshot(&self, snapshot_id: Uuid) -> Result<WorkflowResult> {
        let storage = self
            .snapshot_storage
            .as_ref()
            .ok_or_else(|| AgentError::Workflow("No snapshot storage configured".to_string()))?;

        let snapshot = storage
            .get_snapshot(snapshot_id)
            .await?
            .ok_or_else(|| AgentError::Workflow(format!("Snapshot not found: {}", snapshot_id)))?;

        info!(
            "Resuming workflow from snapshot: {} (created: {})",
            snapshot_id, snapshot.created_at
        );

        // Restore the workflow context
        let context = snapshot.context;

        // Resume execution from the suspended step
        self.execute_from_step(context, snapshot.current_step).await
    }

    /// Execute workflow starting from a specific step
    async fn execute_from_step(
        &self,
        mut context: WorkflowContext,
        start_step: usize,
    ) -> Result<WorkflowResult> {
        info!("Resuming workflow execution from step {}", start_step);

        context.increment_step();

        // Execute steps starting from the specified step
        for (step_index, step) in self.steps.iter().enumerate().skip(start_step) {
            debug!("Executing step: {} (index: {})", step.name(), step_index);

            // Auto-checkpoint if configured
            if self.suspend_config.auto_checkpoint
                && step_index % self.suspend_config.checkpoint_interval == 0
            {
                if let Err(e) = self
                    .suspend(&context, step_index, SuspendReason::Scheduled)
                    .await
                {
                    warn!("Failed to create automatic checkpoint: {}", e);
                }
            }

            match step.execute(&mut context).await? {
                WorkflowDecision::Continue => {
                    continue;
                }
                WorkflowDecision::Complete(response) => {
                    let step_count = context.step_count;
                    info!("Workflow completed after {} steps", step_count);
                    return Ok(WorkflowResult {
                        response,
                        context,
                        completed: true,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
                WorkflowDecision::Jump(step_name) => {
                    debug!("Jump to step requested: {}", step_name);
                    return Err(AgentError::Workflow(
                        "Step jumping not implemented".to_string(),
                    ));
                }
                WorkflowDecision::ExecuteTools(tool_calls) => {
                    debug!("Tool execution requested: {} tools", tool_calls.len());
                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: String::new(),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    }
                    .with_tool_calls(tool_calls));
                }
                WorkflowDecision::RetrieveMemories(query) => {
                    debug!("Memory retrieval requested for: {}", query);
                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: String::new(),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    }
                    .with_memory_query(query));
                }
                WorkflowDecision::Suspend(reason) => {
                    info!("Workflow step requested suspension: {:?}", reason);
                    let snapshot_id = self.suspend(&context, step_index, reason).await?;
                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: format!("Workflow suspended (ID: {})", snapshot_id),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
                WorkflowDecision::WaitForInput(message) => {
                    info!("Workflow waiting for input: {}", message);
                    let reason = SuspendReason::WaitingForInput(message.clone());
                    let snapshot_id = self.suspend(&context, step_index, reason).await?;
                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: format!(
                            "Waiting for input: {} (Suspended with ID: {})",
                            message, snapshot_id
                        ),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
                WorkflowDecision::Sleep(duration_ms) => {
                    info!("Workflow sleeping for {}ms", duration_ms);
                    let started_at = Utc::now();
                    let reason = SuspendReason::Sleep {
                        duration_ms,
                        started_at,
                    };
                    let snapshot_id = self.suspend(&context, step_index, reason).await?;

                    // Note: Auto-resume would require shared ownership of engine
                    // For now, we'll just suspend and let external code handle resume
                    info!(
                        "Workflow suspended for sleep, use resume_from_snapshot({}) after {}ms",
                        snapshot_id, duration_ms
                    );

                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: format!(
                            "Sleeping for {}ms (Suspended with ID: {})",
                            duration_ms, snapshot_id
                        ),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
                WorkflowDecision::SleepUntil(timestamp) => {
                    info!("Workflow sleeping until {}", timestamp);
                    let reason = SuspendReason::SleepUntil(timestamp);
                    let snapshot_id = self.suspend(&context, step_index, reason).await?;

                    // Note: Auto-resume would require shared ownership of engine
                    info!(
                        "Workflow suspended until {}, use resume_from_snapshot({}) after timestamp",
                        timestamp, snapshot_id
                    );

                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: format!(
                            "Sleeping until {} (Suspended with ID: {})",
                            timestamp, snapshot_id
                        ),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
                WorkflowDecision::WaitForEvent {
                    event_id,
                    timeout_ms,
                } => {
                    info!(
                        "Workflow waiting for event '{}' with timeout {:?}ms",
                        event_id, timeout_ms
                    );
                    let started_at = Utc::now();
                    let reason = SuspendReason::WaitingForEvent {
                        event_id: event_id.clone(),
                        timeout_ms,
                        started_at,
                    };
                    let snapshot_id = self.suspend(&context, step_index, reason).await?;

                    // Note: Auto-resume would require shared ownership of engine
                    // Events can be sent using send_event() method and workflows resumed manually
                    info!("Workflow suspended waiting for event '{}', use send_event() and resume_from_snapshot({})", event_id, snapshot_id);

                    let step_count = context.step_count;
                    return Ok(WorkflowResult {
                        response: format!(
                            "Waiting for event '{}' (Suspended with ID: {})",
                            event_id, snapshot_id
                        ),
                        context,
                        completed: false,
                        steps_executed: step_count,
                        pending_tool_calls: None,
                        pending_memory_query: None,
                    });
                }
            }
        }

        let step_count = context.step_count;
        Ok(WorkflowResult {
            response: "Workflow reached maximum steps.".to_string(),
            context,
            completed: true,
            steps_executed: step_count,
            pending_tool_calls: None,
            pending_memory_query: None,
        })
    }

    /// Clean up old snapshots based on configuration
    async fn cleanup_snapshots(&self) -> Result<()> {
        if let Some(ref storage) = self.snapshot_storage {
            let snapshots = storage.list_snapshots(None).await?;

            // Remove excess snapshots (keep only the most recent)
            if snapshots.len() > self.suspend_config.max_snapshots {
                let to_delete = snapshots.len() - self.suspend_config.max_snapshots;
                for snapshot in snapshots
                    .iter()
                    .skip(self.suspend_config.max_snapshots)
                    .take(to_delete)
                {
                    storage.delete_snapshot(snapshot.id).await?;
                }
            }

            // Remove old snapshots based on retention policy
            storage
                .cleanup_old_snapshots(self.suspend_config.snapshot_retention)
                .await?;
        }

        Ok(())
    }

    /// List all available snapshots
    pub async fn list_snapshots(
        &self,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<WorkflowSnapshot>> {
        if let Some(ref storage) = self.snapshot_storage {
            storage.list_snapshots(filter).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Delete a specific snapshot
    pub async fn delete_snapshot(&self, snapshot_id: Uuid) -> Result<bool> {
        if let Some(ref storage) = self.snapshot_storage {
            storage.delete_snapshot(snapshot_id).await
        } else {
            Ok(false)
        }
    }

    /// Store a snapshot through the engine
    pub async fn store_snapshot(&self, snapshot: &WorkflowSnapshot) -> Result<()> {
        if let Some(ref storage) = self.snapshot_storage {
            storage.store_snapshot(snapshot).await
        } else {
            Err(AgentError::Workflow(
                "No snapshot storage configured".to_string(),
            ))
        }
    }

    /// Pause execution for specified milliseconds
    pub async fn sleep(&self, duration_ms: u64) -> Result<()> {
        info!("Sleeping for {} milliseconds", duration_ms);
        sleep(Duration::from_millis(duration_ms)).await;
        Ok(())
    }

    /// Pause until a specific timestamp
    pub async fn sleep_until(&self, timestamp: DateTime<Utc>) -> Result<()> {
        let now = Utc::now();

        if timestamp <= now {
            debug!("Timestamp is in the past, not sleeping");
            return Ok(());
        }

        let duration = timestamp - now;
        let duration_ms = duration.num_milliseconds().max(0) as u64;

        info!("Sleeping until {} ({}ms from now)", timestamp, duration_ms);
        sleep(Duration::from_millis(duration_ms)).await;
        Ok(())
    }

    /// Wait for a specific event to be received
    pub async fn wait_for_event(
        &self,
        event_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<Option<WorkflowEvent>> {
        info!(
            "Waiting for event '{}' with timeout {:?}ms",
            event_id, timeout_ms
        );
        self.event_bus.wait_for_event(event_id, timeout_ms).await
    }

    /// Send an event to resume waiting workflows
    pub fn send_event(&self, event: WorkflowEvent) -> Result<usize> {
        info!("Sending event '{}' (type: {})", event.id, event.event_type);
        self.event_bus.send_event(event)
    }

    /// Create and send an event with given parameters
    pub fn send_simple_event(
        &self,
        event_id: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<usize> {
        let event = WorkflowEvent {
            id: event_id.to_string(),
            event_type: event_type.to_string(),
            payload,
            timestamp: Utc::now(),
            target_workflow_id: None,
        };
        self.send_event(event)
    }

    /// Get the event bus for advanced event operations
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Execute the workflow
    pub async fn execute(&self, context: WorkflowContext) -> Result<WorkflowResult> {
        info!(
            "Starting workflow execution with {} steps",
            self.steps.len()
        );
        self.execute_from_step(context, 0).await
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
            .with_default_steps()
            .with_suspend_config(WorkflowSuspendConfig::default())
    }
}

/// Result of workflow execution
#[derive(Debug)]
pub struct WorkflowResult {
    /// Generated response (if completed)
    pub response: String,

    /// Final workflow context
    pub context: WorkflowContext,

    /// Whether the workflow completed successfully
    pub completed: bool,

    /// Number of steps executed
    pub steps_executed: usize,

    /// Tool calls to execute (if any)
    pub pending_tool_calls: Option<Vec<ToolCall>>,

    /// Memory query to execute (if any)
    pub pending_memory_query: Option<String>,
}

impl WorkflowResult {
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.pending_tool_calls = Some(tool_calls);
        self
    }

    pub fn with_memory_query(mut self, query: String) -> Self {
        self.pending_memory_query = Some(query);
        self
    }

    pub fn has_pending_actions(&self) -> bool {
        self.pending_tool_calls.is_some() || self.pending_memory_query.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::user_message;
    use std::collections::HashMap;
    use std::time::Instant;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_workflow_context() {
        let mut context = WorkflowContext::new(5);

        assert_eq!(context.step_count, 0);
        assert!(context.should_continue());

        context.increment_step();
        assert_eq!(context.step_count, 1);

        context.add_message(user_message("Hello"));
        assert_eq!(context.messages.len(), 1);

        // Test suspend capability
        assert!(context.can_suspend());

        // Test memory retrieval marking
        context.mark_memories_retrieved();
        assert_eq!(
            context.metadata.get("memories_retrieved"),
            Some(&"true".to_string())
        );
    }

    #[tokio::test]
    async fn test_memory_retrieval_step() {
        let step = MemoryRetrievalStep;
        let mut context = WorkflowContext::new(5);

        // No user message
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // With user message
        context.add_message(user_message("What is Rust?"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::RetrieveMemories(_)));
    }

    #[tokio::test]
    async fn test_tool_analysis_step() {
        let step = ToolAnalysisStep;
        let mut context = WorkflowContext::new(5);
        context.available_tools.push("system_info".to_string());

        // No relevant message
        context.add_message(user_message("Hello"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // System info request
        context.add_message(user_message("Show me system info"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::ExecuteTools(_)));
    }

    #[tokio::test]
    async fn test_workflow_engine() {
        let engine = WorkflowEngine::default();
        let mut context = WorkflowContext::new(10);
        context.add_message(user_message("Hello, how are you?"));

        let result = engine.execute(context).await.unwrap();
        assert!(result.completed);
        assert!(!result.response.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_snapshot_serialization() {
        let context = WorkflowContext::new(5);
        let snapshot = WorkflowSnapshot {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            context,
            current_step: 2,
            suspend_reason: SuspendReason::Manual,
            metadata: HashMap::new(),
            step_state: HashMap::new(),
        };

        // Test serialization/deserialization
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: WorkflowSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot.id, deserialized.id);
        assert_eq!(snapshot.current_step, deserialized.current_step);
        assert!(matches!(deserialized.suspend_reason, SuspendReason::Manual));
    }

    #[tokio::test]
    async fn test_file_snapshot_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = FileSnapshotStorage::new(temp_dir.path());

        let context = WorkflowContext::new(5);
        let snapshot = WorkflowSnapshot {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            context,
            current_step: 1,
            suspend_reason: SuspendReason::WaitingForInput("test".to_string()),
            metadata: HashMap::new(),
            step_state: HashMap::new(),
        };

        // Test storing snapshot
        storage.store_snapshot(&snapshot).await.unwrap();

        // Test retrieving snapshot
        let retrieved = storage.get_snapshot(snapshot.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_snapshot = retrieved.unwrap();
        assert_eq!(snapshot.id, retrieved_snapshot.id);
        assert_eq!(snapshot.current_step, retrieved_snapshot.current_step);

        // Test listing snapshots
        let snapshots = storage.list_snapshots(None).await.unwrap();
        assert_eq!(snapshots.len(), 1);

        // Test deleting snapshot
        let deleted = storage.delete_snapshot(snapshot.id).await.unwrap();
        assert!(deleted);

        let retrieved_after_delete = storage.get_snapshot(snapshot.id).await.unwrap();
        assert!(retrieved_after_delete.is_none());
    }

    #[tokio::test]
    async fn test_human_approval_step() {
        let step = HumanApprovalStep::new("Approve this action?".to_string());
        let mut context = WorkflowContext::new(5);

        // No approval yet - should suspend
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(
            decision,
            WorkflowDecision::Suspend(SuspendReason::WaitingForInput(_))
        ));

        // Approval granted
        context
            .metadata
            .insert("human_approval".to_string(), "granted".to_string());
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // Approval denied
        context
            .metadata
            .insert("human_approval".to_string(), "denied".to_string());
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Complete(_)));
    }

    #[tokio::test]
    async fn test_rate_limited_api_step() {
        let step = RateLimitedApiStep::new("test_api".to_string(), 60); // 1 call per second
        let mut context = WorkflowContext::new(5);

        // First call should succeed
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // Immediate second call should trigger rate limit
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(
            decision,
            WorkflowDecision::Suspend(SuspendReason::RateLimit)
        ));
    }

    #[tokio::test]
    async fn test_enhanced_memory_retrieval_step() {
        let step = EnhancedMemoryRetrievalStep;
        let mut context = WorkflowContext::new(5);

        // Normal operation
        context.add_message(user_message("What did we discuss before?"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::RetrieveMemories(_)));

        // External memory unavailable
        context.metadata.insert(
            "external_memory_unavailable".to_string(),
            "true".to_string(),
        );
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(
            decision,
            WorkflowDecision::Suspend(SuspendReason::WaitingForResource(_))
        ));
    }

    #[tokio::test]
    async fn test_workflow_engine_with_suspend_resume() {
        let temp_dir = tempdir().unwrap();
        let storage = FileSnapshotStorage::new(temp_dir.path());

        let engine = WorkflowEngine::new()
            .with_suspend_config(WorkflowSuspendConfig::default())
            .with_snapshot_storage(Box::new(storage))
            .add_step(Box::new(HumanApprovalStep::new(
                "Test approval".to_string(),
            )));

        let mut context = WorkflowContext::new(10);
        context.add_message(user_message("Test message"));

        // Execute workflow - should suspend for human approval
        let result = engine.execute(context).await.unwrap();
        assert!(!result.completed);
        assert!(result.response.contains("suspended"));

        // List snapshots
        let snapshots = engine.list_snapshots(None).await.unwrap();
        assert!(!snapshots.is_empty());

        let snapshot = &snapshots[0];
        assert!(matches!(
            snapshot.suspend_reason,
            SuspendReason::WaitingForInput(_)
        ));

        // Resume with approval
        let mut resumed_context = snapshot.context.clone();
        resumed_context
            .metadata
            .insert("human_approval".to_string(), "granted".to_string());

        let new_snapshot = engine
            .create_snapshot(
                &resumed_context,
                snapshot.current_step,
                SuspendReason::Manual,
            )
            .await
            .unwrap();

        // Store and resume
        engine.store_snapshot(&new_snapshot).await.unwrap();

        let resumed_result = engine.resume_from_snapshot(new_snapshot.id).await.unwrap();
        // The workflow should complete or continue based on the implementation
        assert!(resumed_result.steps_executed > 0);
    }

    #[tokio::test]
    async fn test_suspend_config() {
        let config = WorkflowSuspendConfig {
            auto_checkpoint: true,
            checkpoint_interval: 2,
            max_snapshots: 5,
            snapshot_retention: chrono::Duration::days(1),
        };

        assert!(config.auto_checkpoint);
        assert_eq!(config.checkpoint_interval, 2);
        assert_eq!(config.max_snapshots, 5);
    }

    #[tokio::test]
    async fn test_suspendable_workflow_step_trait() {
        let step = EnhancedMemoryRetrievalStep;
        let context = WorkflowContext::new(5);

        // Test can_suspend
        let can_suspend = step.can_suspend(&context).await;
        assert!(can_suspend);

        // Test suspend_points
        let suspend_points = step.suspend_points();
        assert!(suspend_points.contains(&"external_memory_unavailable".to_string()));
        assert!(suspend_points.contains(&"rate_limit_exceeded".to_string()));

        // Test capture_state with no messages
        let state = step.capture_state(&context).await.unwrap();
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_snapshot_cleanup() {
        let temp_dir = tempdir().unwrap();
        let storage = FileSnapshotStorage::new(temp_dir.path());

        // Create multiple snapshots
        let old_time = Utc::now() - chrono::Duration::days(2);
        for i in 0..3 {
            let snapshot = WorkflowSnapshot {
                id: Uuid::new_v4(),
                created_at: if i == 0 { old_time } else { Utc::now() },
                context: WorkflowContext::new(5),
                current_step: 0,
                suspend_reason: SuspendReason::Manual,
                metadata: HashMap::new(),
                step_state: HashMap::new(),
            };

            storage.store_snapshot(&snapshot).await.unwrap();
        }

        // Test cleanup
        let cleaned = storage
            .cleanup_old_snapshots(chrono::Duration::days(1))
            .await
            .unwrap();
        assert_eq!(cleaned, 1); // Should clean up 1 old snapshot

        let remaining = storage.list_snapshots(None).await.unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn test_event_bus() {
        let event_bus = EventBus::new(10);

        // Test event sending with no subscribers
        let event = WorkflowEvent {
            id: "test-1".to_string(),
            event_type: "test_type".to_string(),
            payload: serde_json::json!({"message": "hello"}),
            timestamp: Utc::now(),
            target_workflow_id: None,
        };

        let sent_count = event_bus.send_event(event.clone()).unwrap();
        assert_eq!(sent_count, 0); // No subscribers

        // Test subscribing and sending
        let mut receiver = event_bus.subscribe("test_type");

        let sent_count = event_bus.send_event(event.clone()).unwrap();
        assert_eq!(sent_count, 1); // One subscriber

        // Test receiving event
        let received_event = receiver.recv().await.unwrap();
        assert_eq!(received_event.id, "test-1");
        assert_eq!(received_event.event_type, "test_type");
    }

    #[tokio::test]
    async fn test_sleep_step() {
        let step = SleepStep::new(100);
        let mut context = WorkflowContext::new(5);

        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Sleep(100)));

        // Test suspend capability
        assert!(step.can_suspend(&context).await);

        // Test state capture
        let state = step.capture_state(&context).await.unwrap();
        assert!(state.is_some());
        let state_value = state.unwrap();
        assert_eq!(state_value["duration_ms"], 100);
        assert_eq!(state_value["step_type"], "sleep");
    }

    #[tokio::test]
    async fn test_sleep_until_step() {
        let future_time = Utc::now() + chrono::Duration::minutes(1);
        let step = SleepUntilStep::new(future_time);
        let mut context = WorkflowContext::new(5);

        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::SleepUntil(_)));

        if let WorkflowDecision::SleepUntil(timestamp) = decision {
            assert_eq!(timestamp, future_time);
        }
    }

    #[tokio::test]
    async fn test_wait_for_event_step() {
        let step = WaitForEventStep::new("test_event".to_string(), Some(5000));
        let mut context = WorkflowContext::new(5);

        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::WaitForEvent { .. }));

        if let WorkflowDecision::WaitForEvent {
            event_id,
            timeout_ms,
        } = decision
        {
            assert_eq!(event_id, "test_event");
            assert_eq!(timeout_ms, Some(5000));
        }
    }

    #[tokio::test]
    async fn test_conditional_pause_step() {
        let step = ConditionalPauseStep::new("should_pause".to_string(), PauseType::Sleep(200));

        let mut context = WorkflowContext::new(5);

        // Test condition not met
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // Test condition met
        context
            .metadata
            .insert("should_pause".to_string(), "true".to_string());
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Sleep(200)));
    }

    #[tokio::test]
    async fn test_workflow_engine_pause_methods() {
        let engine = WorkflowEngine::new();

        // Test sleep
        let start = Instant::now();
        engine.sleep(50).await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 45); // Allow some variance

        // Test sleep_until with past timestamp (should not sleep)
        let past_time = Utc::now() - chrono::Duration::minutes(1);
        let start = Instant::now();
        engine.sleep_until(past_time).await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 10); // Should be very fast

        // Test event sending
        let sent_count = engine
            .send_simple_event("test", "test_type", serde_json::json!({"data": "test"}))
            .unwrap();
        assert_eq!(sent_count, 0); // No subscribers
    }

    #[tokio::test]
    async fn test_workflow_with_sleep_step() {
        let temp_dir = tempdir().unwrap();
        let storage = FileSnapshotStorage::new(temp_dir.path());

        let engine = WorkflowEngine::new()
            .with_suspend_config(WorkflowSuspendConfig::default())
            .with_snapshot_storage(Box::new(storage))
            .add_step(Box::new(SleepStep::new(10))); // Short sleep

        let mut context = WorkflowContext::new(10);
        context.add_message(user_message("Test message"));

        // Execute workflow - should suspend for sleep
        let result = engine.execute(context).await.unwrap();
        assert!(!result.completed);
        assert!(result.response.contains("Sleeping for 10ms"));

        // Verify snapshot was created
        let snapshots = engine.list_snapshots(None).await.unwrap();
        assert!(!snapshots.is_empty());

        let snapshot = &snapshots[0];
        assert!(matches!(
            snapshot.suspend_reason,
            SuspendReason::Sleep { .. }
        ));
    }

    #[tokio::test]
    async fn test_workflow_with_event_step() {
        let temp_dir = tempdir().unwrap();
        let storage = FileSnapshotStorage::new(temp_dir.path());
        let event_bus = Arc::new(EventBus::new(10));

        let engine = WorkflowEngine::new()
            .with_suspend_config(WorkflowSuspendConfig::default())
            .with_snapshot_storage(Box::new(storage))
            .with_event_bus(Arc::clone(&event_bus))
            .add_step(Box::new(WaitForEventStep::new(
                "user_action".to_string(),
                Some(1000),
            )));

        let mut context = WorkflowContext::new(10);
        context.add_message(user_message("Waiting for user action"));

        // Execute workflow - should suspend for event
        let result = engine.execute(context).await.unwrap();
        assert!(!result.completed);
        assert!(result.response.contains("Waiting for event 'user_action'"));

        // Verify suspend reason
        let snapshots = engine.list_snapshots(None).await.unwrap();
        assert!(!snapshots.is_empty());

        let snapshot = &snapshots[0];
        assert!(matches!(
            snapshot.suspend_reason,
            SuspendReason::WaitingForEvent { .. }
        ));
    }

    #[test]
    fn test_suspend_reason_serialization() {
        let sleep_reason = SuspendReason::Sleep {
            duration_ms: 1000,
            started_at: Utc::now(),
        };

        let json = serde_json::to_string(&sleep_reason).unwrap();
        let deserialized: SuspendReason = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, SuspendReason::Sleep { .. }));

        if let SuspendReason::Sleep { duration_ms, .. } = deserialized {
            assert_eq!(duration_ms, 1000);
        }
    }

    #[test]
    fn test_workflow_event_serialization() {
        let event = WorkflowEvent {
            id: "test-event".to_string(),
            event_type: "user_action".to_string(),
            payload: serde_json::json!({"action": "click", "target": "button"}),
            timestamp: Utc::now(),
            target_workflow_id: Some("workflow-123".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WorkflowEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.payload["action"], "click");
        assert_eq!(event.target_workflow_id, deserialized.target_workflow_id);
    }

    #[test]
    fn test_step_schema() {
        let schema = StepSchema::new_object()
            .add_property("name", "string")
            .add_property("age", "number")
            .add_required("name");

        // Valid data
        let valid_data = serde_json::json!({"name": "John", "age": 30});
        assert!(schema.validates(&valid_data));

        // Invalid data (missing required field)
        let invalid_data = serde_json::json!({"age": 30});
        assert!(!schema.validates(&invalid_data));

        // Wrong type
        let wrong_type = serde_json::json!("just a string");
        assert!(!schema.validates(&wrong_type));
    }

    #[tokio::test]
    async fn test_workflow_builder_sequential() {
        let workflow = WorkflowBuilder::new("test_workflow")
            .then(Box::new(SleepStep::new(10)))
            .then(Box::new(SleepStep::new(20)))
            .build();

        let context = WorkflowContext::new(10);
        let result = workflow.execute(context).await.unwrap();

        // Should complete or be suspended depending on implementation
        assert!(result.steps_executed > 0);
    }

    #[tokio::test]
    async fn test_parallel_execution_step() {
        let steps: Vec<Box<dyn WorkflowStep + Send + Sync>> =
            vec![Box::new(SleepStep::new(10)), Box::new(SleepStep::new(20))];

        let parallel_step = ParallelExecutionStep::new(steps);
        let mut context = WorkflowContext::new(10);

        let decision = parallel_step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));
    }

    #[tokio::test]
    async fn test_branch_execution_step() {
        // Condition that returns true if metadata contains "condition" = "true"
        let condition: ConditionFn =
            Arc::new(|context, _| context.metadata.get("condition") == Some(&"true".to_string()));

        let branch_step = BranchExecutionStep::new(
            condition,
            Box::new(SleepStep::new(10)),       // if_true
            Some(Box::new(SleepStep::new(20))), // if_false
        );

        let mut context = WorkflowContext::new(10);

        // Test false condition
        let decision = branch_step.execute(&mut context).await.unwrap();
        // Should execute if_false branch (sleep 20ms) but continue
        assert!(matches!(decision, WorkflowDecision::Sleep(20)));

        // Test true condition
        context
            .metadata
            .insert("condition".to_string(), "true".to_string());
        let decision = branch_step.execute(&mut context).await.unwrap();
        // Should execute if_true branch (sleep 10ms)
        assert!(matches!(decision, WorkflowDecision::Sleep(10)));
    }

    #[tokio::test]
    async fn test_dowhile_loop_execution() {
        let condition: ConditionFn = Arc::new(move |context, _| {
            let current_count = context
                .metadata
                .get("counter")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            current_count < 3
        });

        // Create a step that increments a counter
        let increment_step = MapExecutionStep::new(Arc::new(|context, _| {
            let current_count = context
                .metadata
                .get("counter")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let new_count = current_count + 1;
            // This would normally update context, but we'll simulate it
            serde_json::json!({ "counter": new_count })
        }));

        let loop_step =
            LoopExecutionStep::new(Box::new(increment_step), condition, LoopType::DoWhile);

        let mut context = WorkflowContext::new(10);
        context
            .metadata
            .insert("counter".to_string(), "0".to_string());

        let decision = loop_step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));
    }

    #[tokio::test]
    async fn test_foreach_execution_step() {
        // Extract items from context metadata
        let items_extractor: ItemsExtractorFn = Arc::new(|_context| {
            vec![
                serde_json::json!("item1"),
                serde_json::json!("item2"),
                serde_json::json!("item3"),
            ]
        });

        let foreach_step = ForEachExecutionStep::new(
            Box::new(SleepStep::new(5)), // Process each item with a small sleep
            items_extractor,
        );

        let mut context = WorkflowContext::new(10);
        let decision = foreach_step.execute(&mut context).await.unwrap();

        assert!(matches!(decision, WorkflowDecision::Continue));
    }

    #[tokio::test]
    async fn test_map_execution_step() {
        let mapper: MapperFn = Arc::new(|_context, input_data| {
            // Transform input by adding a "processed" field
            let mut result = input_data.clone();
            result["processed"] = serde_json::json!(true);
            result
        });

        let map_step = MapExecutionStep::new(mapper);
        let mut context = WorkflowContext::new(10);

        // Set some initial data
        context.metadata.insert(
            "last_result".to_string(),
            serde_json::to_string(&serde_json::json!({"data": "test"})).unwrap(),
        );

        let decision = map_step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));

        // Check that data was transformed
        let result = context.metadata.get("last_result").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(result).unwrap();
        assert_eq!(parsed["processed"], true);
        assert_eq!(parsed["data"], "test");
    }

    #[test]
    fn test_workflow_builder_fluent_api() {
        let _condition: ConditionFn =
            Arc::new(|context, _| context.metadata.contains_key("should_branch"));

        let workflow_builder = WorkflowBuilder::new("fluent_test")
            .with_input_schema(StepSchema::new_object())
            .with_output_schema(StepSchema::new_object())
            .with_concurrency_limit(5)
            .with_initial_data(serde_json::json!({"start": true}));

        assert_eq!(workflow_builder.id, "fluent_test");
        assert!(workflow_builder.input_schema.is_some());
        assert!(workflow_builder.output_schema.is_some());
        assert_eq!(workflow_builder.concurrency_limit, Some(5));
        assert!(workflow_builder.get_init_data().is_some());
    }

    #[test]
    fn test_workflow_cloning() {
        let original = WorkflowBuilder::new("original")
            .with_input_schema(StepSchema::new_object().add_property("test", "string"))
            .with_initial_data(serde_json::json!({"cloned": false}));

        let cloned = original.clone_workflow("cloned");

        assert_eq!(cloned.id, "cloned");
        assert_eq!(original.id, "original");
        assert!(cloned.input_schema.is_some());
        assert!(cloned.initial_data.is_some());
    }

    #[test]
    fn test_step_result_serialization() {
        let result = StepResult {
            data: serde_json::json!({"success": true, "value": 42}),
            step_name: "test_step".to_string(),
            metadata: HashMap::new(),
            execution_time_ms: 150,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: StepResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.step_name, deserialized.step_name);
        assert_eq!(result.execution_time_ms, deserialized.execution_time_ms);
        assert_eq!(result.data["success"], true);
        assert_eq!(result.data["value"], 42);
    }
}
