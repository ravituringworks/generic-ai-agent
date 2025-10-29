//! Unified Storage System for AI Agent
//!
//! This module provides a comprehensive storage abstraction that handles:
//! - Suspended workflow state management
//! - Memory thread and message persistence
//! - OpenTelemetry trace data collection
//! - Evaluation dataset and scoring management
//! - Cross-component data consistency and isolation

use crate::error::{AgentError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

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
    WaitingForEvent {
        event_id: String,
        timeout_ms: Option<u64>,
    },
    Sleep {
        duration_ms: u64,
    },
    SleepUntil {
        timestamp: SystemTime,
    },
    ExternalDependency {
        dependency_type: String,
        details: String,
    },
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
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()>;
    async fn get_suspended_workflow(&self, workflow_id: &str) -> Result<Option<SuspendedWorkflow>>;
    async fn list_suspended_workflows(
        &self,
        resource_id: &ResourceId,
    ) -> Result<Vec<SuspendedWorkflow>>;
    async fn resume_workflow(&self, workflow_id: &str) -> Result<SuspendedWorkflow>;
    async fn delete_suspended_workflow(&self, workflow_id: &str) -> Result<()>;

    // Memory Management
    async fn create_memory_thread(&self, thread: &MemoryThread) -> Result<()>;
    async fn get_memory_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>>;
    async fn list_memory_threads(&self, resource_id: &ResourceId) -> Result<Vec<MemoryThread>>;
    async fn add_memory_message(&self, message: &MemoryMessage) -> Result<()>;
    async fn get_memory_messages(
        &self,
        thread_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryMessage>>;
    async fn delete_memory_thread(&self, thread_id: &str) -> Result<()>;

    // Trace Management
    async fn store_trace(&self, trace: &TraceData) -> Result<()>;
    async fn get_trace(&self, trace_id: &str) -> Result<Option<TraceData>>;
    async fn query_traces(
        &self,
        resource_id: &ResourceId,
        filters: TraceFilters,
    ) -> Result<Vec<TraceData>>;
    async fn delete_traces_before(&self, timestamp: SystemTime) -> Result<usize>;

    // Evaluation Management
    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> Result<()>;
    async fn get_eval_dataset(&self, dataset_id: &str) -> Result<Option<EvalDataset>>;
    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> Result<Vec<EvalDataset>>;
    async fn store_eval_score(&self, score: &EvalScore) -> Result<()>;
    async fn get_eval_scores(&self, run_id: &str) -> Result<Vec<EvalScore>>;

    // Storage Management
    async fn get_storage_stats(&self) -> Result<StorageStats>;
    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> Result<CleanupStats>;
}

/// In-memory test implementation
pub struct InMemoryUnifiedStorage {
    workflows: Arc<RwLock<HashMap<String, SuspendedWorkflow>>>,
    threads: Arc<RwLock<HashMap<String, MemoryThread>>>,
    messages: Arc<RwLock<HashMap<String, Vec<MemoryMessage>>>>,
    traces: Arc<RwLock<HashMap<String, TraceData>>>,
    datasets: Arc<RwLock<HashMap<String, EvalDataset>>>,
    scores: Arc<RwLock<HashMap<String, Vec<EvalScore>>>>,
}

impl InMemoryUnifiedStorage {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            threads: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            traces: Arc::new(RwLock::new(HashMap::new())),
            datasets: Arc::new(RwLock::new(HashMap::new())),
            scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UnifiedStorage for InMemoryUnifiedStorage {
    async fn store_suspended_workflow(&self, workflow: &SuspendedWorkflow) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.workflow_id.clone(), workflow.clone());
        Ok(())
    }

    async fn get_suspended_workflow(&self, workflow_id: &str) -> Result<Option<SuspendedWorkflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.get(workflow_id).cloned())
    }

    async fn list_suspended_workflows(
        &self,
        resource_id: &ResourceId,
    ) -> Result<Vec<SuspendedWorkflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows
            .values()
            .filter(|w| &w.resource_id == resource_id)
            .cloned()
            .collect())
    }

    async fn resume_workflow(&self, workflow_id: &str) -> Result<SuspendedWorkflow> {
        let mut workflows = self.workflows.write().await;
        workflows
            .remove(workflow_id)
            .ok_or_else(|| AgentError::Config(format!("Workflow {} not found", workflow_id)))
    }

    async fn delete_suspended_workflow(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(workflow_id);
        Ok(())
    }

    async fn create_memory_thread(&self, thread: &MemoryThread) -> Result<()> {
        let mut threads = self.threads.write().await;
        threads.insert(thread.thread_id.clone(), thread.clone());
        Ok(())
    }

    async fn get_memory_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads.get(thread_id).cloned())
    }

    async fn list_memory_threads(&self, resource_id: &ResourceId) -> Result<Vec<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads
            .values()
            .filter(|t| &t.resource_id == resource_id)
            .cloned()
            .collect())
    }

    async fn add_memory_message(&self, message: &MemoryMessage) -> Result<()> {
        let mut messages = self.messages.write().await;
        let thread_messages = messages
            .entry(message.thread_id.clone())
            .or_insert_with(Vec::new);
        thread_messages.push(message.clone());
        Ok(())
    }

    async fn get_memory_messages(
        &self,
        thread_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryMessage>> {
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

    async fn delete_memory_thread(&self, thread_id: &str) -> Result<()> {
        let mut threads = self.threads.write().await;
        let mut messages = self.messages.write().await;
        threads.remove(thread_id);
        messages.remove(thread_id);
        Ok(())
    }

    async fn store_trace(&self, trace: &TraceData) -> Result<()> {
        let mut traces = self.traces.write().await;
        traces.insert(trace.trace_id.clone(), trace.clone());
        Ok(())
    }

    async fn get_trace(&self, trace_id: &str) -> Result<Option<TraceData>> {
        let traces = self.traces.read().await;
        Ok(traces.get(trace_id).cloned())
    }

    async fn query_traces(
        &self,
        resource_id: &ResourceId,
        filters: TraceFilters,
    ) -> Result<Vec<TraceData>> {
        let traces = self.traces.read().await;
        let mut result: Vec<TraceData> = traces
            .values()
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

    async fn delete_traces_before(&self, timestamp: SystemTime) -> Result<usize> {
        let mut traces = self.traces.write().await;
        let initial_count = traces.len();
        traces.retain(|_, trace| trace.start_time >= timestamp);
        Ok(initial_count - traces.len())
    }

    async fn create_eval_dataset(&self, dataset: &EvalDataset) -> Result<()> {
        let mut datasets = self.datasets.write().await;
        datasets.insert(dataset.dataset_id.clone(), dataset.clone());
        Ok(())
    }

    async fn get_eval_dataset(&self, dataset_id: &str) -> Result<Option<EvalDataset>> {
        let datasets = self.datasets.read().await;
        Ok(datasets.get(dataset_id).cloned())
    }

    async fn list_eval_datasets(&self, resource_id: &ResourceId) -> Result<Vec<EvalDataset>> {
        let datasets = self.datasets.read().await;
        Ok(datasets
            .values()
            .filter(|d| &d.resource_id == resource_id)
            .cloned()
            .collect())
    }

    async fn store_eval_score(&self, score: &EvalScore) -> Result<()> {
        let mut scores = self.scores.write().await;
        let run_scores = scores.entry(score.run_id.clone()).or_insert_with(Vec::new);
        run_scores.push(score.clone());
        Ok(())
    }

    async fn get_eval_scores(&self, run_id: &str) -> Result<Vec<EvalScore>> {
        let scores = self.scores.read().await;
        Ok(scores.get(run_id).cloned().unwrap_or_default())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
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

    async fn cleanup_old_data(&self, retention_policy: &RetentionPolicy) -> Result<CleanupStats> {
        let now = SystemTime::now();
        let traces_cutoff = now
            .checked_sub(retention_policy.traces_retention)
            .unwrap_or(now);

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

/// Storage manager for coordination
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
    ) -> Result<()> {
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

        self.storage
            .store_suspended_workflow(&suspended_workflow)
            .await
    }

    pub async fn resume_workflow(&self, workflow_id: &str) -> Result<(Value, usize)> {
        let suspended = self.storage.resume_workflow(workflow_id).await?;
        Ok((suspended.context_state, suspended.current_step))
    }

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
        let duration_ms = end_time
            .duration_since(start_time)
            .ok()
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

    pub async fn record_evaluation_score(
        &self,
        run_id: &str,
        item_id: &str,
        resource_id: ResourceId,
        metric_name: &str,
        score: f64,
        reason: &str,
        scorer_name: &str,
    ) -> Result<String> {
        let score_id = Uuid::new_v4().to_string();
        let eval_score = EvalScore {
            score_id: score_id.clone(),
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

        self.storage.store_eval_score(&eval_score).await?;
        Ok(score_id)
    }

    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        self.storage.get_storage_stats().await
    }

    pub async fn perform_cleanup(&self) -> Result<CleanupStats> {
        self.storage.cleanup_old_data(&self.retention_policy).await
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            traces_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
            memory_retention: Duration::from_secs(90 * 24 * 3600), // 90 days
            eval_retention: Duration::from_secs(365 * 24 * 3600),  // 1 year
            suspended_workflows_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
        }
    }
}
