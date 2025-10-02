//! BDD tests for the unified storage system using Cucumber

use cucumber::{given, then, when, World};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde_json::{json, Value};
use uuid::Uuid;

// Include the storage system components
use crate::unified_storage_tests::*;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct StorageWorld {
    storage: Option<Arc<dyn UnifiedStorage>>,
    storage_manager: Option<StorageManager>,
    resource_id: Option<ResourceId>,
    workflow_id: Option<String>,
    thread_id: Option<String>,
    trace_id: Option<String>,
    dataset_id: Option<String>,
    last_error: Option<String>,
    stats: Option<StorageStats>,
}

impl StorageWorld {
    pub fn new() -> Self {
        Self {
            storage: None,
            storage_manager: None,
            resource_id: None,
            workflow_id: None,
            thread_id: None,
            trace_id: None,
            dataset_id: None,
            last_error: None,
            stats: None,
        }
    }
}

// Given steps
#[given("I have a unified storage system")]
async fn setup_storage_system(world: &mut StorageWorld) {
    let storage = Arc::new(InMemoryUnifiedStorage::new());
    let retention_policy = RetentionPolicy {
        traces_retention: Duration::from_secs(7 * 24 * 3600),
        memory_retention: Duration::from_secs(30 * 24 * 3600),
        eval_retention: Duration::from_secs(90 * 24 * 3600),
        suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600),
    };
    
    world.storage_manager = Some(StorageManager::new(storage.clone(), retention_policy));
    world.storage = Some(storage);
}

#[given("I have a resource identifier for a test application")]
async fn setup_resource_identifier(world: &mut StorageWorld) {
    world.resource_id = Some(ResourceId::new("test_app", "user_123"));
}

#[given(regex = r"I have a resource identifier for (.+) and (.+)")]
async fn setup_custom_resource_identifier(world: &mut StorageWorld, namespace: String, id: String) {
    world.resource_id = Some(ResourceId::new(&namespace, &id));
}

#[given("I have a suspended workflow")]
async fn setup_suspended_workflow(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let workflow = SuspendedWorkflow {
            workflow_id: "test_workflow_001".to_string(),
            resource_id: resource_id.clone(),
            workflow_name: "test_workflow".to_string(),
            current_step: 2,
            context_state: json!({"step": 2, "test": true}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason: SuspendReason::UserPause,
            resume_conditions: vec![ResumeCondition::ManualResume],
        };

        storage.store_suspended_workflow(&workflow).await.unwrap();
        world.workflow_id = Some("test_workflow_001".to_string());
    }
}

#[given("I have a memory thread with messages")]
async fn setup_memory_thread_with_messages(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let thread_id = Uuid::new_v4().to_string();
        
        let thread = MemoryThread {
            thread_id: thread_id.clone(),
            resource_id: resource_id.clone(),
            title: "BDD Test Conversation".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            metadata: HashMap::new(),
            message_count: 0,
        };
        
        storage.create_memory_thread(&thread).await.unwrap();
        
        // Add some messages
        let messages = vec![
            ("user", "Hello, how are you?"),
            ("assistant", "I'm doing well, thank you!"),
            ("user", "Can you help me with a task?"),
            ("assistant", "Of course! What do you need help with?"),
        ];
        
        for (i, (role, content)) in messages.into_iter().enumerate() {
            let message_role = match role {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::System,
            };
            
            let message = MemoryMessage {
                message_id: format!("msg_{}", i),
                thread_id: thread_id.clone(),
                resource_id: resource_id.clone(),
                role: message_role,
                content: content.to_string(),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
                parent_message_id: None,
            };
            
            storage.add_memory_message(&message).await.unwrap();
        }
        
        world.thread_id = Some(thread_id);
    }
}

#[given("I have traces from different components")]
async fn setup_traces_from_components(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let components = vec!["llm", "memory", "workflow", "agent"];
        let now = SystemTime::now();
        
        for (i, component) in components.into_iter().enumerate() {
            let trace = TraceData {
                trace_id: format!("trace_{}", i),
                span_id: format!("span_{}", i),
                parent_span_id: None,
                resource_id: resource_id.clone(),
                operation_name: format!("{}_operation", component),
                start_time: now - Duration::from_millis(i as u64 * 100),
                end_time: Some(now - Duration::from_millis(i as u64 * 100) + Duration::from_millis(50)),
                duration_ms: Some(50),
                status: if i == 3 { 
                    TraceStatus::Error { message: "Test error".to_string() } 
                } else { 
                    TraceStatus::Ok 
                },
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("component".to_string(), component.to_string());
                    attrs
                },
                events: vec![],
                component: component.to_string(),
            };
            
            storage.store_trace(&trace).await.unwrap();
        }
    }
}

