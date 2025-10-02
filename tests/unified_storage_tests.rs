//! Comprehensive unit tests for the unified storage system

use generic_ai_agent::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;
use tokio::time::sleep;

// Re-include the unified storage system components for testing
// In a real project, these would be in the main library
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use async_trait::async_trait;

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

/// Unified storage interface
#[async_trait]
pub trait UnifiedStorage: Send + Sync {
    // Suspended Workflow Management
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> error::Result<()>;
    async fn get_suspended_workflow(&self, workflow_id: &str) -> error::Result<Option<SuspendedWorkflow>>;
    async fn list_suspended_workflows(&self, resource_id: &ResourceId) -> error::Result<Vec<SuspendedWorkflow>>;
    async fn resume_workflow(&self, workflow_id: &str) -> error::Result<SuspendedWorkflow>;
    async fn delete_suspended_workflow(&self, workflow_id: &str) -> error::Result<()>;
    
    // Memory Management
    async fn create_memory_thread(&self, thread: &MemoryThread) -> error::Result<()>;
    async fn get_memory_thread(&self, thread_id: &str) -> error::Result<Option<MemoryThread>>;
    async fn list_memory_threads(&self, resource_id: &ResourceId) -> error::Result<Vec<MemoryThread>>;
    async fn add_memory_message(&self, message: &MemoryMessage) -> error::Result<()>;
    async fn get_memory_messages(&self, thread_id: &str, limit: Option<usize>) -> error::Result<Vec<MemoryMessage>>;
    async fn delete_memory_thread(&self, thread_id: &str) -> error::Result<()>;
    
    // Trace Management
    async fn store_trace(&self, trace: &TraceData) -> error::Result<()>;
    async fn get_trace(&self, trace_id: &str) -> error::Result<Option<TraceData>>;
    async fn query_traces(&self, resource_id: &ResourceId, filters: TraceFilters) -> error::Result<Vec<TraceData>>;
    async fn delete_traces_before(&self, timestamp: SystemTime) -> error::Result<usize>;
    
    // Evaluation Management
    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> error::Result<()>;
    async fn get_eval_dataset(&self, dataset_id: &str) -> error::Result<Option<EvalDataset>>;
    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> error::Result<Vec<EvalDataset>>;
    async fn store_eval_score(&self, score: &EvalScore) -> error::Result<()>;
    async fn get_eval_scores(&self, run_id: &str) -> error::Result<Vec<EvalScore>>;
    
    // Storage Management
    async fn get_storage_stats(&self) -> error::Result<StorageStats>;
    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> error::Result<CleanupStats>;
}

/// In-memory test implementation
pub struct InMemoryUnifiedStorage {
    workflows: Arc<tokio::sync::RwLock<HashMap<String, SuspendedWorkflow>>>,
    threads: Arc<tokio::sync::RwLock<HashMap<String, MemoryThread>>>,
    messages: Arc<tokio::sync::RwLock<HashMap<String, Vec<MemoryMessage>>>>,
    traces: Arc<tokio::sync::RwLock<HashMap<String, TraceData>>>,
    datasets: Arc<tokio::sync::RwLock<HashMap<String, EvalDataset>>>,
    scores: Arc<tokio::sync::RwLock<HashMap<String, Vec<EvalScore>>>>,
}

