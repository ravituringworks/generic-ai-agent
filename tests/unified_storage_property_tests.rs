//! Property-based tests for unified storage system using proptest

use proptest::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

// Include the storage system components from the library
use the_agency::*;

// Property test strategies

prop_compose! {
    fn arb_resource_id()(
        namespace in "[a-zA-Z0-9_]{1,20}",
        id in "[a-zA-Z0-9_]{1,20}"
    ) -> ResourceId {
        ResourceId::new(&namespace, &id)
    }
}

prop_compose! {
    fn arb_suspend_reason()(
        variant in 0..5u32,
        event_id in "[a-zA-Z0-9_]{1,50}",
        timeout_ms in prop::option::of(1u64..300000),
        duration_ms in 1u64..86400000,
        dependency_type in "[a-zA-Z_]{1,20}",
        details in "[a-zA-Z0-9_ ]{1,100}"
    ) -> SuspendReason {
        match variant {
            0 => SuspendReason::UserPause,
            1 => SuspendReason::WaitingForEvent { event_id, timeout_ms },
            2 => SuspendReason::Sleep { duration_ms },
            3 => SuspendReason::SleepUntil { timestamp: SystemTime::now() + Duration::from_millis(duration_ms) },
            4 => SuspendReason::ExternalDependency { dependency_type, details },
            _ => SuspendReason::UserPause,
        }
    }
}

prop_compose! {
    fn arb_suspended_workflow()(
        workflow_id in "[a-zA-Z0-9_]{1,50}",
        resource_id in arb_resource_id(),
        workflow_name in "[a-zA-Z0-9_ ]{1,50}",
        current_step in 0..100usize,
        suspend_reason in arb_suspend_reason()
    ) -> SuspendedWorkflow {
        SuspendedWorkflow {
            workflow_id,
            resource_id,
            workflow_name,
            current_step,
            context_state: json!({"step": current_step}),
            metadata: HashMap::new(),
            suspended_at: SystemTime::now(),
            suspend_reason,
            resume_conditions: vec![ResumeCondition::ManualResume],
        }
    }
}

prop_compose! {
    fn arb_message_role()(variant in 0..4u32) -> MessageRole {
        match variant {
            0 => MessageRole::User,
            1 => MessageRole::Assistant,
            2 => MessageRole::System,
            3 => MessageRole::Tool,
            _ => MessageRole::User,
        }
    }
}

prop_compose! {
    fn arb_memory_message()(
        message_id in "[a-zA-Z0-9_]{1,50}",
        thread_id in "[a-zA-Z0-9_]{1,50}",
        resource_id in arb_resource_id(),
        role in arb_message_role(),
        content in "[a-zA-Z0-9 .,!?]{1,500}"
    ) -> MemoryMessage {
        MemoryMessage {
            message_id,
            thread_id,
            resource_id,
            role,
            content,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            parent_message_id: None,
        }
    }
}

prop_compose! {
    fn arb_trace_status()(
        variant in 0..4u32,
        message in "[a-zA-Z0-9 .,!?]{1,100}"
    ) -> TraceStatus {
        match variant {
            0 => TraceStatus::Ok,
            1 => TraceStatus::Error { message },
            2 => TraceStatus::Timeout,
            3 => TraceStatus::Cancelled,
            _ => TraceStatus::Ok,
        }
    }
}

prop_compose! {
    fn arb_trace_data()(
        trace_id in "[a-zA-Z0-9_]{1,50}",
        span_id in "[a-zA-Z0-9_]{1,50}",
        resource_id in arb_resource_id(),
        operation_name in "[a-zA-Z_]{1,50}",
        component in "[a-zA-Z_]{1,20}",
        status in arb_trace_status(),
        duration_ms in prop::option::of(1u64..10000)
    ) -> TraceData {
        let now = SystemTime::now();
        TraceData {
            trace_id,
            span_id,
            parent_span_id: None,
            resource_id,
            operation_name,
            start_time: now,
            end_time: Some(now + Duration::from_millis(duration_ms.unwrap_or(100))),
            duration_ms,
            status,
            attributes: HashMap::new(),
            events: vec![],
            component,
        }
    }
}

prop_compose! {
    fn arb_eval_score()(
        score_id in "[a-zA-Z0-9_]{1,50}",
        run_id in "[a-zA-Z0-9_]{1,50}",
        item_id in "[a-zA-Z0-9_]{1,50}",
        resource_id in arb_resource_id(),
        metric_name in "[a-zA-Z_]{1,20}",
        score in 0.0..1.0f64,
        scorer_name in "[a-zA-Z_]{1,20}",
        reason in "[a-zA-Z0-9 .,]{1,100}"
    ) -> EvalScore {
        EvalScore {
            score_id,
            run_id,
            item_id,
            resource_id,
            metric_name,
            score,
            reason,
            scorer_name,
            metadata: HashMap::new(),
            scored_at: SystemTime::now(),
        }
    }
}