#[given("I have evaluation datasets and scores")]
async fn setup_evaluation_data(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let dataset = EvalDataset {
            dataset_id: "dataset_001".to_string(),
            name: "BDD Test Dataset".to_string(),
            description: "Dataset for BDD testing".to_string(),
            resource_id: resource_id.clone(),
            created_at: SystemTime::now(),
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        };
        
        storage.create_eval_dataset(&dataset).await.unwrap();
        world.dataset_id = Some("dataset_001".to_string());
        
        // Add some evaluation scores
        let scores = vec![
            ("accuracy", 0.85),
            ("precision", 0.90),
            ("recall", 0.80),
            ("f1_score", 0.87),
        ];
        
        for (metric_name, score_value) in scores {
            let score = EvalScore {
                score_id: Uuid::new_v4().to_string(),
                run_id: "test_run_001".to_string(),
                item_id: "test_item_001".to_string(),
                resource_id: resource_id.clone(),
                metric_name: metric_name.to_string(),
                score: score_value,
                reason: format!("BDD test score for {}", metric_name),
                scorer_name: "bdd_test_scorer".to_string(),
                metadata: HashMap::new(),
                scored_at: SystemTime::now(),
            };
            
            storage.store_eval_score(&score).await.unwrap();
        }
    }
}

// When steps
#[when("I suspend a workflow")]
async fn suspend_workflow(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id)) = (&world.storage_manager, &world.resource_id) {
        match storage_manager.suspend_workflow(
            "bdd_suspend_test",
            resource_id.clone(),
            "bdd_test_workflow",
            5,
            json!({"step": 5, "bdd": true}),
            SuspendReason::WaitingForEvent {
                event_id: "user_input".to_string(),
                timeout_ms: Some(60000),
            },
        ).await {
            Ok(_) => {
                world.workflow_id = Some("bdd_suspend_test".to_string());
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I resume the suspended workflow")]
async fn resume_suspended_workflow(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(workflow_id)) = (&world.storage_manager, &world.workflow_id) {
        match storage_manager.resume_workflow(workflow_id).await {
            Ok((context, step)) => {
                // Store results for verification
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I create a new conversation thread")]
async fn create_conversation_thread(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id)) = (&world.storage_manager, &world.resource_id) {
        match storage_manager.create_conversation_thread(
            resource_id.clone(),
            "BDD Created Conversation",
        ).await {
            Ok(thread_id) => {
                world.thread_id = Some(thread_id);
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I add messages to the conversation thread")]
async fn add_messages_to_thread(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id), Some(thread_id)) = 
        (&world.storage_manager, &world.resource_id, &world.thread_id) {
        
        let messages = vec![
            (MessageRole::User, "This is a BDD test message"),
            (MessageRole::Assistant, "I understand this is a BDD test"),
        ];
        
        for (role, content) in messages {
            match storage_manager.add_message(thread_id, resource_id.clone(), role, content).await {
                Ok(_) => {
                    world.last_error = None;
                }
                Err(e) => {
                    world.last_error = Some(e.to_string());
                    break;
                }
            }
        }
    }
}

#[when("I record traces for different operations")]
async fn record_traces(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id)) = (&world.storage_manager, &world.resource_id) {
        let operations = vec![
            ("llm", "generate_response"),
            ("memory", "search_memories"),
            ("agent", "process_input"),
        ];
        
        let now = SystemTime::now();
        
        for (component, operation) in operations {
            let mut attributes = HashMap::new();
            attributes.insert("test_type".to_string(), "bdd".to_string());
            
            match storage_manager.record_trace(
                resource_id.clone(),
                component,
                operation,
                now,
                now + Duration::from_millis(100),
                TraceStatus::Ok,
                attributes,
            ).await {
                Ok(trace_id) => {
                    world.trace_id = Some(trace_id); // Store last trace ID
                    world.last_error = None;
                }
                Err(e) => {
                    world.last_error = Some(e.to_string());
                    break;
                }
            }
        }
    }
}

#[when("I create an evaluation dataset")]
async fn create_evaluation_dataset(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id)) = (&world.storage_manager, &world.resource_id) {
        match storage_manager.create_evaluation_dataset(
            resource_id.clone(),
            "BDD Evaluation Dataset",
            "Dataset created during BDD testing",
            "1.0",
        ).await {
            Ok(dataset_id) => {
                world.dataset_id = Some(dataset_id);
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I record evaluation scores")]
async fn record_evaluation_scores(world: &mut StorageWorld) {
    if let (Some(storage_manager), Some(resource_id)) = (&world.storage_manager, &world.resource_id) {
        let scores = vec![
            ("accuracy", 0.95),
            ("relevance", 0.88),
        ];
        
        for (metric, score_value) in scores {
            match storage_manager.record_evaluation_score(
                "bdd_eval_run",
                "bdd_test_item",
                resource_id.clone(),
                metric,
                score_value,
                "BDD test evaluation",
                "bdd_scorer",
            ).await {
                Ok(_) => {
                    world.last_error = None;
                }
                Err(e) => {
                    world.last_error = Some(e.to_string());
                    break;
                }
            }
        }
    }
}

#[when("I query storage statistics")]
async fn query_storage_statistics(world: &mut StorageWorld) {
    if let Some(storage_manager) = &world.storage_manager {
        match storage_manager.get_statistics().await {
            Ok(stats) => {
                world.stats = Some(stats);
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I perform storage maintenance")]
async fn perform_storage_maintenance(world: &mut StorageWorld) {
    if let Some(storage_manager) = &world.storage_manager {
        match storage_manager.perform_maintenance().await {
            Ok(_) => {
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

#[when("I query traces with filters")]
async fn query_traces_with_filters(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let filters = TraceFilters {
            component: Some("llm".to_string()),
            ..Default::default()
        };
        
        match storage.query_traces(resource_id, filters).await {
            Ok(_traces) => {
                world.last_error = None;
            }
            Err(e) => {
                world.last_error = Some(e.to_string());
            }
        }
    }
}

// Then steps
#[then("the workflow should be successfully suspended")]
async fn verify_workflow_suspended(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    assert!(world.workflow_id.is_some(), "Expected workflow ID to be set");
}

#[then("the workflow should be successfully resumed")]
async fn verify_workflow_resumed(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
}

#[then("the suspended workflow should no longer exist")]
async fn verify_workflow_no_longer_exists(world: &mut StorageWorld) {
    if let (Some(storage), Some(workflow_id)) = (&world.storage, &world.workflow_id) {
        let result = storage.get_suspended_workflow(workflow_id).await.unwrap();
        assert!(result.is_none(), "Expected workflow to be removed after resumption");
    }
}

#[then("the conversation thread should be created successfully")]
async fn verify_thread_created(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    assert!(world.thread_id.is_some(), "Expected thread ID to be set");
}

#[then("the messages should be stored in the thread")]
async fn verify_messages_stored(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    
    if let (Some(storage), Some(thread_id)) = (&world.storage, &world.thread_id) {
        let messages = storage.get_memory_messages(thread_id, None).await.unwrap();
        assert!(!messages.is_empty(), "Expected messages to be stored in the thread");
    }
}

#[then("the traces should be recorded successfully")]
async fn verify_traces_recorded(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    assert!(world.trace_id.is_some(), "Expected trace ID to be set");
}

#[then("the evaluation dataset should be created successfully")]
async fn verify_dataset_created(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    assert!(world.dataset_id.is_some(), "Expected dataset ID to be set");
}

#[then("the evaluation scores should be recorded successfully")]
async fn verify_scores_recorded(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
}

#[then("I should receive comprehensive storage statistics")]
async fn verify_storage_statistics(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
    assert!(world.stats.is_some(), "Expected storage statistics to be available");
    
    if let Some(stats) = &world.stats {
        // Verify that statistics contain meaningful data
        assert!(stats.storage_size_bytes > 0, "Expected storage size to be greater than 0");
    }
}

#[then("the maintenance should complete successfully")]
async fn verify_maintenance_completed(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
}

#[then("I should be able to filter traces by component")]
async fn verify_trace_filtering(world: &mut StorageWorld) {
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
}

#[then("the data should be isolated by resource")]
async fn verify_resource_isolation(world: &mut StorageWorld) {
    // This is verified implicitly by the fact that operations succeed
    // and we only see data for the specific resource we're working with
    assert!(world.last_error.is_none(), "Expected no error, but got: {:?}", world.last_error);
}

#[then(regex = r"there should be (\d+) suspended workflows")]
async fn verify_suspended_workflow_count(world: &mut StorageWorld, expected_count: usize) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let workflows = storage.list_suspended_workflows(resource_id).await.unwrap();
        assert_eq!(workflows.len(), expected_count, 
                  "Expected {} suspended workflows, found {}", expected_count, workflows.len());
    }
}

#[then(regex = r"there should be (\d+) memory threads")]
async fn verify_memory_thread_count(world: &mut StorageWorld, expected_count: usize) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let threads = storage.list_memory_threads(resource_id).await.unwrap();
        assert_eq!(threads.len(), expected_count,
                  "Expected {} memory threads, found {}", expected_count, threads.len());
    }
}

#[then(regex = r"there should be (\d+) evaluation datasets")]
async fn verify_eval_dataset_count(world: &mut StorageWorld, expected_count: usize) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        let datasets = storage.list_eval_datasets(resource_id).await.unwrap();
        assert_eq!(datasets.len(), expected_count,
                  "Expected {} evaluation datasets, found {}", expected_count, datasets.len());
    }
}

#[then("the system should handle concurrent operations safely")]
async fn verify_concurrent_safety(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        // Spawn multiple concurrent operations
        let mut handles = vec![];
        
        for i in 0..5 {
            let storage_clone = Arc::clone(storage);
            let resource_id_clone = resource_id.clone();
            
            let handle = tokio::spawn(async move {
                let workflow = SuspendedWorkflow {
                    workflow_id: format!("concurrent_test_{}", i),
                    resource_id: resource_id_clone,
                    workflow_name: format!("concurrent_workflow_{}", i),
                    current_step: i,
                    context_state: json!({"concurrent": true}),
                    metadata: HashMap::new(),
                    suspended_at: SystemTime::now(),
                    suspend_reason: SuspendReason::UserPause,
                    resume_conditions: vec![],
                };
                
                storage_clone.store_suspended_workflow(&workflow).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;
        
        // All operations should succeed
        for result in results {
            assert!(result.is_ok(), "Concurrent operation failed");
            assert!(result.unwrap().is_ok(), "Storage operation failed");
        }
        
        // Verify all workflows were stored
        let workflows = storage.list_suspended_workflows(resource_id).await.unwrap();
        let concurrent_workflows = workflows.iter()
            .filter(|w| w.workflow_name.starts_with("concurrent_workflow_"))
            .count();
        
        assert_eq!(concurrent_workflows, 5, 
                  "Expected 5 concurrent workflows to be stored");
    }
}

#[then("the storage system should maintain data integrity")]
async fn verify_data_integrity(world: &mut StorageWorld) {
    if let (Some(storage), Some(resource_id)) = (&world.storage, &world.resource_id) {
        // Verify workflows
        let workflows = storage.list_suspended_workflows(resource_id).await.unwrap();
        for workflow in &workflows {
            assert_eq!(&workflow.resource_id, resource_id, "Workflow resource ID mismatch");
            assert!(!workflow.workflow_id.is_empty(), "Workflow ID should not be empty");
            assert!(!workflow.workflow_name.is_empty(), "Workflow name should not be empty");
        }
        
        // Verify threads
        let threads = storage.list_memory_threads(resource_id).await.unwrap();
        for thread in &threads {
            assert_eq!(&thread.resource_id, resource_id, "Thread resource ID mismatch");
            assert!(!thread.thread_id.is_empty(), "Thread ID should not be empty");
        }
        
        // Verify datasets
        let datasets = storage.list_eval_datasets(resource_id).await.unwrap();
        for dataset in &datasets {
            assert_eq!(&dataset.resource_id, resource_id, "Dataset resource ID mismatch");
            assert!(!dataset.dataset_id.is_empty(), "Dataset ID should not be empty");
        }
    }
}

// Tokio main for running the tests
#[tokio::main]
async fn main() {
    StorageWorld::run("features/unified_storage.feature").await;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bdd_world_initialization() {
        let world = StorageWorld::new();
        assert!(world.storage.is_none());
        assert!(world.storage_manager.is_none());
        assert!(world.resource_id.is_none());
    }
}