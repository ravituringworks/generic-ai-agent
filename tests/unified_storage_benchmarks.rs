//! Benchmark tests for unified storage system performance evaluation

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;

// Include the storage system components
use crate::unified_storage_tests::*;

pub struct BenchmarkResults {
    pub operation_name: String,
    pub total_operations: usize,
    pub total_duration: Duration,
    pub ops_per_second: f64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
}

impl BenchmarkResults {
    pub fn new(operation_name: String, total_operations: usize, latencies: Vec<Duration>) -> Self {
        let total_duration: Duration = latencies.iter().sum();
        let ops_per_second = total_operations as f64 / total_duration.as_secs_f64();

        let latency_ms: Vec<f64> = latencies.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        let avg_latency_ms = latency_ms.iter().sum::<f64>() / latency_ms.len() as f64;
        let min_latency_ms = latency_ms.iter().copied().fold(f64::INFINITY, f64::min);
        let max_latency_ms = latency_ms.iter().copied().fold(0.0, f64::max);

        Self {
            operation_name,
            total_operations,
            total_duration,
            ops_per_second,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
        }
    }

    pub fn print(&self) {
        println!("📊 Benchmark Results: {}", self.operation_name);
        println!("   Total Operations: {}", self.total_operations);
        println!("   Total Duration: {:?}", self.total_duration);
        println!("   Operations/Second: {:.2}", self.ops_per_second);
        println!("   Average Latency: {:.2}ms", self.avg_latency_ms);
        println!("   Min Latency: {:.2}ms", self.min_latency_ms);
        println!("   Max Latency: {:.2}ms", self.max_latency_ms);
        println!();
    }
}

pub struct StorageBenchmark {
    storage: Arc<dyn UnifiedStorage>,
    storage_manager: StorageManager,
}

impl StorageBenchmark {
    pub fn new() -> Self {
        let storage = Arc::new(InMemoryUnifiedStorage::new());
        let retention_policy = RetentionPolicy {
            traces_retention: Duration::from_secs(7 * 24 * 3600),
            memory_retention: Duration::from_secs(30 * 24 * 3600),
            eval_retention: Duration::from_secs(90 * 24 * 3600),
            suspended_workflows_retention: Duration::from_secs(365 * 24 * 3600),
        };
        let storage_manager = StorageManager::new(storage.clone(), retention_policy);

        Self {
            storage,
            storage_manager,
        }
    }

    // Benchmark workflow operations
    pub async fn benchmark_workflow_operations(
        &self,
        num_operations: usize,
    ) -> Vec<BenchmarkResults> {
        let mut results = Vec::new();
        let resource_id = ResourceId::new("benchmark", "workflow_test");

        // Benchmark workflow storage
        let mut latencies = Vec::new();
        for i in 0..num_operations {
            let workflow = SuspendedWorkflow {
                workflow_id: format!("benchmark_workflow_{}", i),
                resource_id: resource_id.clone(),
                workflow_name: "benchmark_workflow".to_string(),
                current_step: i,
                context_state: json!({"step": i, "data": format!("test_data_{}", i)}),
                metadata: HashMap::new(),
                suspended_at: SystemTime::now(),
                suspend_reason: SuspendReason::UserPause,
                resume_conditions: vec![ResumeCondition::ManualResume],
            };

            let start = Instant::now();
            self.storage
                .store_suspended_workflow(&workflow)
                .await
                .unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Workflow Storage".to_string(),
            num_operations,
            latencies,
        ));