impl InMemoryUnifiedStorage {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            threads: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            messages: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            traces: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            datasets: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            scores: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UnifiedStorage for InMemoryUnifiedStorage {
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> error::Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.workflow_id.clone(), workflow.clone());
        Ok(())
    }
    
    async fn get_suspended_workflow(&self, workflow_id: &str) -> error::Result<Option<SuspendedWorkflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.get(workflow_id).cloned())
    }
    
    async fn list_suspended_workflows(&self, resource_id: &ResourceId) -> error::Result<Vec<SuspendedWorkflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.values()
            .filter(|w| &w.resource_id == resource_id)
            .cloned()
            .collect())
    }
    
    async fn resume_workflow(&self, workflow_id: &str) -> error::Result<SuspendedWorkflow> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(workflow_id)
            .ok_or_else(|| error::AgentError::Config(format!("Workflow {} not found", workflow_id)).into())
    }
    
    async fn delete_suspended_workflow(&self, workflow_id: &str) -> error::Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(workflow_id);
        Ok(())
    }
    
    async fn create_memory_thread(&self, thread: &MemoryThread) -> error::Result<()> {
        let mut threads = self.threads.write().await;
        threads.insert(thread.thread_id.clone(), thread.clone());
        Ok(())
    }
    
    async fn get_memory_thread(&self, thread_id: &str) -> error::Result<Option<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads.get(thread_id).cloned())
    }
    
    async fn list_memory_threads(&self, resource_id: &ResourceId) -> error::Result<Vec<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads.values()
            .filter(|t| &t.resource_id == resource_id)
            .cloned()
            .collect())
    }
    
    async fn add_memory_message(&self, message: &MemoryMessage) -> error::Result<()> {
        let mut messages = self.messages.write().await;
        let thread_messages = messages.entry(message.thread_id.clone()).or_insert_with(Vec::new);
        thread_messages.push(message.clone());
        Ok(())
    }
    
    async fn get_memory_messages(&self, thread_id: &str, limit: Option<usize>) -> error::Result<Vec<MemoryMessage>> {
        let messages = self.messages.read().await;
        if let Some(thread_messages) = messages.get(thread_id) {
            let mut result = thread_messages.clone();
            if let Some(limit) = limit {
                result.truncate(limit);
            }
            Ok(result)
        } else {
            Ok(vec![])
        }
    }
    
    async fn delete_memory_thread(&self, thread_id: &str) -> error::Result<()> {
        let mut threads = self.threads.write().await;
        let mut messages = self.messages.write().await;
        threads.remove(thread_id);
        messages.remove(thread_id);
        Ok(())
    }
    
    async fn store_trace(&self, trace: &TraceData) -> error::Result<()> {
        let mut traces = self.traces.write().await;
        traces.insert(trace.trace_id.clone(), trace.clone());
        Ok(())
    }
    
    async fn get_trace(&self, trace_id: &str) -> error::Result<Option<TraceData>> {
        let traces = self.traces.read().await;
        Ok(traces.get(trace_id).cloned())
    }
    
    async fn query_traces(&self, resource_id: &ResourceId, filters: TraceFilters) -> error::Result<Vec<TraceData>> {
        let traces = self.traces.read().await;
        let mut result: Vec<TraceData> = traces.values()
            .filter(|t| &t.resource_id == resource_id)
            .cloned()
            .collect();
        
        // Apply filters
        if let Some(component) = &filters.component {
            result.retain(|t| &t.component == component);
        }
        
        if let Some(operation_name) = &filters.operation_name {
            result.retain(|t| &t.operation_name == operation_name);
        }
        
        Ok(result)
    }
    
    async fn delete_traces_before(&self, timestamp: SystemTime) -> error::Result<usize> {
        let mut traces = self.traces.write().await;
        let initial_count = traces.len();
        traces.retain(|_, trace| trace.start_time >= timestamp);
        Ok(initial_count - traces.len())
    }
    
    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> error::Result<()> {
        let mut datasets = self.datasets.write().await;
        datasets.insert(dataset.dataset_id.clone(), dataset.clone());
        Ok(())
    }
    
    async fn get_eval_dataset(&self, dataset_id: &str) -> error::Result<Option<EvalDataset>> {
        let datasets = self.datasets.read().await;
        Ok(datasets.get(dataset_id).cloned())
    }
    
    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> error::Result<Vec<EvalDataset>> {
        let datasets = self.datasets.read().await;
        Ok(datasets.values()
            .filter(|d| &d.resource_id == resource_id)
            .cloned()
            .collect())
    }
    
    async fn store_eval_score(&self, score: &EvalScore) -> error::Result<()> {
        let mut scores = self.scores.write().await;
        let run_scores = scores.entry(score.run_id.clone()).or_insert_with(Vec::new);
        run_scores.push(score.clone());
        Ok(())
    }
    
    async fn get_eval_scores(&self, run_id: &str) -> error::Result<Vec<EvalScore>> {
        let scores = self.scores.read().await;
        Ok(scores.get(run_id).cloned().unwrap_or_default())
    }
    
    async fn get_storage_stats(&self) -> error::Result<StorageStats> {
        let workflows = self.workflows.read().await;
        let threads = self.threads.read().await;
        let messages = self.messages.read().await;
        let traces = self.traces.read().await;
        let datasets = self.datasets.read().await;
        let scores = self.scores.read().await;
        
        let total_messages = messages.values().map(|v| v.len()).sum();
        let total_scores = scores.values().map(|v| v.len()).sum();
        
        Ok(StorageStats {
            suspended_workflows: workflows.len(),
            memory_threads: threads.len(),
            memory_messages: total_messages,
            traces: traces.len(),
            eval_datasets: datasets.len(),
            eval_runs: scores.len(),
            eval_scores: total_scores,
            storage_size_bytes: 1024 * 1024, // Mock 1MB
        })
    }
    
    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> error::Result<CleanupStats> {
        let now = SystemTime::now();
        let traces_cutoff = now.checked_sub(retention_policy.traces_retention).unwrap_or(now);
        let memory_cutoff = now.checked_sub(retention_policy.memory_retention).unwrap_or(now);
        
        // Clean traces
        let traces_deleted = self.delete_traces_before(traces_cutoff).await?;
        
        // Mock cleanup stats for messages and workflows
        let messages_deleted = 5;
        let workflows_deleted = 1;
        let eval_data_deleted = 2;
        
        Ok(CleanupStats {
            traces_deleted,
            messages_deleted,
            workflows_deleted,
            eval_data_deleted,
            bytes_freed: 1024 * 100, // Mock 100KB freed
        })
    }
}