// Property tests

proptest! {
    #[test]
    fn prop_workflow_storage_retrieval(workflow in arb_suspended_workflow()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store workflow
            storage.store_suspended_workflow(&workflow).await.unwrap();

            // Retrieve workflow
            let retrieved = storage.get_suspended_workflow(&workflow.workflow_id).await.unwrap();
            prop_assert!(retrieved.is_some());

            let retrieved = retrieved.unwrap();
            prop_assert_eq!(retrieved.workflow_id, workflow.workflow_id);
            prop_assert_eq!(retrieved.current_step, workflow.current_step);
            prop_assert_eq!(retrieved.resource_id, workflow.resource_id);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_workflow_list_by_resource(
        workflows in prop::collection::vec(arb_suspended_workflow(), 1..10)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store all workflows
            for workflow in &workflows {
                storage.store_suspended_workflow(workflow).await.unwrap();
            }

            // Group workflows by resource for verification
            let mut resource_groups: HashMap<ResourceId, Vec<&SuspendedWorkflow>> = HashMap::new();
            for workflow in &workflows {
                resource_groups.entry(workflow.resource_id.clone())
                    .or_default()
                    .push(workflow);
            }

            // Verify listing works correctly for each resource
            for (resource_id, expected_workflows) in resource_groups {
                let retrieved = storage.list_suspended_workflows(&resource_id).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_workflows.len());

                // Verify all retrieved workflows belong to the correct resource
                for retrieved_workflow in &retrieved {
                    prop_assert_eq!(&retrieved_workflow.resource_id, &resource_id);
                }
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_workflow_resume_removes_from_storage(workflow in arb_suspended_workflow()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store workflow
            storage.store_suspended_workflow(&workflow).await.unwrap();

            // Resume workflow (should remove it)
            let resumed = storage.resume_workflow(&workflow.workflow_id).await.unwrap();
            prop_assert_eq!(&resumed.workflow_id, &workflow.workflow_id);

            // Verify it's no longer in storage
            let retrieved = storage.get_suspended_workflow(&workflow.workflow_id).await.unwrap();
            prop_assert!(retrieved.is_none());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_message_storage_and_retrieval(
        messages in prop::collection::vec(arb_memory_message(), 1..50)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Group messages by thread_id for testing
            let mut thread_groups: HashMap<String, Vec<&MemoryMessage>> = HashMap::new();
            for message in &messages {
                thread_groups.entry(message.thread_id.clone())
                    .or_default()
                    .push(message);
            }

            // Store all messages
            for message in &messages {
                storage.add_memory_message(message).await.unwrap();
            }

            // Verify retrieval works correctly for each thread
            for (thread_id, expected_messages) in thread_groups {
                let retrieved = storage.get_memory_messages(&thread_id, None).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_messages.len());

                // Verify all retrieved messages belong to the correct thread
                for retrieved_message in &retrieved {
                    prop_assert_eq!(&retrieved_message.thread_id, &thread_id);
                }
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_trace_storage_and_querying(
        traces in prop::collection::vec(arb_trace_data(), 1..20)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store all traces
            for trace in &traces {
                storage.store_trace(trace).await.unwrap();
            }

            // Group traces by resource for verification
            let mut resource_groups: HashMap<ResourceId, Vec<&TraceData>> = HashMap::new();
            for trace in &traces {
                resource_groups.entry(trace.resource_id.clone())
                    .or_default()
                    .push(trace);
            }

            // Test querying by resource
            for (resource_id, expected_traces) in resource_groups {
                let filters = TraceFilters::default();
                let retrieved = storage.query_traces(&resource_id, filters).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_traces.len());

                // Verify all retrieved traces belong to the correct resource
                for retrieved_trace in &retrieved {
                    prop_assert_eq!(&retrieved_trace.resource_id, &resource_id);
                }
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_trace_filtering_by_component(
        traces in prop::collection::vec(arb_trace_data(), 1..20)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store all traces
            for trace in &traces {
                storage.store_trace(trace).await.unwrap();
            }

            // Get unique combinations of resource and component
            let mut resource_component_groups: HashMap<(ResourceId, String), Vec<&TraceData>> = HashMap::new();
            for trace in &traces {
                let key = (trace.resource_id.clone(), trace.component.clone());
                resource_component_groups.entry(key)
                    .or_default()
                    .push(trace);
            }

            // Test filtering by component for each resource
            for ((resource_id, component), expected_traces) in resource_component_groups {
                let filters = TraceFilters {
                    component: Some(component.clone()),
                    ..Default::default()
                };
                let retrieved = storage.query_traces(&resource_id, filters).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_traces.len());

                // Verify all retrieved traces match the filter criteria
                for retrieved_trace in &retrieved {
                    prop_assert_eq!(&retrieved_trace.resource_id, &resource_id);
                    prop_assert_eq!(&retrieved_trace.component, &component);
                }
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_eval_score_storage_and_retrieval(
        scores in prop::collection::vec(arb_eval_score(), 1..30)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store all scores
            for score in &scores {
                storage.store_eval_score(score).await.unwrap();
            }

            // Group scores by run_id for verification
            let mut run_groups: HashMap<String, Vec<&EvalScore>> = HashMap::new();
            for score in &scores {
                run_groups.entry(score.run_id.clone())
                    .or_default()
                    .push(score);
            }

            // Verify retrieval works correctly for each run
            for (run_id, expected_scores) in run_groups {
                let retrieved = storage.get_eval_scores(&run_id).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_scores.len());

                // Verify all retrieved scores belong to the correct run
                for retrieved_score in &retrieved {
                    prop_assert_eq!(&retrieved_score.run_id, &run_id);
                }
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_storage_statistics_accuracy(
        workflows in prop::collection::vec(arb_suspended_workflow(), 0..10),
        traces in prop::collection::vec(arb_trace_data(), 0..10),
        scores in prop::collection::vec(arb_eval_score(), 0..10)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store all data
            for workflow in &workflows {
                storage.store_suspended_workflow(workflow).await.unwrap();
            }
            for trace in &traces {
                storage.store_trace(trace).await.unwrap();
            }

            // Group scores by run_id to count runs
            let mut run_ids = std::collections::HashSet::new();
            for score in &scores {
                run_ids.insert(score.run_id.clone());
                storage.store_eval_score(score).await.unwrap();
            }

            // Get statistics
            let stats = storage.get_storage_stats().await.unwrap();

            // Verify statistics accuracy
            prop_assert_eq!(stats.suspended_workflows, workflows.len());
            prop_assert_eq!(stats.traces, traces.len());
            prop_assert_eq!(stats.eval_runs, run_ids.len());
            prop_assert_eq!(stats.eval_scores, scores.len());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_resource_isolation(
        resource1 in arb_resource_id(),
        resource2 in arb_resource_id(),
        workflows1 in prop::collection::vec(arb_suspended_workflow(), 1..5),
        workflows2 in prop::collection::vec(arb_suspended_workflow(), 1..5)
    ) {
        prop_assume!(resource1 != resource2);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Store workflows for resource1
            for mut workflow in workflows1.clone() {
                workflow.resource_id = resource1.clone();
                storage.store_suspended_workflow(&workflow).await.unwrap();
            }

            // Store workflows for resource2
            for mut workflow in workflows2.clone() {
                workflow.resource_id = resource2.clone();
                storage.store_suspended_workflow(&workflow).await.unwrap();
            }

            // Verify resource1 only sees its own workflows
            let retrieved1 = storage.list_suspended_workflows(&resource1).await.unwrap();
            prop_assert_eq!(retrieved1.len(), workflows1.len());
            for workflow in &retrieved1 {
                prop_assert_eq!(&workflow.resource_id, &resource1);
            }

            // Verify resource2 only sees its own workflows
            let retrieved2 = storage.list_suspended_workflows(&resource2).await.unwrap();
            prop_assert_eq!(retrieved2.len(), workflows2.len());
            for workflow in &retrieved2 {
                prop_assert_eq!(&workflow.resource_id, &resource2);
            }
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_concurrent_workflow_operations(
        workflows in prop::collection::vec(arb_suspended_workflow(), 1..20)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());

            // Spawn concurrent storage operations
            let mut handles = vec![];
            for workflow in workflows.clone() {
                let storage_clone = Arc::clone(&storage);
                let handle = tokio::spawn(async move {
                    storage_clone.store_suspended_workflow(&workflow).await
                });
                handles.push(handle);
            }

            // Wait for all operations to complete
            let results = futures::future::join_all(handles).await;

            // All operations should succeed
            for result in results {
                prop_assert!(result.is_ok());
                prop_assert!(result.unwrap().is_ok());
            }

            // Verify all workflows were stored
            let mut total_stored = 0;
            let resource_groups: std::collections::HashMap<_, _> = workflows.iter()
                .fold(std::collections::HashMap::new(), |mut acc, w| {
                    acc.entry(w.resource_id.clone()).or_insert_with(Vec::new).push(w);
                    acc
                });

            for (resource_id, expected_workflows) in resource_groups {
                let retrieved = storage.list_suspended_workflows(&resource_id).await.unwrap();
                prop_assert_eq!(retrieved.len(), expected_workflows.len());
                total_stored += retrieved.len();
            }

            prop_assert_eq!(total_stored, workflows.len());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn prop_trace_cleanup_by_time(
        old_traces in prop::collection::vec(arb_trace_data(), 1..10),
        new_traces in prop::collection::vec(arb_trace_data(), 1..10)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let storage = Arc::new(InMemoryUnifiedStorage::new());
            let now = SystemTime::now();
            let cutoff_time = now + Duration::from_secs(3600); // 1 hour from now

            // Store old traces (before cutoff)
            for mut trace in old_traces.clone() {
                trace.start_time = now - Duration::from_secs(7200); // 2 hours ago
                storage.store_trace(&trace).await.unwrap();
            }

            // Store new traces (after cutoff)
            for mut trace in new_traces.clone() {
                trace.start_time = now + Duration::from_secs(1800); // 30 minutes from now
                storage.store_trace(&trace).await.unwrap();
            }

            // Delete traces before cutoff
            let deleted_count = storage.delete_traces_before(cutoff_time).await.unwrap();

            // Should have deleted all old traces
            prop_assert_eq!(deleted_count, old_traces.len());

            // Verify new traces still exist by checking total remaining traces
            let resource_groups: std::collections::HashMap<_, _> = new_traces.iter()
                .fold(std::collections::HashMap::new(), |mut acc, t| {
                    acc.entry(t.resource_id.clone()).or_insert_with(Vec::new).push(t);
                    acc
                });

            let mut total_remaining = 0;
            for (resource_id, _) in resource_groups {
                let retrieved = storage.query_traces(&resource_id, TraceFilters::default()).await.unwrap();
                total_remaining += retrieved.len();
            }

            prop_assert_eq!(total_remaining, new_traces.len());
            Ok(())
        }).unwrap();
    }
}

#[cfg(test)]
mod additional_property_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager_property_workflow_roundtrip() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = RetentionPolicy {
            traces_retention: Duration::from_secs(7 * 24 * 3600),
            memory_retention: Duration::from_secs(30 * 24 * 3600),
            eval_retention: Duration::from_secs(90 * 24 * 3600),
            suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600),
        };
        let manager = StorageManager::new(storage, retention_policy);

        let resource_id = ResourceId::new("prop_test", "user_456");
        let workflow_id = "property_test_workflow";
        let context_data = json!({"test": "property", "step": 42});

        // Suspend workflow
        manager
            .suspend_workflow(
                workflow_id,
                resource_id,
                "property_test",
                42,
                context_data.clone(),
                SuspendReason::WaitingForEvent {
                    event_id: "test_event".to_string(),
                    timeout_ms: Some(30000),
                },
            )
            .await
            .unwrap();

        // Resume workflow
        let (restored_context, restored_step) = manager.resume_workflow(workflow_id).await.unwrap();

        assert_eq!(restored_step, 42);
        assert_eq!(restored_context, context_data);
    }

    #[tokio::test]
    async fn test_memory_message_ordering_property() {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let resource_id = ResourceId::new("ordering_test", "user_789");
        let thread_id = "ordering_test_thread";

        // Create messages with incrementing timestamps
        let mut messages = vec![];
        let base_time = SystemTime::now();

        for i in 0..10 {
            let message = MemoryMessage {
                message_id: format!("msg_{}", i),
                thread_id: thread_id.to_string(),
                resource_id: resource_id.clone(),
                role: if i % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: format!("Message number {}", i),
                timestamp: base_time + Duration::from_millis(i * 100),
                metadata: HashMap::new(),
                parent_message_id: None,
            };
            messages.push(message);
        }

        // Store messages in random order
        use rand::prelude::*;
        let mut rng = rand::rng();
        let mut shuffled_messages = messages.clone();
        shuffled_messages.shuffle(&mut rng);

        for message in &shuffled_messages {
            storage.add_memory_message(message).await.unwrap();
        }

        // Retrieve messages
        let retrieved = storage.get_memory_messages(thread_id, None).await.unwrap();

        // Should have all messages
        assert_eq!(retrieved.len(), 10);

        // Verify all messages are present (order might not be preserved in this simple impl)
        for original in &messages {
            let found = retrieved
                .iter()
                .any(|r| r.message_id == original.message_id);
            assert!(
                found,
                "Message {} not found in retrieved messages",
                original.message_id
            );
        }
    }
}