        // Benchmark workflow retrieval
        let mut latencies = Vec::new();
        for i in 0..num_operations {
            let workflow_id = format!("benchmark_workflow_{}", i);

            let start = Instant::now();
            let _retrieved = self
                .storage
                .get_suspended_workflow(&workflow_id)
                .await
                .unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Workflow Retrieval".to_string(),
            num_operations,
            latencies,
        ));

        // Benchmark workflow listing
        let start = Instant::now();
        let _workflows = self
            .storage
            .list_suspended_workflows(&resource_id)
            .await
            .unwrap();
        let list_duration = start.elapsed();
        results.push(BenchmarkResults::new(
            "Workflow Listing".to_string(),
            1,
            vec![list_duration],
        ));

        results
    }

    // Benchmark memory operations
    pub async fn benchmark_memory_operations(
        &self,
        num_threads: usize,
        messages_per_thread: usize,
    ) -> Vec<BenchmarkResults> {
        let mut results = Vec::new();
        let resource_id = ResourceId::new("benchmark", "memory_test");

        // Create threads first
        let mut thread_ids = Vec::new();
        let mut latencies = Vec::new();

        for i in 0..num_threads {
            let thread = MemoryThread {
                thread_id: format!("benchmark_thread_{}", i),
                resource_id: resource_id.clone(),
                title: format!("Benchmark Thread {}", i),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                metadata: HashMap::new(),
                message_count: 0,
            };

            let start = Instant::now();
            self.storage.create_memory_thread(&thread).await.unwrap();
            latencies.push(start.elapsed());

            thread_ids.push(thread.thread_id.clone());
        }
        results.push(BenchmarkResults::new(
            "Thread Creation".to_string(),
            num_threads,
            latencies,
        ));

        // Benchmark message storage
        let mut latencies = Vec::new();
        for (thread_idx, thread_id) in thread_ids.iter().enumerate() {
            for msg_idx in 0..messages_per_thread {
                let message = MemoryMessage {
                    message_id: format!("benchmark_msg_{}_{}", thread_idx, msg_idx),
                    thread_id: thread_id.clone(),
                    resource_id: resource_id.clone(),
                    role: if msg_idx % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    content: format!("Benchmark message {} in thread {}", msg_idx, thread_idx),
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                    parent_message_id: None,
                };

                let start = Instant::now();
                self.storage.add_memory_message(&message).await.unwrap();
                latencies.push(start.elapsed());
            }
        }
        results.push(BenchmarkResults::new(
            "Message Storage".to_string(),
            num_threads * messages_per_thread,
            latencies,
        ));

        // Benchmark message retrieval
        let mut latencies = Vec::new();
        for thread_id in &thread_ids {
            let start = Instant::now();
            let _messages = self
                .storage
                .get_memory_messages(thread_id, None)
                .await
                .unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Message Retrieval".to_string(),
            num_threads,
            latencies,
        ));

        results
    }

    // Benchmark trace operations
    pub async fn benchmark_trace_operations(&self, num_traces: usize) -> Vec<BenchmarkResults> {
        let mut results = Vec::new();
        let resource_id = ResourceId::new("benchmark", "trace_test");
        let components = vec!["llm", "memory", "workflow", "agent", "tools"];

        // Benchmark trace storage
        let mut latencies = Vec::new();
        for i in 0..num_traces {
            let component = &components[i % components.len()];
            let now = SystemTime::now();
            let trace = TraceData {
                trace_id: format!("benchmark_trace_{}", i),
                span_id: format!("benchmark_span_{}", i),
                parent_span_id: None,
                resource_id: resource_id.clone(),
                operation_name: format!("{}_operation", component),
                start_time: now,
                end_time: Some(now + Duration::from_millis(10 + (i as u64 % 100))),
                duration_ms: Some(10 + (i as u64 % 100)),
                status: if i % 10 == 0 {
                    TraceStatus::Error {
                        message: format!("Test error {}", i),
                    }
                } else {
                    TraceStatus::Ok
                },
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("benchmark_id".to_string(), i.to_string());
                    attrs.insert("component".to_string(), component.to_string());
                    attrs
                },
                events: vec![],
                component: component.to_string(),
            };

            let start = Instant::now();
            self.storage.store_trace(&trace).await.unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Trace Storage".to_string(),
            num_traces,
            latencies,
        ));

        // Benchmark trace querying (all traces)
        let start = Instant::now();
        let _traces = self
            .storage
            .query_traces(&resource_id, TraceFilters::default())
            .await
            .unwrap();
        let query_duration = start.elapsed();
        results.push(BenchmarkResults::new(
            "Trace Query (All)".to_string(),
            1,
            vec![query_duration],
        ));

        // Benchmark trace querying with filters
        let mut latencies = Vec::new();
        for component in &components {
            let filters = TraceFilters {
                component: Some(component.to_string()),
                ..Default::default()
            };

            let start = Instant::now();
            let _traces = self
                .storage
                .query_traces(&resource_id, filters)
                .await
                .unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Trace Query (Filtered)".to_string(),
            components.len(),
            latencies,
        ));

        results
    }

    // Benchmark evaluation operations
    pub async fn benchmark_evaluation_operations(
        &self,
        num_datasets: usize,
        num_scores: usize,
    ) -> Vec<BenchmarkResults> {
        let mut results = Vec::new();
        let resource_id = ResourceId::new("benchmark", "eval_test");

        // Benchmark dataset creation
        let mut dataset_ids = Vec::new();
        let mut latencies = Vec::new();

        for i in 0..num_datasets {
            let dataset = EvalDataset {
                dataset_id: format!("benchmark_dataset_{}", i),
                name: format!("Benchmark Dataset {}", i),
                description: format!("Dataset {} for benchmarking", i),
                resource_id: resource_id.clone(),
                created_at: SystemTime::now(),
                version: format!("1.{}", i),
                metadata: HashMap::new(),
            };

            let start = Instant::now();
            self.storage.create_eval_dataset(&dataset).await.unwrap();
            latencies.push(start.elapsed());

            dataset_ids.push(dataset.dataset_id.clone());
        }
        results.push(BenchmarkResults::new(
            "Dataset Creation".to_string(),
            num_datasets,
            latencies,
        ));

        // Benchmark score storage
        let metrics = vec![
            "accuracy",
            "precision",
            "recall",
            "f1_score",
            "bleu",
            "rouge",
        ];
        let mut latencies = Vec::new();

        for i in 0..num_scores {
            let metric = &metrics[i % metrics.len()];
            let score = EvalScore {
                score_id: format!("benchmark_score_{}", i),
                run_id: format!("benchmark_run_{}", i / 10), // Group scores into runs
                item_id: format!("benchmark_item_{}", i),
                resource_id: resource_id.clone(),
                metric_name: metric.to_string(),
                score: 0.5 + (i as f64 * 0.001) % 0.5, // Vary scores between 0.5 and 1.0
                reason: format!("Benchmark score {} for metric {}", i, metric),
                scorer_name: "benchmark_scorer".to_string(),
                metadata: HashMap::new(),
                scored_at: SystemTime::now(),
            };

            let start = Instant::now();
            self.storage.store_eval_score(&score).await.unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Score Storage".to_string(),
            num_scores,
            latencies,
        ));

        // Benchmark score retrieval
        let num_runs = (num_scores / 10) + 1;
        let mut latencies = Vec::new();

        for i in 0..num_runs {
            let run_id = format!("benchmark_run_{}", i);

            let start = Instant::now();
            let _scores = self.storage.get_eval_scores(&run_id).await.unwrap();
            latencies.push(start.elapsed());
        }
        results.push(BenchmarkResults::new(
            "Score Retrieval".to_string(),
            num_runs,
            latencies,
        ));

        results
    }

    // Benchmark concurrent operations
    pub async fn benchmark_concurrent_operations(
        &self,
        num_concurrent: usize,
        operations_per_task: usize,
    ) -> BenchmarkResults {
        let resource_id = ResourceId::new("benchmark", "concurrent_test");

        let start_time = Instant::now();
        let mut handles = Vec::new();

        for task_id in 0..num_concurrent {
            let storage_clone = Arc::clone(&self.storage);
            let resource_id_clone = resource_id.clone();

            let handle = tokio::spawn(async move {
                for op_id in 0..operations_per_task {
                    let workflow = SuspendedWorkflow {
                        workflow_id: format!("concurrent_{}_{}", task_id, op_id),
                        resource_id: resource_id_clone.clone(),
                        workflow_name: format!("concurrent_workflow_{}_{}", task_id, op_id),
                        current_step: op_id,
                        context_state: json!({"task_id": task_id, "op_id": op_id}),
                        metadata: HashMap::new(),
                        suspended_at: SystemTime::now(),
                        suspend_reason: SuspendReason::UserPause,
                        resume_conditions: vec![],
                    };

                    storage_clone
                        .store_suspended_workflow(&workflow)
                        .await
                        .unwrap();
                }
                task_id
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        let _results = futures::future::join_all(handles).await;
        let total_duration = start_time.elapsed();

        let total_operations = num_concurrent * operations_per_task;
        BenchmarkResults::new(
            "Concurrent Workflow Storage".to_string(),
            total_operations,
            vec![total_duration],
        )
    }

    // Benchmark storage statistics retrieval
    pub async fn benchmark_storage_statistics(&self, num_iterations: usize) -> BenchmarkResults {
        let mut latencies = Vec::new();

        for _ in 0..num_iterations {
            let start = Instant::now();
            let _stats = self.storage.get_storage_stats().await.unwrap();
            latencies.push(start.elapsed());
        }

        BenchmarkResults::new("Storage Statistics".to_string(), num_iterations, latencies)
    }

    // Benchmark cleanup operations
    pub async fn benchmark_cleanup_operations(&self) -> BenchmarkResults {
        let retention_policy = RetentionPolicy {
            traces_retention: Duration::from_secs(3600), // 1 hour
            memory_retention: Duration::from_secs(7200), // 2 hours
            eval_retention: Duration::from_secs(14400),  // 4 hours
            suspended_workflows_retention: Duration::from_secs(86400), // 24 hours
        };

        let start = Instant::now();
        let _cleanup_stats = self
            .storage
            .cleanup_old_data(&retention_policy)
            .await
            .unwrap();
        let duration = start.elapsed();

        BenchmarkResults::new("Cleanup Operations".to_string(), 1, vec![duration])
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_operations_benchmark() {
        let benchmark = StorageBenchmark::new();
        let results = benchmark.benchmark_workflow_operations(100).await;

        for result in &results {
            result.print();
            // Assert reasonable performance (adjust thresholds as needed)
            assert!(
                result.ops_per_second > 1000.0,
                "Workflow operations should be > 1000 ops/sec"
            );
            assert!(
                result.avg_latency_ms < 10.0,
                "Average latency should be < 10ms"
            );
        }
    }

    #[tokio::test]
    async fn test_memory_operations_benchmark() {
        let benchmark = StorageBenchmark::new();
        let results = benchmark.benchmark_memory_operations(10, 50).await;

        for result in &results {
            result.print();
            // Memory operations should be fast
            assert!(
                result.avg_latency_ms < 5.0,
                "Memory operations should be < 5ms average latency"
            );
        }
    }

    #[tokio::test]
    async fn test_trace_operations_benchmark() {
        let benchmark = StorageBenchmark::new();
        let results = benchmark.benchmark_trace_operations(500).await;

        for result in &results {
            result.print();
            // Trace operations should handle high volume
            if result.operation_name.contains("Storage") {
                assert!(
                    result.ops_per_second > 5000.0,
                    "Trace storage should be > 5000 ops/sec"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_evaluation_operations_benchmark() {
        let benchmark = StorageBenchmark::new();
        let results = benchmark.benchmark_evaluation_operations(5, 100).await;

        for result in &results {
            result.print();
            // Evaluation operations should be reasonably fast
            assert!(
                result.avg_latency_ms < 5.0,
                "Evaluation operations should be < 5ms average"
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations_benchmark() {
        let benchmark = StorageBenchmark::new();
        let result = benchmark.benchmark_concurrent_operations(10, 100).await;

        result.print();
        // Concurrent operations should scale well
        assert!(
            result.ops_per_second > 1000.0,
            "Concurrent operations should be > 1000 ops/sec"
        );
    }

    #[tokio::test]
    async fn test_storage_statistics_benchmark() {
        let benchmark = StorageBenchmark::new();
        let result = benchmark.benchmark_storage_statistics(50).await;

        result.print();
        // Statistics should be very fast
        assert!(
            result.avg_latency_ms < 1.0,
            "Statistics should be < 1ms average"
        );
    }

    #[tokio::test]
    async fn test_full_benchmark_suite() {
        println!("🚀 Running Full Unified Storage Benchmark Suite");
        println!("================================================\n");

        let benchmark = StorageBenchmark::new();

        // Workflow benchmarks
        println!("📊 Workflow Operations Benchmark");
        let workflow_results = benchmark.benchmark_workflow_operations(1000).await;
        for result in workflow_results {
            result.print();
        }

        // Memory benchmarks
        println!("📊 Memory Operations Benchmark");
        let memory_results = benchmark.benchmark_memory_operations(20, 100).await;
        for result in memory_results {
            result.print();
        }

        // Trace benchmarks
        println!("📊 Trace Operations Benchmark");
        let trace_results = benchmark.benchmark_trace_operations(2000).await;
        for result in trace_results {
            result.print();
        }

        // Evaluation benchmarks
        println!("📊 Evaluation Operations Benchmark");
        let eval_results = benchmark.benchmark_evaluation_operations(10, 500).await;
        for result in eval_results {
            result.print();
        }

        // Concurrent operations benchmark
        println!("📊 Concurrent Operations Benchmark");
        let concurrent_result = benchmark.benchmark_concurrent_operations(20, 200).await;
        concurrent_result.print();

        // Statistics benchmark
        println!("📊 Storage Statistics Benchmark");
        let stats_result = benchmark.benchmark_storage_statistics(100).await;
        stats_result.print();

        // Cleanup benchmark
        println!("📊 Cleanup Operations Benchmark");
        let cleanup_result = benchmark.benchmark_cleanup_operations().await;
        cleanup_result.print();

        println!("✅ Full benchmark suite completed successfully!");
    }

    #[tokio::test]
    async fn test_load_test_scenario() {
        println!("🔥 Load Test Scenario - Simulating High-Volume Production Workload");
        println!("===================================================================\n");

        let benchmark = StorageBenchmark::new();

        // Simulate a high-load scenario
        let start_time = Instant::now();

        // Create a lot of concurrent operations
        let mut handles = Vec::new();

        // Spawn workflow tasks
        for i in 0..5 {
            let storage_clone = Arc::clone(&benchmark.storage);
            let resource_id = ResourceId::new("load_test", &format!("workflow_worker_{}", i));

            let handle = tokio::spawn(async move {
                for j in 0..200 {
                    let workflow = SuspendedWorkflow {
                        workflow_id: format!("load_test_workflow_{}_{}", i, j),
                        resource_id: resource_id.clone(),
                        workflow_name: format!("load_test_workflow_{}", i),
                        current_step: j,
                        context_state: json!({"worker": i, "operation": j}),
                        metadata: HashMap::new(),
                        suspended_at: SystemTime::now(),
                        suspend_reason: SuspendReason::UserPause,
                        resume_conditions: vec![],
                    };

                    storage_clone
                        .store_suspended_workflow(&workflow)
                        .await
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Spawn trace tasks
        for i in 0..3 {
            let storage_clone = Arc::clone(&benchmark.storage);
            let resource_id = ResourceId::new("load_test", &format!("trace_worker_{}", i));

            let handle = tokio::spawn(async move {
                for j in 0..500 {
                    let trace = TraceData {
                        trace_id: format!("load_test_trace_{}_{}", i, j),
                        span_id: format!("load_test_span_{}_{}", i, j),
                        parent_span_id: None,
                        resource_id: resource_id.clone(),
                        operation_name: "load_test_operation".to_string(),
                        start_time: SystemTime::now(),
                        end_time: Some(SystemTime::now() + Duration::from_millis(50)),
                        duration_ms: Some(50),
                        status: TraceStatus::Ok,
                        attributes: HashMap::new(),
                        events: vec![],
                        component: "load_test".to_string(),
                    };

                    storage_clone.store_trace(&trace).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        futures::future::join_all(handles).await;

        let total_duration = start_time.elapsed();
        let total_operations = (5 * 200) + (3 * 500); // 1000 workflow + 1500 trace = 2500 ops

        let result = BenchmarkResults::new(
            "Load Test Scenario".to_string(),
            total_operations,
            vec![total_duration],
        );

        result.print();

        // Load test should handle reasonable volume
        assert!(
            result.ops_per_second > 500.0,
            "Load test should achieve > 500 ops/sec"
        );
        assert!(
            result.total_duration.as_secs() < 30,
            "Load test should complete in < 30 seconds"
        );

        // Verify data integrity after load test
        let stats = benchmark.storage.get_storage_stats().await.unwrap();
        assert!(
            stats.suspended_workflows >= 1000,
            "Should have stored workflow operations"
        );
        assert!(stats.traces >= 1500, "Should have stored trace operations");

        println!(
            "✅ Load test passed - system handled {} operations in {:?}",
            total_operations, total_duration
        );
    }
}

// Example usage and integration tests
#[cfg(test)]
mod integration_benchmark_tests {
    use super::*;

    #[tokio::test]
    async fn test_realistic_application_scenario() {
        println!("🌟 Realistic Application Scenario Benchmark");
        println!("==========================================\n");

        let benchmark = StorageBenchmark::new();
        let resource_id = ResourceId::new("production_app", "user_session_123");

        // Simulate a realistic application flow
        let start_time = Instant::now();

        // 1. User starts a workflow
        benchmark
            .storage_manager
            .suspend_workflow(
                "user_onboarding_flow",
                resource_id.clone(),
                "user_onboarding",
                0,
                json!({"step": "welcome", "user_id": "123"}),
                SuspendReason::WaitingForEvent {
                    event_id: "user_input".to_string(),
                    timeout_ms: Some(300000),
                },
            )
            .await
            .unwrap();

        // 2. Create conversation thread
        let thread_id = benchmark
            .storage_manager
            .create_conversation_thread(resource_id.clone(), "Onboarding Assistant Chat")
            .await
            .unwrap();

        // 3. Simulate conversation
        let conversation_pairs = vec![
            ("Hello! Welcome to our platform.", "assistant"),
            ("Hi there! I'm excited to get started.", "user"),
            (
                "Great! Let's begin with setting up your profile.",
                "assistant",
            ),
            ("Sounds good. What information do you need?", "user"),
            ("I'll need your name, email, and preferences.", "assistant"),
            ("My name is John Doe, email is john@example.com", "user"),
            ("Perfect! I've saved that information.", "assistant"),
        ];

        for (content, role) in conversation_pairs {
            let message_role = match role {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::System,
            };

            benchmark
                .storage_manager
                .add_message(&thread_id, resource_id.clone(), message_role, content)
                .await
                .unwrap();
        }

        // 4. Record traces for various operations
        let operations = vec![
            ("llm", "generate_welcome_message", 150),
            ("memory", "store_user_profile", 25),
            ("workflow", "progress_to_next_step", 30),
            ("agent", "process_user_input", 75),
            ("tools", "validate_email", 100),
        ];

        for (component, operation, duration_ms) in operations {
            let now = SystemTime::now();
            let mut attributes = HashMap::new();
            attributes.insert("user_id".to_string(), "123".to_string());
            attributes.insert("session_id".to_string(), "session_123".to_string());

            benchmark
                .storage_manager
                .record_trace(
                    resource_id.clone(),
                    component,
                    operation,
                    now,
                    now + Duration::from_millis(duration_ms),
                    TraceStatus::Ok,
                    attributes,
                )
                .await
                .unwrap();
        }

        // 5. Create evaluation dataset and record scores
        let dataset_id = benchmark
            .storage_manager
            .create_evaluation_dataset(
                resource_id.clone(),
                "onboarding_quality_eval",
                "Evaluation dataset for onboarding flow quality",
                "1.0",
            )
            .await
            .unwrap();

        let eval_metrics = vec![
            ("user_satisfaction", 0.92),
            ("completion_rate", 0.87),
            ("time_to_complete", 0.78),
            ("clarity_score", 0.89),
        ];

        for (metric, score) in eval_metrics {
            benchmark
                .storage_manager
                .record_evaluation_score(
                    "onboarding_eval_run_001",
                    "user_session_123",
                    resource_id.clone(),
                    metric,
                    score,
                    &format!("Automated evaluation scored {} for {}", score, metric),
                    "onboarding_evaluator",
                )
                .await
                .unwrap();
        }

        // 6. Resume the workflow
        let (context, step) = benchmark
            .storage_manager
            .resume_workflow("user_onboarding_flow")
            .await
            .unwrap();
        assert_eq!(step, 0);
        assert_eq!(context["step"], "welcome");

        let total_duration = start_time.elapsed();

        println!("✅ Realistic scenario completed in {:?}", total_duration);
        println!("   - Workflow: suspended and resumed");
        println!(
            "   - Conversation: {} messages stored",
            conversation_pairs.len()
        );
        println!("   - Traces: {} operations recorded", operations.len());
        println!("   - Evaluation: {} metrics recorded", eval_metrics.len());

        // Verify final state
        let stats = benchmark.storage_manager.get_statistics().await.unwrap();
        println!("\n📊 Final Statistics:");
        println!(
            "   Storage Size: {:.2} MB",
            stats.storage_size_bytes as f64 / (1024.0 * 1024.0)
        );
        println!("   Memory Threads: {}", stats.memory_threads);
        println!("   Memory Messages: {}", stats.memory_messages);
        println!("   Traces: {}", stats.traces);
        println!("   Eval Datasets: {}", stats.eval_datasets);
        println!("   Eval Scores: {}", stats.eval_scores);

        // Performance assertions
        assert!(
            total_duration.as_secs() < 5,
            "Realistic scenario should complete quickly"
        );
        assert!(
            stats.memory_threads >= 1,
            "Should have conversation threads"
        );
        assert!(
            stats.traces >= operations.len(),
            "Should have recorded traces"
        );
        assert!(
            stats.eval_scores >= eval_metrics.len(),
            "Should have evaluation scores"
        );
    }
}

pub async fn run_all_benchmarks() {
    println!("🚀 Unified Storage System - Complete Benchmark Suite");
    println!("===================================================\n");

    let benchmark = StorageBenchmark::new();

    // Run all benchmark categories
    let workflow_results = benchmark.benchmark_workflow_operations(1000).await;
    let memory_results = benchmark.benchmark_memory_operations(25, 100).await;
    let trace_results = benchmark.benchmark_trace_operations(2000).await;
    let eval_results = benchmark.benchmark_evaluation_operations(10, 500).await;
    let concurrent_result = benchmark.benchmark_concurrent_operations(20, 200).await;
    let stats_result = benchmark.benchmark_storage_statistics(100).await;
    let cleanup_result = benchmark.benchmark_cleanup_operations().await;

    // Print all results
    for result in workflow_results {
        result.print();
    }
    for result in memory_results {
        result.print();
    }
    for result in trace_results {
        result.print();
    }
    for result in eval_results {
        result.print();
    }
    concurrent_result.print();
    stats_result.print();
    cleanup_result.print();

    println!("✅ All benchmarks completed successfully!");
}
