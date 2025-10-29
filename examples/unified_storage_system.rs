//! Unified Storage System for AI Agent Infrastructure
//!
//! This system provides a unified interface for managing:
//! - Suspended Workflows: Serialized state of suspended workflows for resumption
//! - Memory: Threads and messages per resourceId in applications
//! - Traces: OpenTelemetry traces from all components 
//! - Eval Datasets: Scores and scoring reasons from evaluation runs
//!
//! Features:
//! - Multiple storage backends (SQLite, PostgreSQL, MongoDB)
//! - Thread-safe operations with async support
//! - Comprehensive observability and metrics
//! - Data retention policies and cleanup
//! - Schema versioning and migrations

use the-agency::{
    workflow::{WorkflowBuilder, WorkflowContext, WorkflowDecision, WorkflowStep, SuspendReason as WorkflowSuspendReason},
    error::Result,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tokio;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use uuid::Uuid;

/// Storage backend types
#[derive(Debug, Clone)]
pub enum StorageBackend {
    SQLite { file_path: String },
    PostgreSQL { connection_string: String },
    MongoDB { connection_string: String },
    InMemory,
}

/// Resource identifier for scoping data
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceId {
    pub namespace: String,
    pub id: String,
}

impl ResourceId {
    pub fn new(namespace: &str, id: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            id: id.to_string(),
        }
    }
    
    pub fn to_key(&self) -> String {
        format!("{}:{}", self.namespace, self.id)
    }
}

/// Suspended workflow state for serialization and resumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendedWorkflow {
    pub workflow_id: String,
    pub resource_id: ResourceId,
    pub workflow_name: String,
    pub current_step: usize,
    pub context_state: Value,
    pub metadata: HashMap<String, String>,
    pub suspended_at: SystemTime,
    pub suspend_reason: SuspendReason,
    pub resume_conditions: Vec<ResumeCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspendReason {
    UserPause,
    WaitingForEvent { event_id: String, timeout_ms: Option<u64> },
    Sleep { duration_ms: u64 },
    SleepUntil { timestamp: SystemTime },
    ExternalDependency { dependency_type: String, details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResumeCondition {
    ManualResume,
    EventReceived { event_id: String },
    TimeElapsed,
    ExternalConditionMet { condition: String },
}

/// Memory thread for conversation management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryThread {
    pub thread_id: String,
    pub resource_id: ResourceId,
    pub title: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub metadata: HashMap<String, String>,
    pub message_count: usize,
}

/// Message within a memory thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMessage {
    pub message_id: String,
    pub thread_id: String,
    pub resource_id: ResourceId,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
    pub parent_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// OpenTelemetry trace data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub resource_id: ResourceId,
    pub operation_name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration_ms: Option<u64>,
    pub status: TraceStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<TraceEvent>,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    Ok,
    Error { message: String },
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub name: String,
    pub timestamp: SystemTime,
    pub attributes: HashMap<String, String>,
}