// Storage manager for coordination
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
    
    pub async fn suspend_workflow(
        &self,
        workflow_id: &str,
        resource_id: ResourceId,
        workflow_name: &str,
        current_step: usize,
        context_state: Value,
        suspend_reason: SuspendReason,
    ) -> error::Result<()> {
        let suspended_workflow = SuspendedWorkflow {
            workflow_id: workflow_id.to_string(),
            resource_id,
            workflow_name: workflow_name.to_string(),
            current_step,
            context_state,
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason,
            resume_conditions: vec![ResumeCondition::ManualResume],
        };
        
        self.storage.store_suspended_workflow(&suspended_workflow).await
    }
    
    pub async fn resume_workflow(&self, workflow_id: &str) -> error::Result<(Value, usize)> {
        let suspended = self.storage.resume_workflow(workflow_id).await?;
        Ok((suspended.context_state, suspended.current_step))
    }
    
    pub async fn create_conversation_thread(
        &self,
        resource_id: ResourceId,
        title: &str,
    ) -> error::Result<String> {
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
    
    pub async fn add_message(
        &self,
        thread_id: &str,
        resource_id: ResourceId,
        role: MessageRole,
        content: &str,
    ) -> error::Result<String> {
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
    
    pub async fn record_trace(
        &self,
        resource_id: ResourceId,
        component: &str,
        operation_name: &str,
        start_time: SystemTime,
        end_time: SystemTime,
        status: TraceStatus,
        attributes: HashMap<String, String>,
    ) -> error::Result<String> {
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
    
    pub async fn create_evaluation_dataset(
        &self,
        resource_id: ResourceId,
        name: &str,
        description: &str,
        version: &str,
    ) -> error::Result<String> {
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
    
    pub async fn record_evaluation_score(
        &self,
        run_id: &str,
        item_id: &str,
        resource_id: ResourceId,
        metric_name: &str,
        score: f64,
        reason: &str,
        scorer_name: &str,
    ) -> error::Result<()> {
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
    
    pub async fn perform_maintenance(&self) -> error::Result<CleanupStats> {
        self.storage.cleanup_old_data(&self.retention_policy).await
    }
    
    pub async fn get_statistics(&self) -> error::Result<StorageStats> {
        self.storage.get_storage_stats().await
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_resource_id() -> ResourceId {
        ResourceId::new("test_app", "user_123")
    }

    fn create_test_retention_policy() -> RetentionPolicy {
        RetentionPolicy {
            traces_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
            memory_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
            eval_retention: Duration::from_secs(90 * 24 * 3600), // 90 days
            suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600), // 1 year
        }
    }

    #[tokio::test]
    async fn test_resource_id_creation_and_key() {
        let resource_id = ResourceId::new("test_namespace", "test_id");
        assert_eq!(resource_id.namespace, "test_namespace");
        assert_eq!(resource_id.id, "test_id");
        assert_eq!(resource_id.to_key(), "test_namespace:test_id");
    }

    #[tokio::test]
    async fn test_suspended_workflow_storage_and_retrieval() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let workflow = SuspendedWorkflow {
            workflow_id: "test_workflow_001".to_string(),
            resource_id: resource_id.clone(),
            workflow_name: "test_workflow".to_string(),
            current_step: 3,
            context_state: json!({"step": 3, "data": "test_data"}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![ResumeCondition::ManualResume],
        };

        // Store workflow
        storage.store_suspended_workflow(&workflow).await.unwrap();

        // Retrieve workflow
        let retrieved = storage.get_suspended_workflow("test_workflow_001").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.workflow_id, "test_workflow_001");
        assert_eq!(retrieved.current_step, 3);
        assert_eq!(retrieved.resource_id, resource_id);

        // List workflows for resource
        let workflows = storage.list_suspended_workflows(&resource_id).await.unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].workflow_id, "test_workflow_001");
    }

    #[tokio::test]
    async fn test_workflow_resume_and_delete() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let workflow = SuspendedWorkflow {
            workflow_id: "test_workflow_002".to_string(),
            resource_id: resource_id.clone(),
            workflow_name: "resume_test".to_string(),
            current_step: 5,
            context_state: json!({"step": 5, "resumed": true}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::Sleep { duration_ms: 5000 },
            resume_conditions: vec![ResumeCondition::TimeElapsed],
        };

        storage.store_suspended_workflow(&workflow).await.unwrap();

        // Resume workflow (should remove it from storage)
        let resumed = storage.resume_workflow("test_workflow_002").await.unwrap();
        assert_eq!(resumed.workflow_id, "test_workflow_002");
        assert_eq!(resumed.current_step, 5);

        // Should no longer exist
        let retrieved = storage.get_suspended_workflow("test_workflow_002").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_thread_management() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let thread = MemoryThread {
            thread_id: "thread_001".to_string(),
            resource_id: resource_id.clone(),
            title: "Test Conversation".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        };

        // Create thread
        storage.create_memory_thread(&thread).await.unwrap();

        // Retrieve thread
        let retrieved = storage.get_memory_thread("thread_001").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.thread_id, "thread_001");
        assert_eq!(retrieved.title, "Test Conversation");

        // List threads for resource
        let threads = storage.list_memory_threads(&resource_id).await.unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].thread_id, "thread_001");
    }

    #[tokio::test]
    async fn test_memory_message_management() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        // Create thread first
        let thread = MemoryThread {
            thread_id: "thread_002".to_string(),
            resource_id: resource_id.clone(),
            title: "Message Test Thread".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        };
        storage.create_memory_thread(&thread).await.unwrap();

        // Add messages
        let messages = vec![
            ("msg_001", MessageRole::User, "Hello, how are you?"),
            ("msg_002", MessageRole::Assistant, "I'm doing well, thank you!"),
            ("msg_003", MessageRole::User, "What's the weather like?"),
        ];

        for (msg_id, role, content) in messages {
            let message = MemoryMessage {
                message_id: msg_id.to_string(),
                thread_id: "thread_002".to_string(),
                resource_id: resource_id.clone(),
                role,
                content: content.to_string(),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
                parent_message_id: None,
            };
            storage.add_memory_message(&message).await.unwrap();
        }

        // Retrieve all messages
        let retrieved = storage.get_memory_messages("thread_002", None).await.unwrap();
        assert_eq!(retrieved.len(), 3);
        assert_eq!(retrieved[0].message_id, "msg_001");

        // Retrieve limited messages
        let limited = storage.get_memory_messages("thread_002", Some(2)).await.unwrap();
        assert_eq!(limited.len(), 2);

        // Delete thread (should remove messages too)
        storage.delete_memory_thread("thread_002").await.unwrap();
        let retrieved = storage.get_memory_messages("thread_002", None).await.unwrap();
        assert!(retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_trace_management() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        let now = SystemTime::now();
        
        let trace = TraceData {
            trace_id: "trace_001".to_string(),
            span_id: "span_001".to_string(),
            parent_span_id: None,
            resource_id: resource_id.clone(),
            operation_name: "test_operation".to_string(),
            start_time: now,
            end_time: Some(now + Duration::from_millis(100)),
            duration_ms: Some(100),
            status: TraceStatus::Ok,
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("test_key".to_string(), "test_value".to_string());
                attrs
            },
            events: vec![],
            component: "test_component".to_string(),
        };

        // Store trace
        storage.store_trace(&trace).await.unwrap();

        // Retrieve trace
        let retrieved = storage.get_trace("trace_001").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.trace_id, "trace_001");
        assert_eq!(retrieved.operation_name, "test_operation");
        assert_eq!(retrieved.component, "test_component");

        // Query traces with filters
        let filters = TraceFilters {
            component: Some("test_component".to_string()),
            ..Default::default()
        };
        let traces = storage.query_traces(&resource_id, filters).await.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].trace_id, "trace_001");
    }

    #[tokio::test]
    async fn test_trace_cleanup() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        let now = SystemTime::now();
        
        // Store traces with different timestamps
        for i in 0..5 {
            let trace = TraceData {
                trace_id: format!("trace_{:03}", i),
                span_id: format!("span_{:03}", i),
                parent_span_id: None,
                resource_id: resource_id.clone(),
                operation_name: "test_operation".to_string(),
                start_time: now - Duration::from_secs(i * 3600), // Each trace 1 hour older
                end_time: Some(now - Duration::from_secs(i * 3600) + Duration::from_millis(100)),
                duration_ms: Some(100),
                status: TraceStatus::Ok,
                attributes: HashMap::new(),
                events: vec![],
                component: "test_component".to_string(),
            };
            storage.store_trace(&trace).await.unwrap();
        }

        // Delete traces older than 2 hours
        let cutoff = now - Duration::from_secs(2 * 3600);
        let deleted_count = storage.delete_traces_before(cutoff).await.unwrap();
        assert_eq!(deleted_count, 3); // Should delete traces 2, 3, 4

        // Verify remaining traces
        let filters = TraceFilters::default();
        let remaining = storage.query_traces(&resource_id, filters).await.unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn test_evaluation_dataset_management() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let dataset = EvalDataset {
            dataset_id: "dataset_001".to_string(),
            name: "Test Dataset".to_string(),
            description: "A test evaluation dataset".to_string(),
            resource_id: resource_id.clone(),
            created_at: SystemTime::now(),
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        };

        // Store dataset
        storage.create_eval_dataset(&dataset).await.unwrap();

        // Retrieve dataset
        let retrieved = storage.get_eval_dataset("dataset_001").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.dataset_id, "dataset_001");
        assert_eq!(retrieved.name, "Test Dataset");
        assert_eq!(retrieved.version, "1.0");

        // List datasets for resource
        let datasets = storage.list_eval_datasets(&resource_id).await.unwrap();
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].dataset_id, "dataset_001");
    }

    #[tokio::test]
    async fn test_evaluation_score_management() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let scores = vec![
            ("score_001", "run_001", "accuracy", 0.85),
            ("score_002", "run_001", "precision", 0.90),
            ("score_003", "run_001", "recall", 0.80),
        ];

        // Store scores
        for (score_id, run_id, metric, score_value) in scores {
            let score = EvalScore {
                score_id: score_id.to_string(),
                run_id: run_id.to_string(),
                item_id: "item_001".to_string(),
                resource_id: resource_id.clone(),
                metric_name: metric.to_string(),
                score: score_value,
                reason: format!("Scored {} for {}", score_value, metric),
                scorer_name: "test_scorer".to_string(),
                metadata: HashMap::new(),
                scored_at: SystemTime::now(),
            };
            storage.store_eval_score(&score).await.unwrap();
        }

        // Retrieve scores for run
        let run_scores = storage.get_eval_scores("run_001").await.unwrap();
        assert_eq!(run_scores.len(), 3);
        
        let accuracy_score = run_scores.iter().find(|s| s.metric_name == "accuracy").unwrap();
        assert_eq!(accuracy_score.score, 0.85);
        assert_eq!(accuracy_score.scorer_name, "test_scorer");
    }

    #[tokio::test]
    async fn test_storage_statistics() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();

        // Add some data
        let workflow = SuspendedWorkflow {
            workflow_id: "workflow_stats_test".to_string(),
            resource_id: resource_id.clone(),
            workflow_name: "stats_test".to_string(),
            current_step: 1,
            context_state: json!({}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![],
        };
        storage.store_suspended_workflow(&workflow).await.unwrap();

        let thread = MemoryThread {
            thread_id: "thread_stats_test".to_string(),
            resource_id: resource_id.clone(),
            title: "Stats Test Thread".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        };
        storage.create_memory_thread(&thread).await.unwrap();

        // Get stats
        let stats = storage.get_storage_stats().await.unwrap();
        assert_eq!(stats.suspended_workflows, 1);
        assert_eq!(stats.memory_threads, 1);
        assert_eq!(stats.storage_size_bytes, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_storage_manager_workflow_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = create_test_retention_policy();
        let manager = StorageManager::new(storage, retention_policy);
        let resource_id = create_test_resource_id();

        // Suspend workflow
        manager.suspend_workflow(
            "manager_test_001",
            resource_id.clone(),
            "test_workflow",
            2,
            json!({"step": 2, "test": true}),
            SuspendReason::WaitingForEvent {
                event_id: "user_input".to_string(),
                timeout_ms: Some(30000),
            },
        ).await.unwrap();

        // Resume workflow
        let (context, step) = manager.resume_workflow("manager_test_001").await.unwrap();
        assert_eq!(step, 2);
        assert_eq!(context["step"], 2);
        assert_eq!(context["test"], true);
    }

    #[tokio::test]
    async fn test_storage_manager_conversation_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = create_test_retention_policy();
        let manager = StorageManager::new(storage, retention_policy);
        let resource_id = create_test_resource_id();

        // Create conversation thread
        let thread_id = manager.create_conversation_thread(
            resource_id.clone(),
            "Test Conversation"
        ).await.unwrap();

        assert!(!thread_id.is_empty());

        // Add messages
        let message_id1 = manager.add_message(
            &thread_id,
            resource_id.clone(),
            MessageRole::User,
            "Hello, how are you?"
        ).await.unwrap();

        let message_id2 = manager.add_message(
            &thread_id,
            resource_id.clone(),
            MessageRole::Assistant,
            "I'm doing well, thanks!"
        ).await.unwrap();

        assert!(!message_id1.is_empty());
        assert!(!message_id2.is_empty());
        assert_ne!(message_id1, message_id2);
    }

    #[tokio::test]
    async fn test_storage_manager_trace_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = create_test_retention_policy();
        let manager = StorageManager::new(storage, retention_policy);
        let resource_id = create_test_resource_id();

        let now = SystemTime::now();
        let mut attributes = HashMap::new();
        attributes.insert("operation_type".to_string(), "test".to_string());
        attributes.insert("user_id".to_string(), "123".to_string());

        // Record trace
        let trace_id = manager.record_trace(
            resource_id,
            "test_component",
            "test_operation",
            now,
            now + Duration::from_millis(150),
            TraceStatus::Ok,
            attributes,
        ).await.unwrap();

        assert!(!trace_id.is_empty());
    }

    #[tokio::test]
    async fn test_storage_manager_evaluation_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = create_test_retention_policy();
        let manager = StorageManager::new(storage, retention_policy);
        let resource_id = create_test_resource_id();

        // Create evaluation dataset
        let dataset_id = manager.create_evaluation_dataset(
            resource_id.clone(),
            "Test Evaluation Dataset",
            "A comprehensive test dataset",
            "2.0",
        ).await.unwrap();

        assert!(!dataset_id.is_empty());

        // Record evaluation score
        manager.record_evaluation_score(
            "eval_run_001",
            "item_001",
            resource_id,
            "accuracy",
            0.92,
            "High accuracy achieved",
            "automated_evaluator",
        ).await.unwrap();
    }

    #[tokio::test]
    async fn test_storage_manager_maintenance_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = create_test_retention_policy();
        let manager = StorageManager::new(storage, retention_policy);

        // Perform maintenance
        let cleanup_stats = manager.perform_maintenance().await.unwrap();
        assert!(cleanup_stats.bytes_freed > 0);

        // Get statistics
        let stats = manager.get_statistics().await.unwrap();
        assert!(stats.storage_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_concurrent_storage_operations() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        // Spawn concurrent tasks
        let mut handles = vec![];
        
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let resource_id_clone = resource_id.clone();
            
            let handle = tokio::spawn(async move {
                // Each task stores a workflow and a trace
                let workflow = SuspendedWorkflow {
                    workflow_id: format!("concurrent_test_{}", i),
                    resource_id: resource_id_clone.clone(),
                    workflow_name: format!("concurrent_workflow_{}", i),
                    current_step: i,
                    context_state: json!({"thread": i}),
                    metadata: HashMap::new(),
                    suspended_at: SystemTime::now(),
                    suspend_reason: SuspendReason::UserPause,
                    resume_conditions: vec![],
                };
                
                storage_clone.store_suspended_workflow(&workflow).await.unwrap();
                
                let trace = TraceData {
                    trace_id: format!("concurrent_trace_{}", i),
                    span_id: format!("concurrent_span_{}", i),
                    parent_span_id: None,
                    resource_id: resource_id_clone,
                    operation_name: "concurrent_operation".to_string(),
                    start_time: SystemTime::now(),
                    end_time: Some(SystemTime::now()),
                    duration_ms: Some(10),
                    status: TraceStatus::Ok,
                    attributes: HashMap::new(),
                    events: vec![],
                    component: "concurrent_component".to_string(),
                };
                
                storage_clone.store_trace(&trace).await.unwrap();
                
                i
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let results: Vec<_> = futures::future::join_all(handles).await.into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        assert_eq!(results.len(), 10);
        
        // Verify all data was stored
        let workflows = storage.list_suspended_workflows(&resource_id).await.unwrap();
        assert_eq!(workflows.len(), 10);
        
        let filters = TraceFilters {
            component: Some("concurrent_component".to_string()),
            ..Default::default()
        };
        let traces = storage.query_traces(&resource_id, filters).await.unwrap();
        assert_eq!(traces.len(), 10);
    }

    #[tokio::test]
    async fn test_resource_isolation() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        
        let resource_1 = ResourceId::new("app1", "user1");
        let resource_2 = ResourceId::new("app2", "user2");
        
        // Store data for different resources
        let workflow_1 = SuspendedWorkflow {
            workflow_id: "isolation_test_1".to_string(),
            resource_id: resource_1.clone(),
            workflow_name: "workflow_1".to_string(),
            current_step: 1,
            context_state: json!({}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![],
        };
        
        let workflow_2 = SuspendedWorkflow {
            workflow_id: "isolation_test_2".to_string(),
            resource_id: resource_2.clone(),
            workflow_name: "workflow_2".to_string(),
            current_step: 1,
            context_state: json!({}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![],
        };
        
        storage.store_suspended_workflow(&workflow_1).await.unwrap();
        storage.store_suspended_workflow(&workflow_2).await.unwrap();
        
        // Verify isolation
        let workflows_1 = storage.list_suspended_workflows(&resource_1).await.unwrap();
        let workflows_2 = storage.list_suspended_workflows(&resource_2).await.unwrap();
        
        assert_eq!(workflows_1.len(), 1);
        assert_eq!(workflows_2.len(), 1);
        assert_eq!(workflows_1[0].workflow_id, "isolation_test_1");
        assert_eq!(workflows_2[0].workflow_id, "isolation_test_2");
    }

    #[tokio::test]
    async fn test_serialization_deserialization() {
        let resource_id = ResourceId::new("test", "123");
        
        // Test SuspendedWorkflow serialization
        let workflow = SuspendedWorkflow {
            workflow_id: "serialization_test".to_string(),
            resource_id: resource_id.clone(),
            workflow_name: "test_workflow".to_string(),
            current_step: 5,
            context_state: json!({"complex": {"nested": "data"}, "array": [1, 2, 3]}),
            metadata: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), "value1".to_string());
                map.insert("key2".to_string(), "value2".to_string());
                map
            },
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::WaitingForEvent {
                event_id: "test_event".to_string(),
                timeout_ms: Some(60000),
            },
            resume_conditions: vec![
                ResumeCondition::EventReceived { event_id: "test_event".to_string() },
                ResumeCondition::TimeElapsed,
            ],
        };
        
        let serialized = serde_json::to_string(&workflow).unwrap();
        let deserialized: SuspendedWorkflow = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(workflow.workflow_id, deserialized.workflow_id);
        assert_eq!(workflow.current_step, deserialized.current_step);
        assert_eq!(workflow.context_state, deserialized.context_state);
        assert_eq!(workflow.metadata, deserialized.metadata);
        
        // Test TraceStatus serialization
        let status_ok = TraceStatus::Ok;
        let status_error = TraceStatus::Error { message: "Test error".to_string() };
        
        let serialized_ok = serde_json::to_string(&status_ok).unwrap();
        let serialized_error = serde_json::to_string(&status_error).unwrap();
        
        let deserialized_ok: TraceStatus = serde_json::from_str(&serialized_ok).unwrap();
        let deserialized_error: TraceStatus = serde_json::from_str(&serialized_error).unwrap();
        
        assert!(matches!(deserialized_ok, TraceStatus::Ok));
        if let TraceStatus::Error { message } = deserialized_error {
            assert_eq!(message, "Test error");
        } else {
            panic!("Expected error status");
        }
    }
    
    #[tokio::test] 
    async fn test_error_handling() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        
        // Test resuming non-existent workflow
        let result = storage.resume_workflow("non_existent").await;
        assert!(result.is_err());
        
        // Test getting non-existent thread
        let thread = storage.get_memory_thread("non_existent").await.unwrap();
        assert!(thread.is_none());
        
        // Test getting non-existent trace
        let trace = storage.get_trace("non_existent").await.unwrap();
        assert!(trace.is_none());
        
        // Test getting non-existent dataset
        let dataset = storage.get_eval_dataset("non_existent").await.unwrap();
        assert!(dataset.is_none());
        
        // Test getting scores for non-existent run
        let scores = storage.get_eval_scores("non_existent").await.unwrap();
        assert!(scores.is_empty());
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = create_test_resource_id();
        
        let start_time = SystemTime::now();
        
        // Store a large number of items quickly
        let mut handles = vec![];
        
        for batch in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let resource_id_clone = resource_id.clone();
            
            let handle = tokio::spawn(async move {
                for i in 0..100 {
                    let item_id = batch * 100 + i;
                    
                    // Store a trace
                    let trace = TraceData {
                        trace_id: format!("perf_trace_{}", item_id),
                        span_id: format!("perf_span_{}", item_id),
                        parent_span_id: None,
                        resource_id: resource_id_clone.clone(),
                        operation_name: "performance_test".to_string(),
                        start_time: SystemTime::now(),
                        end_time: Some(SystemTime::now()),
                        duration_ms: Some(1),
                        status: TraceStatus::Ok,
                        attributes: HashMap::new(),
                        events: vec![],
                        component: "performance_component".to_string(),
                    };
                    
                    storage_clone.store_trace(&trace).await.unwrap();
                }
                batch
            });
            
            handles.push(handle);
        }
        
        futures::future::join_all(handles).await;
        
        let elapsed = start_time.elapsed().unwrap();
        println!("Stored 1000 traces in {:?}", elapsed);
        
        // Verify all traces were stored
        let filters = TraceFilters {
            component: Some("performance_component".to_string()),
            ..Default::default()
        };
        let traces = storage.query_traces(&resource_id, filters).await.unwrap();
        assert_eq!(traces.len(), 1000);
        
        // Performance should be reasonable (less than 5 seconds for 1000 items)
        assert!(elapsed.as_secs() < 5);
    }
}