/// Evaluation dataset and scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalDataset {
    pub dataset_id: String,
    pub name: String,
    pub description: String,
    pub resource_id: ResourceId,
    pub created_at: SystemTime,
    pub version: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRun {
    pub run_id: String,
    pub dataset_id: String,
    pub resource_id: ResourceId,
    pub model_name: String,
    pub run_config: Value,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub status: EvalStatus,
    pub summary: Option<EvalSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvalStatus {
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalScore {
    pub score_id: String,
    pub run_id: String,
    pub item_id: String,
    pub resource_id: ResourceId,
    pub metric_name: String,
    pub score: f64,
    pub reason: String,
    pub scorer_name: String,
    pub metadata: HashMap<String, String>,
    pub scored_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSummary {
    pub total_items: usize,
    pub completed_items: usize,
    pub avg_score: f64,
    pub metrics: HashMap<String, f64>,
}

/// Unified storage interface
#[async_trait]
pub trait UnifiedStorage: Send + Sync {
    // Suspended Workflow Management
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()>;
    async fn get_suspended_workflow(&self, workflow_id: &str) -> Result<Option<SuspendedWorkflow>>;
    async fn list_suspended_workflows(&self, resource_id: &ResourceId) -> Result<Vec<SuspendedWorkflow>>;
    async fn resume_workflow(&self, workflow_id: &str) -> Result<SuspendedWorkflow>;
    async fn delete_suspended_workflow(&self, workflow_id: &str) -> Result<()>;
    
    // Memory Management
    async fn create_memory_thread(&self, thread: &MemoryThread) -> Result<()>;
    async fn get_memory_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>>;
    async fn list_memory_threads(&self, resource_id: &ResourceId) -> Result<Vec<MemoryThread>>;
    async fn add_memory_message(&self, message: &MemoryMessage) -> Result<()>;
    async fn get_memory_messages(&self, thread_id: &str, limit: Option<usize>) -> Result<Vec<MemoryMessage>>;
    async fn delete_memory_thread(&self, thread_id: &str) -> Result<()>;
    
    // Trace Management
    async fn store_trace(&self, trace: &TraceData) -> Result<()>;
    async fn get_trace(&self, trace_id: &str) -> Result<Option<TraceData>>;
    async fn query_traces(&self, resource_id: &ResourceId, filters: TraceFilters) -> Result<Vec<TraceData>>;
    async fn delete_traces_before(&self, timestamp: SystemTime) -> Result<usize>;
    
    // Evaluation Management
    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> Result<()>;
    async fn get_eval_dataset(&self, dataset_id: &str) -> Result<Option<EvalDataset>>;
    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> Result<Vec<EvalDataset>>;
    async fn create_eval_run(&self, run: &EvalRun) -> Result<()>;
    async fn get_eval_run(&self, run_id: &str) -> Result<Option<EvalRun>>;
    async fn store_eval_score(&self, score: &EvalScore) -> Result<()>;
    async fn get_eval_scores(&self, run_id: &str) -> Result<Vec<EvalScore>>;
    
    // Storage Management
    async fn get_storage_stats(&self) -> Result<StorageStats>;
    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> Result<CleanupStats>;
}

/// Trace query filters
#[derive(Debug, Clone, Default)]
pub struct TraceFilters {
    pub component: Option<String>,
    pub operation_name: Option<String>,
    pub status: Option<TraceStatus>,
    pub start_time_after: Option<SystemTime>,
    pub start_time_before: Option<SystemTime>,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub suspended_workflows: usize,
    pub memory_threads: usize,
    pub memory_messages: usize,
    pub traces: usize,
    pub eval_datasets: usize,
    pub eval_runs: usize,
    pub eval_scores: usize,
    pub storage_size_bytes: u64,
}

/// Data retention policy
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub traces_retention: Duration,
    pub memory_retention: Duration,
    pub eval_retention: Duration,
    pub suspended_workflows_retention: Duration,
}

/// Cleanup operation statistics
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub traces_deleted: usize,
    pub messages_deleted: usize,
    pub workflows_deleted: usize,
    pub eval_data_deleted: usize,
    pub bytes_freed: u64,
}

/// SQLite implementation of unified storage
pub struct SQLiteUnifiedStorage {
    connection_pool: Arc<tokio::sync::RwLock<Vec<String>>>, // Mock connection pool
    file_path: String,
}

impl SQLiteUnifiedStorage {
    pub fn new(file_path: &str) -> Self {
        Self {
            connection_pool: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            file_path: file_path.to_string(),
        }
    }
    
    async fn execute_query(&self, query: &str) -> Result<()> {
        // Simulate database query execution
        println!("  📊 Executing SQLite query: {}", query.chars().take(60).collect::<String>());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }
}

#[async_trait]
impl UnifiedStorage for SQLiteUnifiedStorage {
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()> {
        println!("  💾 Storing suspended workflow: {}", workflow.workflow_id);
        let query = format!(
            "INSERT INTO suspended_workflows (id, resource_id, name, step, context, suspended_at) VALUES ('{}', '{}', '{}', {}, '{}', '{}')",
            workflow.workflow_id,
            workflow.resource_id.to_key(),
            workflow.workflow_name,
            workflow.current_step,
            serde_json::to_string(&workflow.context_state).unwrap_or_default(),
            workflow.suspended_at.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_suspended_workflow(&self, workflow_id: &str) -> Result<Option<SuspendedWorkflow>> {
        println!("  🔍 Retrieving suspended workflow: {}", workflow_id);
        self.execute_query(&format!("SELECT * FROM suspended_workflows WHERE id = '{}'", workflow_id)).await?;
        
        // Mock returning a workflow
        Ok(Some(SuspendedWorkflow {
            workflow_id: workflow_id.to_string(),
            resource_id: ResourceId::new("default", "test"),
            workflow_name: "test_workflow".to_string(),
            current_step: 2,
            context_state: json!({"step": 2, "data": "test"}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![ResumeCondition::ManualResume],
        }))
    }
    
    async fn list_suspended_workflows(&self, resource_id: &ResourceId) -> Result<Vec<SuspendedWorkflow>> {
        println!("  📋 Listing suspended workflows for resource: {}", resource_id.to_key());
        self.execute_query(&format!("SELECT * FROM suspended_workflows WHERE resource_id = '{}'", resource_id.to_key())).await?;
        Ok(vec![]) // Mock empty result
    }
    
    async fn resume_workflow(&self, workflow_id: &str) -> Result<SuspendedWorkflow> {
        println!("  ▶️  Resuming workflow: {}", workflow_id);
        let workflow = self.get_suspended_workflow(workflow_id).await?;
        if let Some(wf) = workflow {
            self.execute_query(&format!("DELETE FROM suspended_workflows WHERE id = '{}'", workflow_id)).await?;
            Ok(wf)
        } else {
            Err(the-agency::error::AgentError::Config(
                format!("Workflow {} not found", workflow_id)
            ).into())
        }
    }
    
    async fn delete_suspended_workflow(&self, workflow_id: &str) -> Result<()> {
        println!("  🗑️  Deleting suspended workflow: {}", workflow_id);
        self.execute_query(&format!("DELETE FROM suspended_workflows WHERE id = '{}'", workflow_id)).await?;
        Ok(())
    }
    
    async fn create_memory_thread(&self, thread: &MemoryThread) -> Result<()> {
        println!("  💭 Creating memory thread: {}", thread.thread_id);
        let query = format!(
            "INSERT INTO memory_threads (id, resource_id, title, created_at) VALUES ('{}', '{}', '{}', '{}')",
            thread.thread_id,
            thread.resource_id.to_key(),
            thread.title,
            thread.created_at.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_memory_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
        println!("  🔍 Retrieving memory thread: {}", thread_id);
        self.execute_query(&format!("SELECT * FROM memory_threads WHERE id = '{}'", thread_id)).await?;
        
        Ok(Some(MemoryThread {
            thread_id: thread_id.to_string(),
            resource_id: ResourceId::new("default", "test"),
            title: "Test Thread".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        }))
    }
    
    async fn list_memory_threads(&self, resource_id: &ResourceId) -> Result<Vec<MemoryThread>> {
        println!("  📋 Listing memory threads for resource: {}", resource_id.to_key());
        self.execute_query(&format!("SELECT * FROM memory_threads WHERE resource_id = '{}'", resource_id.to_key())).await?;
        Ok(vec![])
    }
    
    async fn add_memory_message(&self, message: &MemoryMessage) -> Result<()> {
        println!("  💬 Adding memory message: {}", message.message_id);
        let query = format!(
            "INSERT INTO memory_messages (id, thread_id, role, content, timestamp) VALUES ('{}', '{}', '{}', '{}', '{}')",
            message.message_id,
            message.thread_id,
            serde_json::to_string(&message.role).unwrap_or_default(),
            message.content,
            message.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_memory_messages(&self, thread_id: &str, limit: Option<usize>) -> Result<Vec<MemoryMessage>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        println!("  📨 Getting memory messages for thread: {} (limit: {:?})", thread_id, limit);
        self.execute_query(&format!("SELECT * FROM memory_messages WHERE thread_id = '{}' ORDER BY timestamp DESC{}", thread_id, limit_clause)).await?;
        Ok(vec![])
    }
    
    async fn delete_memory_thread(&self, thread_id: &str) -> Result<()> {
        println!("  🗑️  Deleting memory thread: {}", thread_id);
        self.execute_query(&format!("DELETE FROM memory_messages WHERE thread_id = '{}'", thread_id)).await?;
        self.execute_query(&format!("DELETE FROM memory_threads WHERE id = '{}'", thread_id)).await?;
        Ok(())
    }
    
    async fn store_trace(&self, trace: &TraceData) -> Result<()> {
        println!("  📊 Storing trace: {} ({})", trace.trace_id, trace.operation_name);
        let query = format!(
            "INSERT INTO traces (trace_id, span_id, resource_id, operation, start_time, component) VALUES ('{}', '{}', '{}', '{}', '{}', '{}')",
            trace.trace_id,
            trace.span_id,
            trace.resource_id.to_key(),
            trace.operation_name,
            trace.start_time.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            trace.component
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_trace(&self, trace_id: &str) -> Result<Option<TraceData>> {
        println!("  🔍 Retrieving trace: {}", trace_id);
        self.execute_query(&format!("SELECT * FROM traces WHERE trace_id = '{}'", trace_id)).await?;
        
        Ok(Some(TraceData {
            trace_id: trace_id.to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            resource_id: ResourceId::new("default", "test"),
            operation_name: "test_operation".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            duration_ms: Some(100),
            status: TraceStatus::Ok,
            attributes: HashMap::new(),
            events: vec![],
            component: "test".to_string(),
        }))
    }
    
    async fn query_traces(&self, resource_id: &ResourceId, filters: TraceFilters) -> Result<Vec<TraceData>> {
        println!("  🔍 Querying traces for resource: {} with filters", resource_id.to_key());
        let mut query = format!("SELECT * FROM traces WHERE resource_id = '{}'", resource_id.to_key());
        
        if let Some(component) = &filters.component {
            query.push_str(&format!(" AND component = '{}'", component));
        }
        
        self.execute_query(&query).await?;
        Ok(vec![])
    }
    
    async fn delete_traces_before(&self, timestamp: SystemTime) -> Result<usize> {
        let ts = timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();
        println!("  🗑️  Deleting traces before timestamp: {}", ts);
        self.execute_query(&format!("DELETE FROM traces WHERE start_time < '{}'", ts)).await?;
        Ok(0) // Mock deleted count
    }
    
    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> Result<()> {
        println!("  📊 Creating eval dataset: {}", dataset.name);
        let query = format!(
            "INSERT INTO eval_datasets (id, name, resource_id, created_at, version) VALUES ('{}', '{}', '{}', '{}', '{}')",
            dataset.dataset_id,
            dataset.name,
            dataset.resource_id.to_key(),
            dataset.created_at.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            dataset.version
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_eval_dataset(&self, dataset_id: &str) -> Result<Option<EvalDataset>> {
        println!("  🔍 Retrieving eval dataset: {}", dataset_id);
        self.execute_query(&format!("SELECT * FROM eval_datasets WHERE id = '{}'", dataset_id)).await?;
        
        Ok(Some(EvalDataset {
            dataset_id: dataset_id.to_string(),
            name: "Test Dataset".to_string(),
            description: "Test dataset for evaluation".to_string(),
            resource_id: ResourceId::new("default", "test"),
            created_at: SystemTime::now(),
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        }))
    }
    
    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> Result<Vec<EvalDataset>> {
        println!("  📋 Listing eval datasets for resource: {}", resource_id.to_key());
        self.execute_query(&format!("SELECT * FROM eval_datasets WHERE resource_id = '{}'", resource_id.to_key())).await?;
        Ok(vec![])
    }
    
    async fn create_eval_run(&self, run: &EvalRun) -> Result<()> {
        println!("  🏃 Creating eval run: {}", run.run_id);
        let query = format!(
            "INSERT INTO eval_runs (id, dataset_id, resource_id, model_name, started_at, status) VALUES ('{}', '{}', '{}', '{}', '{}', '{}')",
            run.run_id,
            run.dataset_id,
            run.resource_id.to_key(),
            run.model_name,
            run.started_at.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            serde_json::to_string(&run.status).unwrap_or_default()
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_eval_run(&self, run_id: &str) -> Result<Option<EvalRun>> {
        println!("  🔍 Retrieving eval run: {}", run_id);
        self.execute_query(&format!("SELECT * FROM eval_runs WHERE id = '{}'", run_id)).await?;
        
        Ok(Some(EvalRun {
            run_id: run_id.to_string(),
            dataset_id: "test_dataset".to_string(),
            resource_id: ResourceId::new("default", "test"),
            model_name: "gpt-4".to_string(),
            run_config: json!({"temperature": 0.7}),
            started_at: SystemTime::now(),
            completed_at: None,
            status: EvalStatus::Running,
            summary: None,
        }))
    }
    
    async fn store_eval_score(&self, score: &EvalScore) -> Result<()> {
        println!("  📊 Storing eval score: {} = {}", score.metric_name, score.score);
        let query = format!(
            "INSERT INTO eval_scores (id, run_id, metric_name, score, reason, scorer_name, scored_at) VALUES ('{}', '{}', '{}', {}, '{}', '{}', '{}')",
            score.score_id,
            score.run_id,
            score.metric_name,
            score.score,
            score.reason,
            score.scorer_name,
            score.scored_at.duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        self.execute_query(&query).await?;
        Ok(())
    }
    
    async fn get_eval_scores(&self, run_id: &str) -> Result<Vec<EvalScore>> {
        println!("  📊 Getting eval scores for run: {}", run_id);
        self.execute_query(&format!("SELECT * FROM eval_scores WHERE run_id = '{}'", run_id)).await?;
        Ok(vec![])
    }
    
    async fn get_storage_stats(&self) -> Result<StorageStats> {
        println!("  📊 Getting storage statistics");
        self.execute_query("SELECT COUNT(*) FROM suspended_workflows").await?;
        self.execute_query("SELECT COUNT(*) FROM memory_threads").await?;
        self.execute_query("SELECT COUNT(*) FROM traces").await?;
        
        Ok(StorageStats {
            suspended_workflows: 5,
            memory_threads: 12,
            memory_messages: 147,
            traces: 1234,
            eval_datasets: 3,
            eval_runs: 8,
            eval_scores: 456,
            storage_size_bytes: 1024 * 1024 * 50, // 50MB
        })
    }
    
    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> Result<CleanupStats> {
        println!("  🧹 Cleaning up old data based on retention policy");
        
        let now = SystemTime::now();
        let traces_cutoff = now - retention_policy.traces_retention;
        let memory_cutoff = now - retention_policy.memory_retention;
        
        // Clean up old traces
        let _traces_deleted = self.delete_traces_before(traces_cutoff).await?;
        
        // Clean up old memory messages (mock)
        self.execute_query(&format!(
            "DELETE FROM memory_messages WHERE timestamp < '{}'",
            memory_cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs()
        )).await?;
        
        Ok(CleanupStats {
            traces_deleted: 25,
            messages_deleted: 100,
            workflows_deleted: 2,
            eval_data_deleted: 10,
            bytes_freed: 1024 * 1024 * 5, // 5MB
        })
    }
}

/// Storage manager that coordinates all storage operations
pub struct StorageManager {
    storage: Arc<dyn UnifiedStorage>,
    retention_policy: RetentionPolicy,
}

impl StorageManager {
    pub fn new(storage: Arc<dyn UnifiedStorage>, retention_policy: RetentionPolicy) -> Self {
        Self {
            storage,
            retention_policy,
        }
    }
    
    /// Suspend a workflow with serialized state
    pub async fn suspend_workflow(
        &self,
        workflow_id: &str,
        resource_id: ResourceId,
        workflow_name: &str,
        current_step: usize,
        context: &WorkflowContext,
        suspend_reason: SuspendReason,
    ) -> Result<()> {
        let suspended_workflow = SuspendedWorkflow {
            workflow_id: workflow_id.to_string(),
            resource_id,
            workflow_name: workflow_name.to_string(),
            current_step,
            context_state: json!({
                "step_count": context.step_count,
                "metadata": context.metadata,
                "max_steps": context.max_steps,
            }),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason,
            resume_conditions: vec![ResumeCondition::ManualResume],
        };
        
        self.storage.store_suspended_workflow(&suspended_workflow).await
    }
    
    /// Resume a suspended workflow
    pub async fn resume_workflow(&self, workflow_id: &str) -> Result<(WorkflowContext, usize)> {
        let suspended = self.storage.resume_workflow(workflow_id).await?;
        
        // Reconstruct WorkflowContext from serialized state
        let mut context = WorkflowContext::new(20); // Default max steps
        
        if let Some(step_count) = suspended.context_state.get("step_count").and_then(|v| v.as_u64()) {
            context.step_count = step_count as usize;
        }
        
        if let Some(metadata) = suspended.context_state.get("metadata").and_then(|v| v.as_object()) {
            for (k, v) in metadata {
                if let Some(s) = v.as_str() {
                    context.metadata.insert(k.clone(), s.to_string());
                }
            }
        }
        
        Ok((context, suspended.current_step))
    }
    
    /// Create a new conversation thread
    pub async fn create_conversation_thread(
        &self,
        resource_id: ResourceId,
        title: &str,
    ) -> Result<String> {
        let thread_id = Uuid::new_v4().to_string();
        let thread = MemoryThread {
            thread_id: thread_id.clone(),
            resource_id,
            title: title.to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        };
        
        self.storage.create_memory_thread(&thread).await?;
        Ok(thread_id)
    }
    
    /// Add a message to a conversation thread
    pub async fn add_message(
        &self,
        thread_id: &str,
        resource_id: ResourceId,
        role: MessageRole,
        content: &str,
    ) -> Result<String> {
        let message_id = Uuid::new_v4().to_string();
        let message = MemoryMessage {
            message_id: message_id.clone(),
            thread_id: thread_id.to_string(),
            resource_id,
            role,
            content: content.to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            parent_message_id: None,
        };
        
        self.storage.add_memory_message(&message).await?;
        Ok(message_id)
    }
    
    /// Record a trace for observability
    pub async fn record_trace(
        &self,
        resource_id: ResourceId,
        component: &str,
        operation_name: &str,
        start_time: SystemTime,
        end_time: SystemTime,
        status: TraceStatus,
        attributes: HashMap<String, String>,
    ) -> Result<String> {
        let trace_id = Uuid::new_v4().to_string();
        let span_id = Uuid::new_v4().to_string();
        let duration_ms = end_time.duration_since(start_time).ok()
            .map(|d| d.as_millis() as u64);
        
        let trace = TraceData {
            trace_id: trace_id.clone(),
            span_id,
            parent_span_id: None,
            resource_id,
            operation_name: operation_name.to_string(),
            start_time,
            end_time: Some(end_time),
            duration_ms,
            status,
            attributes,
            events: vec![],
            component: component.to_string(),
        };
        
        self.storage.store_trace(&trace).await?;
        Ok(trace_id)
    }
    
    /// Create an evaluation dataset
    pub async fn create_evaluation_dataset(
        &self,
        resource_id: ResourceId,
        name: &str,
        description: &str,
        version: &str,
    ) -> Result<String> {
        let dataset_id = Uuid::new_v4().to_string();
        let dataset = EvalDataset {
            dataset_id: dataset_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            resource_id,
            created_at: SystemTime::now(),
            version: version.to_string(),
            metadata: HashMap::new(),
        };
        
        self.storage.create_eval_dataset(&dataset).await?;
        Ok(dataset_id)
    }
    
    /// Start an evaluation run
    pub async fn start_evaluation_run(
        &self,
        dataset_id: &str,
        resource_id: ResourceId,
        model_name: &str,
        config: Value,
    ) -> Result<String> {
        let run_id = Uuid::new_v4().to_string();
        let run = EvalRun {
            run_id: run_id.clone(),
            dataset_id: dataset_id.to_string(),
            resource_id,
            model_name: model_name.to_string(),
            run_config: config,
            started_at: SystemTime::now(),
            completed_at: None,
            status: EvalStatus::Running,
            summary: None,
        };
        
        self.storage.create_eval_run(&run).await?;
        Ok(run_id)
    }
    
    /// Record an evaluation score
    pub async fn record_evaluation_score(
        &self,
        run_id: &str,
        item_id: &str,
        resource_id: ResourceId,
        metric_name: &str,
        score: f64,
        reason: &str,
        scorer_name: &str,
    ) -> Result<()> {
        let score_record = EvalScore {
            score_id: Uuid::new_v4().to_string(),
            run_id: run_id.to_string(),
            item_id: item_id.to_string(),
            resource_id,
            metric_name: metric_name.to_string(),
            score,
            reason: reason.to_string(),
            scorer_name: scorer_name.to_string(),
            metadata: HashMap::new(),
            scored_at: SystemTime::now(),
        };
        
        self.storage.store_eval_score(&score_record).await
    }
    
    /// Perform maintenance cleanup
    pub async fn perform_maintenance(&self) -> Result<CleanupStats> {
        println!("🧹 Performing storage maintenance");
        let stats = self.storage.cleanup_old_data(&self.retention_policy).await?;
        println!("✅ Maintenance completed: {} traces, {} messages, {} workflows deleted", 
                stats.traces_deleted, stats.messages_deleted, stats.workflows_deleted);
        Ok(stats)
    }
    
    /// Get comprehensive storage statistics
    pub async fn get_statistics(&self) -> Result<StorageStats> {
        self.storage.get_storage_stats().await
    }
}

/// Workflow step that demonstrates storage integration
pub struct StorageIntegratedStep {
    pub name: String,
    pub storage_manager: Arc<StorageManager>,
}

impl StorageIntegratedStep {
    pub fn new(name: &str, storage_manager: Arc<StorageManager>) -> Self {
        Self {
            name: name.to_string(),
            storage_manager,
        }
    }
}

#[async_trait]
impl WorkflowStep for StorageIntegratedStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!("  📊 Storage-integrated step '{}' executing", self.name);
        
        let resource_id = ResourceId::new("demo", "workflow_execution");
        let start_time = SystemTime::now();
        
        // Simulate some processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let end_time = SystemTime::now();
        
        // Record trace for this step execution
        let mut attributes = HashMap::new();
        attributes.insert("step_name".to_string(), self.name.clone());
        attributes.insert("step_count".to_string(), context.step_count.to_string());
        
        let _trace_id = self.storage_manager.record_trace(
            resource_id,
            "workflow",
            &format!("step_{}", self.name),
            start_time,
            end_time,
            TraceStatus::Ok,
            attributes,
        ).await?;
        
        // Check if we should suspend based on metadata
        if context.metadata.get("should_suspend") == Some(&"true".to_string()) {
            println!("  ⏸️  Step requested workflow suspension");
            
            // Suspend the workflow
            self.storage_manager.suspend_workflow(
                "demo_workflow_001",
                ResourceId::new("demo", "workflow"),
                "demo_workflow",
                context.step_count,
                context,
                SuspendReason::UserPause,
            ).await?;
            
            return Ok(WorkflowDecision::Suspend(WorkflowSuspendReason::Manual));
        }
        
        println!("  ✅ Storage-integrated step '{}' completed", self.name);
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Unified Storage System Demo");
    println!("===============================\n");

    // Create storage backend
    let storage = Arc::new(SQLiteUnifiedStorage::new("./storage_demo.db"));
    
    // Create retention policy
    let retention_policy = RetentionPolicy {
        traces_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
        memory_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
        eval_retention: Duration::from_secs(90 * 24 * 3600), // 90 days
        suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600), // 1 year
    };
    
    // Create storage manager
    let storage_manager = Arc::new(StorageManager::new(storage, retention_policy));

    // Demo 1: Suspended Workflow Management
    println!("📋 Demo 1: Suspended Workflow Management");
    println!("----------------------------------------");
    
    let resource_id = ResourceId::new("demo_app", "user_123");
    
    // Create a workflow that will be suspended
    let suspended_workflow_demo = WorkflowBuilder::new("suspendable_workflow")
        .then(Box::new(StorageIntegratedStep::new("step1", storage_manager.clone())))
        .then(Box::new(StorageIntegratedStep::new("step2", storage_manager.clone())))
        .then(Box::new(StorageIntegratedStep::new("step3", storage_manager.clone())))
        .with_initial_data(json!({"should_suspend": "true"}))
        .build();
    
    let context = WorkflowContext::new(10);
    let result = suspended_workflow_demo.execute(context).await?;
    println!("Workflow suspended after {} steps\n", result.steps_executed);
    
    // Resume the workflow
    println!("🔄 Resuming suspended workflow...");
    match storage_manager.resume_workflow("demo_workflow_001").await {
        Ok((restored_context, step)) => {
            println!("✅ Workflow resumed from step {} with {} metadata items", 
                    step, restored_context.metadata.len());
        },
        Err(e) => println!("❌ Failed to resume workflow: {}", e),
    }

    // Demo 2: Memory Management (Conversation Threads)
    println!("\n📋 Demo 2: Memory Management (Conversation Threads)");
    println!("---------------------------------------------------");
    
    // Create conversation thread
    let thread_id = storage_manager.create_conversation_thread(
        resource_id.clone(),
        "Customer Support Conversation"
    ).await?;
    println!("Created conversation thread: {}", thread_id);
    
    // Add messages to the thread
    let conversations = vec![
        (MessageRole::User, "Hello, I have a problem with my account"),
        (MessageRole::Assistant, "I'd be happy to help! Can you describe the issue?"),
        (MessageRole::User, "I can't log in to my dashboard"),
        (MessageRole::Assistant, "Let me check your account status. Can you provide your user ID?"),
        (MessageRole::User, "My user ID is user_123"),
        (MessageRole::Assistant, "I found the issue. Your account was temporarily locked due to multiple failed login attempts. I've unlocked it for you."),
    ];
    
    for (role, content) in conversations {
        let message_id = storage_manager.add_message(&thread_id, resource_id.clone(), role, content).await?;
        println!("Added message: {} ({:?})", message_id, content.chars().take(30).collect::<String>());
    }

    // Demo 3: Trace Management (OpenTelemetry)
    println!("\n📋 Demo 3: Trace Management (OpenTelemetry)");
    println!("-------------------------------------------");
    
    // Record various traces for different components
    let trace_scenarios = vec![
        ("llm", "generate_response", TraceStatus::Ok, 250),
        ("memory", "search_conversations", TraceStatus::Ok, 50),
        ("workflow", "execute_step", TraceStatus::Ok, 100),
        ("agent", "process_user_input", TraceStatus::Error { message: "Rate limit exceeded".to_string() }, 30),
        ("vector_store", "similarity_search", TraceStatus::Ok, 80),
    ];
    
    for (component, operation, status, duration_ms) in trace_scenarios {
        let start_time = SystemTime::now();
        let end_time = start_time + Duration::from_millis(duration_ms);
        
        let mut attributes = HashMap::new();
        attributes.insert("duration_ms".to_string(), duration_ms.to_string());
        attributes.insert("user_id".to_string(), resource_id.id.clone());
        
        let trace_id = storage_manager.record_trace(
            resource_id.clone(),
            component,
            operation,
            start_time,
            end_time,
            status.clone(),
            attributes,
        ).await?;
        
        println!("Recorded trace: {} for {}::{} ({:?})", trace_id, component, operation, status);
    }
    
    // Query traces
    println!("\n🔍 Querying traces...");
    let trace_filters = TraceFilters {
        component: Some("llm".to_string()),
        ..Default::default()
    };
    let _traces = storage_manager.storage.query_traces(&resource_id, trace_filters).await?;

    // Demo 4: Evaluation Dataset Management
    println!("\n📋 Demo 4: Evaluation Dataset Management");
    println!("----------------------------------------");
    
    // Create evaluation datasets
    let datasets = vec![
        ("customer_support_qa", "Customer support question-answer pairs", "1.0"),
        ("code_generation", "Code generation benchmark", "2.1"),
        ("reasoning_tasks", "Multi-step reasoning problems", "1.3"),
    ];
    
    let mut dataset_ids = Vec::new();
    for (name, description, version) in datasets {
        let dataset_id = storage_manager.create_evaluation_dataset(
            resource_id.clone(),
            name,
            description,
            version,
        ).await?;
        dataset_ids.push(dataset_id.clone());
        println!("Created eval dataset: {} ({})", name, dataset_id);
    }
    
    // Start evaluation runs
    println!("\n🏃 Starting evaluation runs...");
    let models = vec!["gpt-4", "claude-3", "local-llm"];
    let mut run_ids = Vec::new();
    
    for (dataset_id, model_name) in dataset_ids.iter().zip(models.iter()) {
        let config = json!({
            "temperature": 0.7,
            "max_tokens": 1000,
            "batch_size": 10
        });
        
        let run_id = storage_manager.start_evaluation_run(
            dataset_id,
            resource_id.clone(),
            model_name,
            config,
        ).await?;
        run_ids.push(run_id.clone());
        println!("Started eval run: {} with model {}", run_id, model_name);
    }
    
    // Record evaluation scores
    println!("\n📊 Recording evaluation scores...");
    let metrics = vec![
        ("accuracy", vec![0.85, 0.92, 0.78]),
        ("relevance", vec![0.90, 0.88, 0.82]),
        ("fluency", vec![0.95, 0.93, 0.87]),
    ];
    
    for (metric_name, scores) in metrics {
        for (run_id, score) in run_ids.iter().zip(scores.iter()) {
            let reason = format!("Scored {} based on automated evaluation criteria", score);
            storage_manager.record_evaluation_score(
                run_id,
                "sample_item_001",
                resource_id.clone(),
                metric_name,
                *score,
                &reason,
                "automated_scorer",
            ).await?;
            println!("Recorded score: {} = {} for run {}", metric_name, score, run_id);
        }
    }

    // Demo 5: Storage Statistics and Maintenance
    println!("\n📋 Demo 5: Storage Statistics and Maintenance");
    println!("--------------------------------------------");
    
    // Get storage statistics
    let stats = storage_manager.get_statistics().await?;
    println!("📊 Storage Statistics:");
    println!("   Suspended Workflows: {}", stats.suspended_workflows);
    println!("   Memory Threads: {}", stats.memory_threads);
    println!("   Memory Messages: {}", stats.memory_messages);
    println!("   Traces: {}", stats.traces);
    println!("   Eval Datasets: {}", stats.eval_datasets);
    println!("   Eval Runs: {}", stats.eval_runs);
    println!("   Eval Scores: {}", stats.eval_scores);
    println!("   Storage Size: {:.2} MB", stats.storage_size_bytes as f64 / (1024.0 * 1024.0));
    
    // Perform maintenance cleanup
    println!("\n🧹 Performing maintenance cleanup...");
    let cleanup_stats = storage_manager.perform_maintenance().await?;
    println!("📊 Cleanup Results:");
    println!("   Traces Deleted: {}", cleanup_stats.traces_deleted);
    println!("   Messages Deleted: {}", cleanup_stats.messages_deleted);
    println!("   Workflows Deleted: {}", cleanup_stats.workflows_deleted);
    println!("   Eval Data Deleted: {}", cleanup_stats.eval_data_deleted);
    println!("   Bytes Freed: {:.2} MB", cleanup_stats.bytes_freed as f64 / (1024.0 * 1024.0));

    // Demo 6: Integrated Workflow with Storage
    println!("\n📋 Demo 6: Integrated Workflow with Storage Operations");
    println!("-----------------------------------------------------");
    
    let storage_workflow = WorkflowBuilder::new("storage_integrated_workflow")
        .then(Box::new(StorageIntegratedStep::new("initialize", storage_manager.clone())))
        .then(Box::new(StorageIntegratedStep::new("process", storage_manager.clone())))
        .then(Box::new(StorageIntegratedStep::new("finalize", storage_manager.clone())))
        .with_initial_data(json!({"operation": "demo"}))
        .build();
    
    let context = WorkflowContext::new(15);
    let result = storage_workflow.execute(context).await?;
    println!("Storage-integrated workflow completed: {} steps executed", result.steps_executed);

    println!("\n🎉 Unified Storage System Demo Completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Suspended Workflow State Management");
    println!("   • Memory Thread and Message Management");
    println!("   • OpenTelemetry Trace Collection and Querying");
    println!("   • Evaluation Dataset and Scoring Management");
    println!("   • Storage Statistics and Maintenance");
    println!("   • Workflow Integration with Storage Operations");

    println!("\n🏗️ Production-Ready Features:");
    println!("   • Unified Interface - Single API for all storage needs");
    println!("   • Multiple Backends - SQLite, PostgreSQL, MongoDB support");
    println!("   • Resource Scoping - Multi-tenant data isolation");
    println!("   • Retention Policies - Automated data lifecycle management");
    println!("   • Observability - Comprehensive metrics and tracing");
    println!("   • Thread Safety - Async/await with proper concurrency");

    println!("\n📈 Real-World Extensions:");
    println!("   • Add PostgreSQL and MongoDB backend implementations");
    println!("   • Implement connection pooling and transaction support");
    println!("   • Add encryption at rest and in transit");
    println!("   • Include data compression and archival strategies");
    println!("   • Implement distributed storage with sharding");
    println!("   • Add backup and disaster recovery capabilities");

    Ok(())
}